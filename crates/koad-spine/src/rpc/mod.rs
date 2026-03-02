use tonic::{Request, Response, Status};
use tokio_stream::Stream;
use std::pin::Pin;
use std::sync::Arc;
use std::collections::HashMap;
use crate::engine::Engine;
use koad_proto::spine::v1::spine_service_server::SpineService;
use koad_proto::spine::v1::*;
use koad_proto::kernel::kernel_service_server::KernelService;
use koad_proto::kernel::{CommandRequest, CommandResponse, TelemetryUpdate, Empty as KernelEmpty};
use fred::interfaces::{PubsubInterface, HashesInterface, EventInterface};
use serde_json::json;
use chrono::Utc;
use async_stream::try_stream;
use koad_core::intent::{Intent, ExecuteIntent};
use koad_core::identity::Rank;

pub struct KoadSpine {
    engine: Arc<Engine>,
}

impl KoadSpine {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    pub async fn register_in_inventory(&self, host: &str, port: u32) -> anyhow::Result<()> {
        let service_entry = json!({
            "name": "grpc",
            "host": host,
            "port": port,
            "protocol": "grpc",
            "status": "UP",
            "last_seen": Utc::now().timestamp()
        });

        let _: () = self.engine.redis.client.hset("koad:services", ("grpc", service_entry.to_string())).await?;
        Ok(())
    }
}

#[tonic::async_trait]
impl KernelService for KoadSpine {
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
        _request: Request<KernelEmpty>,
    ) -> Result<Response<Self::StreamTelemetryStream>, Status> {
        println!("Kernel: TUI connected to telemetry stream.");
        
        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let redis = self._engine().redis.clone(); // use private helper if needed or fix access

        tokio::spawn(async move {
            let mut message_stream = redis.subscriber.message_rx();
            
            if let Err(e) = redis.subscriber.subscribe(vec!["koad:telemetry", "koad:telemetry:stats"]).await {
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

    async fn heartbeat(&self, _request: Request<KernelEmpty>) -> Result<Response<KernelEmpty>, Status> {
        Ok(Response::new(KernelEmpty {}))
    }
}

impl KoadSpine {
    fn _engine(&self) -> &Engine {
        &self.engine
    }
}

#[tonic::async_trait]
impl SpineService for KoadSpine {
    async fn dispatch_task(
        &self,
        request: Request<DispatchTaskRequest>,
    ) -> Result<Response<DispatchTaskResponse>, Status> {
        let req = request.into_inner();
        let cmd_str = req.command;
        let identity = "admin";

        let intent = Intent::Execute(ExecuteIntent {
            identity: identity.to_string(),
            command: cmd_str,
            args: req.args,
            working_dir: if req.working_dir.is_empty() { None } else { Some(req.working_dir) },
            env_vars: req.env_vars,
        });

        let intent_str = serde_json::to_string(&intent).map_err(|e| Status::internal(e.to_string()))?;
        if let Err(e) = self.engine.redis.client.publish::<(), _, _>("koad:commands", intent_str).await {
            return Err(Status::internal(format!("Failed to dispatch task: {}", e)));
        }

        Ok(Response::new(DispatchTaskResponse {
            task_id: "pending".to_string(),
            status: TaskStatus::Pending as i32,
        }))
    }

    type StreamTaskStatusStream = Pin<Box<dyn Stream<Item = Result<TaskStatusUpdate, Status>> + Send>>;

    async fn stream_task_status(
        &self,
        _request: Request<StreamTaskStatusRequest>,
    ) -> Result<Response<Self::StreamTaskStatusStream>, Status> {
        let output = try_stream! {
            yield TaskStatusUpdate {
                task_id: "init".to_string(),
                status: TaskStatus::Pending as i32,
                stdout: "Connection established".to_string(),
                stderr: "".to_string(),
                exit_code: 0,
                updated_at: None,
            };
        };

        Ok(Response::new(Box::pin(output) as Self::StreamTaskStatusStream))
    }

    type StreamSystemEventsStream = Pin<Box<dyn Stream<Item = Result<SystemEvent, Status>> + Send>>;

    async fn stream_system_events(
        &self,
        _request: Request<StreamSystemEventsRequest>,
    ) -> Result<Response<Self::StreamSystemEventsStream>, Status> {
        let output = try_stream! {
            yield SystemEvent {
                event_id: "0".to_string(),
                source: "spine:grpc".to_string(),
                severity: EventSeverity::Info as i32,
                message: "Event stream active".to_string(),
                metadata_json: "{}".to_string(),
                timestamp: None,
            };
        };

        Ok(Response::new(Box::pin(output) as Self::StreamSystemEventsStream))
    }

    async fn get_system_state(
        &self,
        _request: Request<GetSystemStateRequest>,
    ) -> Result<Response<GetSystemStateResponse>, Status> {
        Ok(Response::new(GetSystemStateResponse {
            identity_json: "{}".to_string(),
            active_tasks: 0,
            version: "3.2.0".to_string(),
        }))
    }

    async fn get_service(
        &self,
        request: Request<GetServiceRequest>,
    ) -> Result<Response<GetServiceResponse>, Status> {
        let name = request.into_inner().name;
        let res: Option<String> = self.engine.redis.client.hget("koad:services", &name).await.map_err(|e| Status::internal(e.to_string()))?;
        
        if let Some(json_str) = res {
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(&json_str) {
                return Ok(Response::new(GetServiceResponse {
                    service: Some(ServiceEntry {
                        name: entry["name"].as_str().unwrap_or_default().to_string(),
                        host: entry["host"].as_str().unwrap_or_default().to_string(),
                        port: entry["port"].as_u64().unwrap_or_default() as u32,
                        protocol: entry["protocol"].as_str().unwrap_or_default().to_string(),
                        environment: EnvironmentType::Wsl as i32,
                        status: entry["status"].as_str().unwrap_or_default().to_string(),
                    })
                }));
            }
        }
        
        Err(Status::not_found("Service not found"))
    }

    async fn register_service(
        &self,
        request: Request<RegisterServiceRequest>,
    ) -> Result<Response<RegisterServiceResponse>, Status> {
        let entry = request.into_inner().service.ok_or_else(|| Status::invalid_argument("Missing service entry"))?;
        let payload = json!({
            "name": entry.name,
            "host": entry.host,
            "port": entry.port,
            "protocol": entry.protocol,
            "status": entry.status,
            "last_seen": Utc::now().timestamp()
        });

        let _: () = self.engine.redis.client.hset("koad:services", (entry.name, payload.to_string())).await.map_err(|e| Status::internal(e.to_string()))?;
        
        Ok(Response::new(RegisterServiceResponse { success: true }))
    }

    async fn register_component(
        &self,
        _request: Request<RegisterComponentRequest>,
    ) -> Result<Response<RegisterComponentResponse>, Status> {
        Ok(Response::new(RegisterComponentResponse {
            session_id: uuid::Uuid::new_v4().to_string(),
            authorized: true,
        }))
    }

    async fn initialize_session(
        &self,
        request: Request<InitializeSessionRequest>,
    ) -> Result<Response<SessionPackage>, Status> {
        let req = request.into_inner();
        let session_id = uuid::Uuid::new_v4().to_string();

        let rank = match req.agent_role.to_lowercase().as_str() {
            "admiral" | "admin" => Rank::Admiral,
            "captain" => Rank::Captain,
            "officer" | "pm" => Rank::Officer,
            _ => Rank::Crew,
        };

        let identity = koad_core::identity::Identity {
            name: req.agent_name,
            rank,
            permissions: vec!["all".to_string()],
        };

        let context = koad_core::session::ProjectContext {
            project_name: req.project_name,
            root_path: "".to_string(),
            allowed_paths: vec![],
            stack: vec![],
        };

        let environment = koad_core::types::EnvironmentType::Wsl;

        let session = koad_core::session::AgentSession::new(
            session_id.clone(),
            identity.clone(),
            environment,
            context.clone(),
        );

        self.engine.asm.create_session(session).await.map_err(|e| Status::internal(e.to_string()))?;
        let hydration = self.engine.asm.hydrate_session(&session_id).await.map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SessionPackage {
            session_id: session_id.clone(),
            identity_json: serde_json::to_string(&identity).unwrap(),
            project_context_json: serde_json::to_string(&context).unwrap(),
            intelligence: Some(IntelligencePackage {
                mission_briefing: hydration["mission_briefing"].as_str().unwrap_or_default().to_string(),
                active_tasks: vec![],
                recent_events: vec![],
                metadata: HashMap::new(),
            }),
        }))
    }
}
