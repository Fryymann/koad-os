//! Sector Service
//!
//! gRPC service for managing workspace sectors and Redis-backed sector state.

use fred::interfaces::KeysInterface;
use koad_core::utils::redis::RedisClient;
use koad_proto::citadel::v5::sector_server::Sector;
use koad_proto::citadel::v5::*;
use koad_sandbox::{PolicyResult, Sandbox};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

/// Service implementation for the `Sector` gRPC interface.
#[derive(Clone)]
pub struct SectorService {
    redis: Arc<RedisClient>,
    sandbox: Arc<Sandbox>,
}

impl SectorService {
    /// Creates a new `SectorService`.
    pub fn new(redis: Arc<RedisClient>, sandbox: Arc<Sandbox>) -> Self {
        Self { redis, sandbox }
    }
}

#[tonic::async_trait]
impl Sector for SectorService {
    /// Acquire a lock on a shared resource/sector.
    async fn acquire_lock(
        &self,
        request: Request<LockRequest>,
    ) -> Result<Response<LockResponse>, Status> {
        let req = request.into_inner();
        let sector_id = &req.sector_id;
        let ttl_ms = req.ttl_ms;

        let lock_id = uuid::Uuid::new_v4().to_string();
        let lock_key = format!("koad:lock:{}", sector_id);

        let acquired: bool = self
            .redis
            .pool
            .set(
                &lock_key,
                &lock_id,
                Some(fred::types::Expiration::PX(ttl_ms as i64)),
                Some(fred::types::SetOptions::NX),
                false,
            )
            .await
            .map_err(|e| Status::internal(format!("Lock acquisition failed: {}", e)))?;

        let context = req.context.map(|c| TraceContext {
            trace_id: c.trace_id,
            origin: "Citadel".to_string(),
            actor: "citadel".to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
            level: WorkspaceLevel::LevelCitadel as i32,
        });

        if acquired {
            info!("Sector: Lock acquired on '{}' (id: {})", sector_id, lock_id);
        }

        Ok(Response::new(LockResponse {
            acquired,
            lock_id: if acquired { lock_id } else { String::new() },
            context,
        }))
    }

    /// Release a lock on a shared resource/sector.
    async fn release_lock(
        &self,
        request: Request<LockRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let sector_id = &req.sector_id;
        let lock_key = format!("koad:lock:{}", sector_id);

        let deleted: i64 = self
            .redis
            .pool
            .del(&lock_key)
            .await
            .map_err(|e| Status::internal(format!("Lock release failed: {}", e)))?;

        info!(
            "Sector: Lock released on '{}' (deleted: {})",
            sector_id, deleted
        );

        Ok(Response::new(StatusResponse {
            success: deleted > 0,
            message: if deleted > 0 {
                "Lock released".to_string()
            } else {
                "Lock not found (may have expired)".to_string()
            },
            context: None,
        }))
    }

    /// Validate an agent command or intent against sandbox policies.
    async fn validate_intent(
        &self,
        request: Request<IntentRequest>,
    ) -> Result<Response<IntentResponse>, Status> {
        let req = request.into_inner();

        let result = self
            .sandbox
            .evaluate(&req.agent_name, &req.agent_rank, &req.command);

        let (allowed, reason) = match result {
            PolicyResult::Allowed => (true, "Command permitted".to_string()),
            PolicyResult::Denied(r) => (false, r),
        };

        Ok(Response::new(IntentResponse {
            allowed,
            reason,
            context: req.context,
        }))
    }
}
