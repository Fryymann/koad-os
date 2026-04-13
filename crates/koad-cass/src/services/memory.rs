//! Memory Service Implementation

use crate::storage::Storage;
use koad_intelligence::router::InferenceRouter;
use koad_proto::cass::v1::memory_service_server::MemoryService;
use koad_proto::cass::v1::{EpisodicMemory, FactCard, FactQuery, FactResponse};
use koad_proto::citadel::v5::StatusResponse;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

pub struct CassMemoryService {
    storage: Arc<dyn Storage>,
    intelligence: Arc<InferenceRouter>,
}

impl CassMemoryService {
    pub fn new(storage: Arc<dyn Storage>, intelligence: Arc<InferenceRouter>) -> Self {
        Self {
            storage,
            intelligence,
        }
    }
}

#[tonic::async_trait]
impl MemoryService for CassMemoryService {
    async fn commit_fact(
        &self,
        request: Request<FactCard>,
    ) -> Result<Response<StatusResponse>, Status> {
        let fact = request.into_inner();
        info!(domain = %fact.domain, "Memory: Committing fact");

        self.storage
            .commit_fact(fact)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StatusResponse {
            success: true,
            message: "Fact committed to ledger.".to_string(),
            context: None,
        }))
    }

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

    async fn record_episode(
        &self,
        request: Request<EpisodicMemory>,
    ) -> Result<Response<StatusResponse>, Status> {
        let ep = request.into_inner();
        info!(session = %ep.session_id, "Memory: Recording episode");

        // Intelligence: Extract facts from summary
        let summary = ep.summary.clone();
        let intelligence = self.intelligence.clone();
        tokio::spawn(async move {
            let _ = intelligence.score(&summary).await;
            // Future: auto-extract FactCards
        });

        self.storage
            .record_episode(ep)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StatusResponse {
            success: true,
            message: "Episode recorded.".to_string(),
            context: None,
        }))
    }
}
