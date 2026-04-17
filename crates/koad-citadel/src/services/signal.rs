//! Signal Service
//!
//! gRPC service for subscribing to and broadcasting events on the Koad Stream.

use crate::signal_corps::quota::QuotaValidator;
use futures::Stream;
use koad_core::signal::SignalCorps;
use koad_proto::citadel::v5::signal_server::Signal;
use koad_proto::citadel::v5::*;
use std::pin::Pin;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn};

/// Service implementation for the `Signal` gRPC interface.
#[derive(Clone)]
pub struct SignalService {
    signal_corps: Arc<SignalCorps>,
    quota: Arc<QuotaValidator>,
}

impl SignalService {
    /// Creates a new `SignalService`.
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

    /// Subscribe to the Signal Corps event stream.
    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let req = request.into_inner();
        let topics = req.topics;
        let actor = req
            .context
            .as_ref()
            .map(|c| c.actor.clone())
            .unwrap_or_else(|| "anonymous".to_string());

        info!(agent = %actor, topics = ?topics, "Signal: New subscription");

        let corps = self.signal_corps.clone();
        let agent_name = actor.clone();

        // Ensure consumer group exists
        if let Err(e) = corps.ensure_consumer_groups(&agent_name, &topics).await {
            error!("Signal: Failed to ensure consumer groups: {}", e);
            return Err(Status::internal("Failed to initialize stream subscription"));
        }

        let output = async_stream::try_stream! {
            loop {
                // Read 1 message at a time, block for 5s
                match corps.read_messages(&agent_name, &topics, Some(1), Some(5000)).await {
                    Ok(messages) => {
                        for (topic, entry_id, fields) in messages {
                            let event = Event {
                                topic: topic.clone(),
                                payload: fields.get("payload").cloned().unwrap_or_default(),
                                context: None, // In future, reconstruct from fields
                            };
                            yield event;

                            // Auto-ack for now
                            let _ = corps.ack(&agent_name, &topic, &[entry_id]).await;
                        }
                    }
                    Err(e) => {
                        warn!("Signal: Stream read error for {}: {}", agent_name, e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        };

        Ok(Response::new(Box::pin(output) as Self::SubscribeStream))
    }

    /// Broadcast a high-signal event to the bus.
    async fn broadcast(&self, request: Request<Event>) -> Result<Response<StatusResponse>, Status> {
        let event = request.into_inner();
        let topic = event.topic;
        let payload = event.payload;
        let actor = event
            .context
            .as_ref()
            .map(|c| c.actor.clone())
            .unwrap_or_else(|| "anonymous".to_string());
        let trace_id = event
            .context
            .as_ref()
            .map(|c| c.trace_id.clone())
            .unwrap_or_else(|| "unknown".to_string());

        // Quota Check
        self.quota.check_and_record(&actor, &trace_id).await?;

        info!(agent = %actor, topic = %topic, "Signal: Broadcasting event");

        self.signal_corps
            .broadcast(&topic, &payload, &trace_id, &actor)
            .await
            .map_err(|e| Status::internal(format!("Broadcast failed: {}", e)))?;

        Ok(Response::new(StatusResponse {
            success: true,
            message: "Event broadcasted".to_string(),
            context: None,
        }))
    }

    async fn send_signal(
        &self,
        request: Request<SendSignalRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(StatusResponse {
            success: true,
            message: "Signal sent (stub)".to_string(),
            context: req.context,
        }))
    }

    async fn get_signals(
        &self,
        request: Request<GetSignalsRequest>,
    ) -> Result<Response<GetSignalsResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(GetSignalsResponse {
            signals: vec![],
            context: req.context,
        }))
    }

    async fn update_signal_status(
        &self,
        request: Request<UpdateSignalStatusRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        Ok(Response::new(StatusResponse {
            success: true,
            message: "Status updated (stub)".to_string(),
            context: req.context,
        }))
    }
}
