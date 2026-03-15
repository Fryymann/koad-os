//! Stream Service Implementation
//!
//! Bridge between gRPC and the Notion-based Koad Stream.

use koad_bridge_notion::NotionClient;
use koad_proto::cass::v1::stream_service_server::StreamService;
use koad_proto::cass::v1::SignalRequest;
use koad_proto::citadel::v5::StatusResponse;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

/// Service implementation for the `StreamService` gRPC interface.
pub struct CassStreamService {
    notion: Arc<NotionClient>,
    database_id: String,
}

impl CassStreamService {
    /// Creates a new `CassStreamService`.
    pub fn new(notion: Arc<NotionClient>, database_id: String) -> Self {
        Self {
            notion,
            database_id,
        }
    }
}

#[tonic::async_trait]
impl StreamService for CassStreamService {
    /// Posts a signal to the Notion-based Koad Stream.
    ///
    /// # Errors
    /// Returns `INTERNAL` if the Notion API request fails.
    async fn post_signal(
        &self,
        request: Request<SignalRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();

        let actor = req
            .context
            .as_ref()
            .map(|c| c.actor.clone())
            .unwrap_or_else(|| "CASS".to_string());

        info!(
            author = %actor,
            target = %req.target_agent,
            topic = %req.topic,
            "Stream: Posting signal to Notion"
        );

        // TODO: In a full implementation, the NotionClient would be a trait to allow mocking.
        // For now, we wrap the call in a Result map.
        self.notion
            .post_to_stream(
                &self.database_id,
                &actor,
                &req.target_agent,
                &req.topic,
                &req.priority,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(StatusResponse {
            success: true,
            message: "Signal posted to Koad Stream".to_string(),
            context: None,
        }))
    }
}
