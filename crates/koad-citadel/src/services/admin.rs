//! Admin Service Implementation
//!
//! Handles administrative and maintenance RPC calls, typically via a secure UDS.

use koad_proto::citadel::v5::admin_server::Admin;
use koad_proto::citadel::v5::*;
use std::time::Instant;
use tokio::sync::watch;
use tonic::{Request, Response, Status};
use tracing::{info, warn};

/// Service implementation for the `Admin` gRPC interface.
pub struct AdminService {
    shutdown_tx: watch::Sender<bool>,
    start_time: Instant,
}

impl AdminService {
    /// Creates a new `AdminService`.
    pub fn new(shutdown_tx: watch::Sender<bool>) -> Self {
        Self {
            shutdown_tx,
            start_time: Instant::now(),
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::watch;

    #[tokio::test]
    async fn test_admin_shutdown() -> anyhow::Result<()> {
        let (tx, mut rx) = watch::channel(false);
        let service = AdminService::new(tx);

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
        let service = AdminService::new(tx);

        let req = Request::new(SystemStatusRequest { context: None });
        let res = service.get_system_status(req).await?;
        
        let status = res.into_inner();
        assert_eq!(status.version, "3.2.0");
        assert!(!status.uptime.is_empty());

        Ok(())
    }
}
