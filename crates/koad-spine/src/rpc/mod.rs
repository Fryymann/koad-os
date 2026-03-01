use tonic::{Request, Response, Status};
use tokio_stream::Stream;
use std::pin::Pin;
use std::sync::Arc;
use koad_proto::kernel::kernel_service_server::KernelService;
use koad_proto::kernel::{CommandRequest, CommandResponse, TelemetryUpdate, Empty};
use crate::engine::Engine;
use fred::interfaces::{PubsubInterface, EventInterface};

pub struct KoadKernel {
    _engine: Arc<Engine>,
}

impl KoadKernel {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { _engine: engine }
    }
}

#[tonic::async_trait]
impl KernelService for KoadKernel {
    async fn execute(
        &self,
        request: Request<CommandRequest>,
    ) -> Result<Response<CommandResponse>, Status> {
        let req = request.into_inner();
        println!("Kernel: Executing command [{}] from {}", req.name, req.identity);

        Ok(Response::new(CommandResponse {
            command_id: req.command_id,
            success: true,
            output: format!("Command '{}' executed successfully by v3 Kernel.", req.name),
            error: "".to_string(),
        }))
    }

    type StreamTelemetryStream = Pin<Box<dyn Stream<Item = Result<TelemetryUpdate, Status>> + Send>>;

    async fn stream_telemetry(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::StreamTelemetryStream>, Status> {
        println!("Kernel: TUI connected to telemetry stream.");
        
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let redis = self._engine.redis.clone();

        tokio::spawn(async move {
            let mut message_stream = redis.client.message_rx();
            
            if let Err(e) = redis.client.subscribe("koad:telemetry").await {
                eprintln!("Kernel Telemetry Error: Failed to subscribe to Redis: {}", e);
                return;
            }

            while let Ok(message) = message_stream.recv().await {
                let payload = message.value.as_string().unwrap_or_default();
                if tx.send(Ok(TelemetryUpdate {
                    source: "redis".to_string(),
                    message: payload,
                    timestamp: chrono::Utc::now().timestamp(),
                    level: 0, // INFO
                })).await.is_err() {
                    break;
                }
            }
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(output_stream) as Self::StreamTelemetryStream))
    }

    async fn heartbeat(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        Ok(Response::new(Empty {}))
    }
}
