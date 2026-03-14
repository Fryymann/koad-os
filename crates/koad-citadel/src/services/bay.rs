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
    /// Provisions a new personal bay for an agent.
    ///
    /// # Errors
    /// Returns `INTERNAL` if the filesystem or database operations fail.
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

        info!("PersonalBay: Provisioned bay for '{}'", agent_name);

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Bay provisioned for '{}'", agent_name),
            context: None,
        }))
    }

    /// Creates a git worktree assigned to an agent for a specific task.
    ///
    /// # Errors
    /// Returns `INVALID_ARGUMENT` if `task_id` is missing.
    /// Returns `INTERNAL` if worktree creation or mapping fails.
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
        });

        Ok(Response::new(WorkspaceResponse {
            worktree_path: worktree_path.to_string_lossy().to_string(),
            context,
        }))
    }

    /// Queries the current health and status of an agent's personal bay.
    ///
    /// # Errors
    /// Returns `NOT_FOUND` if the bay has not been provisioned.
    async fn get_status(&self, request: Request<BayQuery>) -> Result<Response<BayStatus>, Status> {
        let req = request.into_inner();
        let agent_name = &req.agent_name;

        let health = self
            .bay_store
            .get_health(agent_name)
            .await
            .map_err(|e| Status::not_found(format!("Bay not found: {}", e)))?;

        let now = Utc::now();
        let context = req.context.map(|c| TraceContext {
            trace_id: c.trace_id,
            origin: "Citadel".to_string(),
            actor: "citadel".to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: now.timestamp(),
                nanos: 0,
            }),
        });

        Ok(Response::new(BayStatus {
            agent_name: agent_name.clone(),
            health,
            last_sync: now.to_rfc3339(),
            context,
        }))
    }
}
