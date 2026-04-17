//! Personal Bay Service
//!
//! Provides gRPC handlers for provisioning agent bays, creating worktrees,
//! and querying bay status.

use crate::state::bay_store::BayStore;
use crate::workspace::manager::WorkspaceManager;
use chrono::Utc;
use koad_proto::citadel::v5::personal_bay_server::PersonalBay;
use koad_proto::citadel::v5::*;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

/// Service implementation for the `PersonalBay` gRPC interface.
#[derive(Clone)]
pub struct PersonalBayService {
    bay_store: Arc<BayStore>,
    workspace_manager: Arc<WorkspaceManager>,
}

impl PersonalBayService {
    /// Creates a new `PersonalBayService`.
    pub fn new(bay_store: Arc<BayStore>, workspace_manager: Arc<WorkspaceManager>) -> Self {
        Self {
            bay_store,
            workspace_manager,
        }
    }
}

#[tonic::async_trait]
impl PersonalBay for PersonalBayService {
    /// Provision a Personal Bay for an agent.
    async fn provision(
        &self,
        request: Request<ProvisionRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let agent_name = &req.agent_name;

        self.bay_store
            .provision(agent_name)
            .await
            .map_err(|e| Status::internal(format!("Bay provisioning failed: {}", e)))?;

        if req.initial_xp > 0 {
            // Seed XP if provided (usually for first-time migration or manual sync)
            let _ = self
                .bay_store
                .update_xp_and_level(agent_name, req.initial_xp, 1)
                .await;
        }

        info!("PersonalBay: Provisioned bay for '{}'", agent_name);

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Bay provisioned for '{}'", agent_name),
            context: None,
        }))
    }

    /// Provision a workspace (git worktree) for a task.
    async fn provision_workspace(
        &self,
        request: Request<WorkspaceRequest>,
    ) -> Result<Response<WorkspaceResponse>, Status> {
        let req = request.into_inner();
        let agent_name = &req.agent_name;
        let task_id = &req.task_id;

        if task_id.is_empty() {
            return Err(Status::invalid_argument("task_id is required"));
        }

        let worktree_path = self
            .workspace_manager
            .create_worktree(agent_name, task_id, &self.bay_store)
            .await
            .map_err(|e| Status::internal(format!("Worktree creation failed: {}", e)))?;

        let now = Utc::now();
        let context = req.context.map(|c| TraceContext {
            trace_id: c.trace_id,
            origin: "Citadel".to_string(),
            actor: "citadel".to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: now.timestamp(),
                nanos: 0,
            }),
            level: WorkspaceLevel::LevelCitadel as i32,
        });

        Ok(Response::new(WorkspaceResponse {
            worktree_path: worktree_path.to_string_lossy().to_string(),
            context,
        }))
    }

    /// Query the health and status of a bay.
    async fn get_status(&self, request: Request<BayQuery>) -> Result<Response<BayStatus>, Status> {
        let req = request.into_inner();
        let agent_name = &req.agent_name;

        let health = self
            .bay_store
            .get_health(agent_name)
            .await
            .map_err(|e| Status::not_found(format!("Bay not found: {}", e)))?;

        let (xp, level) = self
            .bay_store
            .get_xp_and_level(agent_name)
            .await
            .unwrap_or((0, 1));

        let now = Utc::now();
        let context = req.context.map(|c| TraceContext {
            trace_id: c.trace_id,
            origin: "Citadel".to_string(),
            actor: "citadel".to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: now.timestamp(),
                nanos: 0,
            }),
            level: WorkspaceLevel::LevelCitadel as i32,
        });

        Ok(Response::new(BayStatus {
            agent_name: agent_name.clone(),
            health,
            last_sync: now.to_rfc3339(),
            xp,
            level,
            context,
        }))
    }
}
