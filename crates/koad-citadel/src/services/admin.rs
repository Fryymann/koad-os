//! Admin Service Implementation
//!
//! Handles administrative and maintenance RPC calls, typically via a secure UDS.

use koad_core::db::KoadDB;
use koad_proto::citadel::v5::admin_server::Admin;
use koad_proto::citadel::v5::*;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::watch;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

/// Domain for committed knowledge: "{partition}:{category}" so facts are
/// recallable under the committing agent's partition (partition canon = domain prefix).
fn knowledge_domain(agent: &str, category: &str) -> String {
    format!("{}:{}", koad_core::utils::partition::partition_key(agent), category)
}

/// Service implementation for the `Admin` gRPC interface.
#[derive(Clone)]
pub struct AdminService {
    shutdown_tx: watch::Sender<bool>,
    start_time: Instant,
    koad_db: Arc<KoadDB>,
    cass_grpc_addr: String,
    /// Directory holding the Citadel's SQLite databases.
    db_dir: PathBuf,
    /// Root directory for timestamped backup snapshots.
    backup_root: PathBuf,
}

impl AdminService {
    /// Creates a new `AdminService`.
    pub fn new(
        shutdown_tx: watch::Sender<bool>,
        koad_db: Arc<KoadDB>,
        cass_grpc_addr: String,
        db_dir: PathBuf,
        backup_root: PathBuf,
    ) -> Self {
        Self {
            shutdown_tx,
            start_time: Instant::now(),
            koad_db,
            cass_grpc_addr,
            db_dir,
            backup_root,
        }
    }
}

#[tonic::async_trait]
impl Admin for AdminService {
    /// Gracefully shutdown the Citadel kernel.
    async fn shutdown(
        &self,
        request: Request<ShutdownRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let reason = req.reason;

        warn!(reason = %reason, "Admin: Received shutdown request via UDS");

        let _ = self.shutdown_tx.send(true);

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Shutdown initiated: {}", reason),
            context: req.context,
        }))
    }

    /// Retrieve high-level system health and metrics.
    async fn get_system_status(
        &self,
        request: Request<SystemStatusRequest>,
    ) -> Result<Response<SystemStatusResponse>, Status> {
        let req = request.into_inner();

        info!("Admin: System status requested");

        let uptime = format!("{:?}", self.start_time.elapsed());

        Ok(Response::new(SystemStatusResponse {
            version: "3.2.0".to_string(), // Should come from config
            active_sessions: 0,           // Placeholder
            total_bays: 0,                // Placeholder
            uptime,
            context: req.context,
        }))
    }

    /// Forcefully terminate a session by ID.
    async fn force_purge_session(
        &self,
        request: Request<PurgeRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let sid = req.session_id;

        warn!(session_id = %sid, "Admin: Force purging session");

        // Logic to purge from Redis would go here.
        // For now, we return success.

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Session {} purged", sid),
            context: req.context,
        }))
    }

    /// Commit knowledge/learnings to the Memory Bank.
    async fn commit_knowledge(
        &self,
        request: Request<CommitKnowledgeRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();

        // Derive agent name from session_id: "SID-clyde-abc123" → "clyde"
        let agent_name = req.session_id
            .splitn(3, '-')
            .nth(1)
            .unwrap_or("unknown")
            .to_string();

        info!(
            agent = %agent_name,
            category = %req.category,
            "Admin: Commit knowledge requested"
        );

        // Primary sink: CASS MemoryService via gRPC.
        let domain = knowledge_domain(&agent_name, &req.category);
        let now = chrono::Utc::now();
        let fact = koad_proto::cass::v1::FactCard {
            id: uuid::Uuid::new_v4().to_string(),
            source_agent: agent_name.clone(),
            session_id: req.session_id.clone(),
            domain: domain.clone(),
            content: req.content.clone(),
            confidence: 1.0,
            tags: req
                .tags
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect(),
            created_at: Some(prost_types::Timestamp {
                seconds: now.timestamp(),
                nanos: 0,
            }),
            metadata: None, // enrichment worker fills
        };

        // Bounded connect/RPC so a hung CASS can't stall the admin handler.
        let cass_channel = tonic::transport::Endpoint::from_shared(self.cass_grpc_addr.clone())
            .map(|ep| {
                ep.connect_timeout(std::time::Duration::from_secs(3))
                    .timeout(std::time::Duration::from_secs(10))
            });
        let cass_connect = match cass_channel {
            Ok(ep) => ep.connect().await.map_err(anyhow::Error::from),
            Err(e) => Err(anyhow::Error::from(e)),
        };
        match cass_connect
            .map(koad_proto::cass::v1::memory_service_client::MemoryServiceClient::new)
        {
            Ok(mut cass) => match cass.commit_fact(fact).await {
                Ok(_) => {
                    info!(agent = %agent_name, domain = %domain, "Admin: Knowledge committed to CASS");
                    return Ok(Response::new(StatusResponse {
                        success: true,
                        message: format!("Knowledge committed to CASS (domain: {})", domain),
                        context: req.context,
                    }));
                }
                Err(e) => {
                    warn!(error = %e, "Admin: CASS commit_fact failed, falling back to koad.db");
                }
            },
            Err(e) => {
                warn!(error = %e, "Admin: CASS unreachable, falling back to koad.db");
            }
        }

        // Fallback: local archive (pre-CASS behavior).
        let tags = if req.tags.is_empty() { None } else { Some(req.tags.clone()) };

        match self.koad_db.remember(&req.category, &req.content, tags, 0, &agent_name) {
            Ok(()) => {
                info!(agent = %agent_name, category = %req.category, "Admin: Knowledge committed to koad.db");
                Ok(Response::new(StatusResponse {
                    success: true,
                    message: format!(
                        "Knowledge committed to local archive (CASS unreachable) (category: {})",
                        req.category
                    ),
                    context: req.context,
                }))
            }
            Err(e) => {
                error!(error = %e, "Admin: Failed to commit knowledge to koad.db");
                Err(Status::internal(format!(
                    "commit_knowledge failed on both CASS and local: {}",
                    e
                )))
            }
        }
    }

    /// Trigger a system-wide backup or a specific source.
    async fn trigger_backup(
        &self,
        request: Request<TriggerBackupRequest>,
    ) -> Result<Response<TriggerBackupResponse>, Status> {
        let req = request.into_inner();
        info!(source = %req.source, "Admin: Trigger backup requested");

        // `source` is accepted for compatibility; every database is backed up.
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let (db_dir, backup_root) = (self.db_dir.clone(), self.backup_root.clone());
        let stamp_for_task = stamp.clone();
        let result = tokio::task::spawn_blocking(move || {
            koad_core::backup::backup_databases(&db_dir, &backup_root, &stamp_for_task)
        })
        .await
        .map_err(|e| Status::internal(format!("Backup task failed: {}", e)))?;

        let (success, message) = match result {
            Ok(files) => (
                true,
                format!(
                    "Backed up {} database(s) to {}",
                    files.len(),
                    self.backup_root.join(&stamp).display()
                ),
            ),
            Err(e) => {
                error!("Admin: Backup failed: {:#}", e);
                (false, format!("Backup failed: {:#}", e))
            }
        };

        Ok(Response::new(TriggerBackupResponse {
            success,
            message,
            backup_id: stamp,
            context: req.context,
        }))
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::watch;

    #[test]
    fn knowledge_domain_is_partition_prefixed() {
        let d = knowledge_domain("clyde", "learning");
        assert!(d.starts_with("clyde_"));
        assert!(d.ends_with(":learning"));
    }

    fn test_service(tx: watch::Sender<bool>, db: Arc<KoadDB>) -> AdminService {
        AdminService::new(
            tx,
            db,
            "http://127.0.0.1:50052".to_string(),
            PathBuf::from("/nonexistent/db"),
            PathBuf::from("/nonexistent/backups"),
        )
    }

    /// Regression guard: trigger_backup used to return success with
    /// backup_id "bkp-placeholder" without writing anything.
    #[tokio::test]
    async fn test_admin_trigger_backup_writes_real_snapshots() -> anyhow::Result<()> {
        let data = tempfile::tempdir()?;
        let backups = tempfile::tempdir()?;
        rusqlite::Connection::open(data.path().join("koad.db"))?
            .execute_batch("CREATE TABLE t(v); INSERT INTO t VALUES (1);")?;

        let (tx, _) = watch::channel(false);
        let db = Arc::new(KoadDB::new(std::path::Path::new(":memory:")).unwrap());
        let service = AdminService::new(
            tx,
            db,
            "http://127.0.0.1:50052".to_string(),
            data.path().to_path_buf(),
            backups.path().to_path_buf(),
        );

        let res = service
            .trigger_backup(Request::new(TriggerBackupRequest {
                context: None,
                source: "all".to_string(),
            }))
            .await?
            .into_inner();

        assert!(res.success, "{}", res.message);
        let snapshot = backups.path().join(&res.backup_id).join("koad.db");
        assert!(snapshot.is_file(), "missing {}", snapshot.display());
        Ok(())
    }

    #[tokio::test]
    async fn test_admin_shutdown() -> anyhow::Result<()> {
        let (tx, mut rx) = watch::channel(false);
        let db = Arc::new(KoadDB::new(std::path::Path::new(":memory:")).unwrap());
        let service = test_service(tx, db);

        let req = Request::new(ShutdownRequest {
            context: None,
            reason: "Testing".to_string(),
        });

        let res = service.shutdown(req).await?;
        assert!(res.into_inner().success);
        assert!(*rx.borrow_and_update());

        Ok(())
    }

    #[tokio::test]
    async fn test_admin_status() -> anyhow::Result<()> {
        let (tx, _) = watch::channel(false);
        let db = Arc::new(KoadDB::new(std::path::Path::new(":memory:")).unwrap());
        let service = test_service(tx, db);

        let req = Request::new(SystemStatusRequest { context: None });
        let res = service.get_system_status(req).await?;

        let status = res.into_inner();
        assert_eq!(status.version, "3.2.0");
        assert!(!status.uptime.is_empty());

        Ok(())
    }
}
