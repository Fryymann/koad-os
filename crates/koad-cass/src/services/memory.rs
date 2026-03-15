//! Memory Service Implementation
//!
//! Handles RPC calls for committing and querying persistent agent memory.

use koad_intelligence::router::InferenceRouter;
use koad_proto::cass::v1::memory_service_server::MemoryService;
use koad_proto::cass::v1::{EpisodicMemory, FactCard, FactQuery, FactResponse};
use koad_proto::citadel::v5::StatusResponse;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

/// Service implementation for the `MemoryService` gRPC interface.
pub struct CassMemoryService {
    storage: Arc<dyn crate::storage::Storage>,
    intelligence: Arc<InferenceRouter>,
}

impl CassMemoryService {
    /// Creates a new `CassMemoryService`.
    pub fn new(
        storage: Arc<dyn crate::storage::Storage>,
        intelligence: Arc<InferenceRouter>,
    ) -> Self {
        Self {
            storage,
            intelligence,
        }
    }
}

#[tonic::async_trait]
impl MemoryService for CassMemoryService {
    /// Commits a fact card to the persistent ledger.
    async fn commit_fact(
        &self,
        request: Request<FactCard>,
    ) -> Result<Response<StatusResponse>, Status> {
        let mut fact = request.into_inner();

        // Automated Significance Scoring (Phase 3)
        // Only re-score if confidence is at default (0.0 or 1.0) and content is present
        if (fact.confidence == 0.0 || fact.confidence == 1.0) && !fact.content.is_empty() {
            if let Ok(score) = self.intelligence.score(&fact.content).await {
                info!("MemoryService: Scored fact significance as {}", score);
                fact.confidence = score;
            }
        }

        self.storage
            .commit_fact(fact)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StatusResponse {
            success: true,
            message: "Fact committed to ledger".to_string(),
            context: None,
        }))
    }

    /// Queries facts based on domain and tags.
    async fn query_facts(
        &self,
        request: Request<FactQuery>,
    ) -> Result<Response<FactResponse>, Status> {
        let req = request.into_inner();
        let facts = self
            .storage
            .query_facts(&req.domain, &req.tags, req.limit)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(FactResponse { facts }))
    }

    /// Records a summary of a session as an episodic memory.
    async fn record_episode(
        &self,
        request: Request<EpisodicMemory>,
    ) -> Result<Response<StatusResponse>, Status> {
        let episode = request.into_inner();
        self.storage
            .record_episode(episode)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StatusResponse {
            success: true,
            message: "Episode recorded".to_string(),
            context: None,
        }))
    }
}
