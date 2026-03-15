//! Signal Service
//!
//! gRPC service wrapping [`SignalCorps`] for broadcast and subscribe operations,
//! with per-agent quota enforcement via [`QuotaValidator`].

use crate::signal_corps::quota::QuotaValidator;
use koad_core::signal::SignalCorps;
use koad_proto::citadel::v5::signal_server::Signal;
use koad_proto::citadel::v5::*;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};
use tracing::info;

#[derive(Clone)]
pub struct SignalService {
    signal_corps: Arc<SignalCorps>,
    quota: Arc<QuotaValidator>,
}

impl SignalService {
    pub fn new(signal_corps: Arc<SignalCorps>, quota: Arc<QuotaValidator>) -> Self {
        Self {
            signal_corps,
            quota,
        }
    }
}

#[tonic::async_trait]
impl Signal for SignalService {
    type SubscribeStream = Pin<Box<dyn Stream<Item = Result<Event, Status>> + Send + 'static>>;

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let req = request.into_inner();
        let topics = req.topics.clone();
        let actor = req
            .context
            .as_ref()
            .map(|c| c.actor.clone())
            .unwrap_or_else(|| "unknown".to_string());

        info!("Signal: '{}' subscribing to topics: {:?}", actor, topics);

        // Ensure consumer groups exist
        self.signal_corps
            .ensure_consumer_groups(&actor, &topics)
            .await
            .map_err(|e| Status::internal(format!("Consumer group setup failed: {}", e)))?;

        let corps = self.signal_corps.clone();
        let agent_name = actor.clone();
        let topics_clone = topics.clone();

        let stream = async_stream::stream! {
            loop {
                match corps.read_messages(&agent_name, &topics_clone, Some(10), Some(5000)).await {
                    Ok(messages) => {
                        for (topic, entry_id, fields) in &messages {
                            let payload = fields.get("payload").cloned().unwrap_or_default();
                            let trace_id = fields.get("trace_id").cloned().unwrap_or_default();
                            let msg_actor = fields.get("actor").cloned().unwrap_or_default();

                            yield Ok(Event {
                                topic: topic.clone(),
                                payload,
                                context: Some(TraceContext {
                                    trace_id,
                                    origin: "Citadel".to_string(),
                                    actor: msg_actor,
                                    timestamp: Some(prost_types::Timestamp {
                                        seconds: chrono::Utc::now().timestamp(),
                                        nanos: 0,
                                    }),
                                    level: WorkspaceLevel::LevelCitadel as i32,
                                }),
                            });

                            // ACK the message
                            let _ = corps.ack(&agent_name, topic, &[entry_id.clone()]).await;
                        }
                    }
                    Err(e) => {
                        yield Err(Status::internal(format!("Stream read error: {}", e)));
                        break;
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(stream)))
    }

    async fn broadcast(&self, request: Request<Event>) -> Result<Response<StatusResponse>, Status> {
        let event = request.into_inner();
        let topic = &event.topic;
        let payload = &event.payload;

        let (trace_id, actor) = event
            .context
            .as_ref()
            .map(|c| (c.trace_id.clone(), c.actor.clone()))
            .unwrap_or_else(|| ("unknown".to_string(), "unknown".to_string()));

        // Check quota
        let signal_id = uuid::Uuid::new_v4().to_string();
        self.quota.check_and_record(&actor, &signal_id).await?;

        // Broadcast to stream
        self.signal_corps
            .broadcast(topic, payload, &trace_id, &actor)
            .await
            .map_err(|e| Status::internal(format!("Broadcast failed: {}", e)))?;

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Event broadcast to '{}'", topic),
            context: None,
        }))
    }
}
