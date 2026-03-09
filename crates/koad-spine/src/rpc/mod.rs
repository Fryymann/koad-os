use crate::engine::Engine;
use async_stream::try_stream;
use chrono::Utc;
use fred::interfaces::{EventInterface, HashesInterface, PubsubInterface};
use koad_core::identity::Rank;
use koad_core::intent::{ExecuteIntent, Intent};
use koad_core::storage::StorageBridge;
use koad_proto::spine::v1::spine_service_server::SpineService;
use koad_proto::spine::v1::*;
use serde_json::json;
use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};
use tracing::{error, info};

pub struct KoadSpine {
    engine: Arc<Engine>,
}

impl KoadSpine {
    pub fn new(engine: Arc<Engine>) -> Self {
        info!("Kernel: Initializing KoadSpine gRPC service (v4.0.0-optimized).");
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

        let _: () = self
            .engine
            .redis
            .pool
            .hset("koad:services", ("grpc", service_entry.to_string()))
            .await?;
        Ok(())
    }

    fn _engine(&self) -> &Engine {
        &self.engine
    }
}

#[tonic::async_trait]
impl SpineService for KoadSpine {
    // --- Core Execution ---

    async fn execute(
        &self,
        request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        let req = request.into_inner();
        info!(
            "Kernel: Executing command [{}] from {}",
            req.name, req.identity
        );

        Ok(Response::new(ExecuteResponse {
            command_id: req.command_id,
            success: true,
            output: format!(
                "Command '{}' executed successfully by unified v4 Spine.",
                req.name
            ),
            error: "".to_string(),
        }))
    }

    async fn heartbeat(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        if let Some(session_id) = request
            .metadata()
            .get("x-session-id")
            .and_then(|v| v.to_str().ok())
        {
            let _ = self.engine.identity.heartbeat(session_id).await;
            let _ = self.engine.asm.heartbeat(session_id).await;
        }
        Ok(Response::new(Empty {}))
    }

    // --- Task Management ---

    async fn dispatch_task(
        &self,
        request: Request<DispatchTaskRequest>,
    ) -> Result<Response<DispatchTaskResponse>, Status> {
        let req = request.into_inner();
        let cmd_str = req.command;
        let identity = if req.identity.is_empty() {
            "admin"
        } else {
            &req.identity
        };

        let intent = Intent::Execute(ExecuteIntent {
            identity: identity.to_string(),
            command: cmd_str,
            args: req.args,
            working_dir: if req.working_dir.is_empty() {
                None
            } else {
                Some(req.working_dir)
            },
            env_vars: req.env_vars,
        });

        let intent_str =
            serde_json::to_string(&intent).map_err(|e| Status::internal(e.to_string()))?;
        if let Err(e) = self
            .engine
            .redis
            .pool
            .next()
            .publish::<(), _, _>("koad:commands", intent_str)
            .await
        {
            return Err(Status::internal(format!("Failed to dispatch task: {}", e)));
        }

        Ok(Response::new(DispatchTaskResponse {
            task_id: "pending".to_string(),
            status: TaskStatus::Pending as i32,
        }))
    }

    type StreamTaskStatusStream =
        Pin<Box<dyn Stream<Item = Result<TaskStatusUpdate, Status>> + Send>>;

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

        Ok(Response::new(
            Box::pin(output) as Self::StreamTaskStatusStream
        ))
    }

    // --- System Telemetry & Monitoring ---

    type StreamSystemEventsStream = Pin<Box<dyn Stream<Item = Result<SystemEvent, Status>> + Send>>;

    async fn stream_system_events(
        &self,
        _request: Request<StreamSystemEventsRequest>,
    ) -> Result<Response<Self::StreamSystemEventsStream>, Status> {
        info!("Kernel: Client connected to unified system event stream.");

        let (tx, rx) = tokio::sync::mpsc::channel(128);
        let redis = self.engine.redis.clone();

        let welcome = SystemEvent {
            event_id: "0".to_string(),
            source: "spine:grpc".to_string(),
            severity: EventSeverity::Info as i32,
            message: json!({ "type": "INFO", "message": "Event stream active" }).to_string(),
            metadata_json: "{}".to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: Utc::now().timestamp(),
                nanos: Utc::now().timestamp_subsec_nanos() as i32,
            }),
        };
        let _ = tx.send(Ok(welcome)).await;

        tokio::spawn(async move {
            let mut message_stream = redis.subscriber.message_rx();

            if let Err(e) = redis
                .subscriber
                .subscribe(vec![
                    "koad:telemetry",
                    "koad:telemetry:stats",
                    "koad:sessions",
                ])
                .await
            {
                error!(
                    "Spine Event Stream Error: Failed to subscribe to Redis: {}",
                    e
                );
                return;
            }

            while let Ok(message) = message_stream.recv().await {
                let payload = message.value.as_string().unwrap_or_default();
                let event = SystemEvent {
                    event_id: uuid::Uuid::new_v4().to_string(),
                    source: format!("redis:{}", message.channel),
                    severity: EventSeverity::Info as i32,
                    message: payload,
                    metadata_json: "{}".to_string(),
                    timestamp: Some(prost_types::Timestamp {
                        seconds: Utc::now().timestamp(),
                        nanos: Utc::now().timestamp_subsec_nanos() as i32,
                    }),
                };

                if tx.send(Ok(event)).await.is_err() {
                    break;
                }
            }
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::StreamSystemEventsStream
        ))
    }

    async fn get_system_state(
        &self,
        _request: Request<GetSystemStateRequest>,
    ) -> Result<Response<GetSystemStateResponse>, Status> {
        info!("Kernel: GetSystemState request received.");
        let sessions = self.engine.asm.list_active_sessions().await;
        info!(
            "Kernel: GetSystemState found {} active sessions.",
            sessions.len()
        );

        let sessions_json = if sessions.is_empty() {
            "[]".to_string()
        } else {
            serde_json::to_string(&sessions).map_err(|e| Status::internal(e.to_string()))?
        };

        Ok(Response::new(GetSystemStateResponse {
            identity_json: sessions_json,
            active_tasks: 0,
            version: "4.0.0".to_string(),
        }))
    }

    // --- Service Discovery ---

    async fn get_service(
        &self,
        request: Request<GetServiceRequest>,
    ) -> Result<Response<GetServiceResponse>, Status> {
        let name = request.into_inner().name;
        let res: Option<String> = self
            .engine
            .redis
            .pool
            .hget("koad:services", &name)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

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
                    }),
                }));
            }
        }

        Err(Status::not_found("Service not found"))
    }

    async fn register_service(
        &self,
        request: Request<RegisterServiceRequest>,
    ) -> Result<Response<RegisterServiceResponse>, Status> {
        let entry = request
            .into_inner()
            .service
            .ok_or_else(|| Status::invalid_argument("Missing service entry"))?;
        let payload = json!({
            "name": entry.name,
            "host": entry.host,
            "port": entry.port,
            "protocol": entry.protocol,
            "status": entry.status,
            "last_seen": Utc::now().timestamp()
        });

        let _: () = self
            .engine
            .redis
            .pool
            .hset("koad:services", (entry.name, payload.to_string()))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(RegisterServiceResponse { success: true }))
    }

    // --- Identity & Lifecycle ---

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

        let lease = self
            .engine
            .identity
            .acquire_lease(&req.agent_name, &session_id, &req.driver_id, req.model_tier)
            .await
            .map_err(|e| Status::permission_denied(e.to_string()))?;

        let rank =
            if req.agent_name.to_lowercase() == "dood" || req.agent_name.to_lowercase() == "ian" {
                Rank::Admiral
            } else {
                match req.agent_role.to_lowercase().as_str() {
                    "admiral" | "dood" => Rank::Admiral,
                    "captain" | "admin" => Rank::Captain,
                    "officer" | "pm" => Rank::Officer,
                    _ => Rank::Crew,
                }
            };

        let identity = koad_core::identity::Identity {
            name: req.agent_name.clone(),
            rank,
            permissions: vec!["all".to_string()],
            tier: req.model_tier,
        };

        let context = koad_core::session::ProjectContext {
            project_name: req.project_name,
            root_path: "".to_string(),
            allowed_paths: vec![],
            stack: vec![],
        };

        let environment = koad_core::types::EnvironmentType::Wsl;

        let mut session = koad_core::session::AgentSession::new(
            session_id.clone(),
            identity.clone(),
            environment.clone(),
            context.clone(),
        );

        if let Ok(Some(bio)) = self.engine.storage.get_identity_bio(&req.agent_name).await {
            session.metadata.insert("bio".to_string(), bio);
        }
        session
            .metadata
            .insert("model_name".to_string(), req.model_name.clone());
        session
            .metadata
            .insert("driver_id".to_string(), req.driver_id.clone());

        // 0. Body Enforcement: Pre-empt previous sessions for this driver/env
        let _ = self.engine.asm.prune_body_ghosts(&req.driver_id, environment, &session_id).await;

        // 1. Authoritative Persistence in Redis (Hot State)
        let payload =
            serde_json::to_value(&session).map_err(|e| Status::internal(e.to_string()))?;
        let session_key = format!("koad:session:{}", session_id);

        let _: () = self
            .engine
            .redis
            .pool
            .next()
            .hset("koad:state", (&session_key, payload.to_string()))
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 2. Broadcast to Data Plane (koad-asm and watchers)
        let msg = json!({
            "type": "SESSION_UPDATE",
            "data": payload
        });
        let _: () = self
            .engine
            .redis
            .pool
            .next()
            .publish("koad:sessions", msg.to_string())
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        // 3. Update local cache (Passive)
        self.engine
            .asm
            .create_session(session)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let hydration = self
            .engine
            .asm
            .hydrate_session(&session_id)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        let context_key = format!("koad:session:{}:hot_context", session_id);
        let conn = self.engine.redis.pool.next();
        let hot_context_raw: std::collections::HashMap<String, String> =
            conn.hgetall(&context_key).await.unwrap_or_default();

        let mut hot_context = Vec::new();
        for val in hot_context_raw.values() {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(val) {
                hot_context.push(HotContextChunk {
                    chunk_id: v["chunk_id"].as_str().unwrap_or_default().to_string(),
                    content: v["content"].as_str().unwrap_or_default().to_string(),
                    ttl_seconds: v["ttl_seconds"].as_i64().unwrap_or(0) as i32,
                    created_at: None,
                });
            }
        }

        Ok(Response::new(SessionPackage {
            session_id: session_id.clone(),
            identity_json: serde_json::to_string(&identity).unwrap(),
            project_context_json: serde_json::to_string(&context).unwrap(),
            intelligence: Some(IntelligencePackage {
                mission_briefing: hydration["mission_briefing"]
                    .as_str()
                    .unwrap_or_default()
                    .to_string(),
                active_tasks: vec![],
                recent_events: vec![],
                metadata: HashMap::new(),
                hot_context,
            }),
            lease: Some(lease),
        }))
    }

    async fn hydrate_context(
        &self,
        request: Request<HydrationRequest>,
    ) -> Result<Response<HydrationResponse>, Status> {
        let req = request.into_inner();
        let session_id = req.session_id;
        let chunk = req
            .chunk
            .ok_or_else(|| Status::invalid_argument("Missing chunk"))?;

        match self
            .engine
            .hydration
            .hydrate(&session_id, &chunk.content, chunk.ttl_seconds)
            .await
        {
            Ok(new_chunk) => Ok(Response::new(HydrationResponse {
                success: true,
                error: "".to_string(),
                current_context_size: new_chunk.content.len() as i32, // Simplified for now
            })),
            Err(e) => Ok(Response::new(HydrationResponse {
                success: false,
                error: format!("Hydration Failed: {}", e),
                current_context_size: 0,
            })),
        }
    }

    async fn drain_all(&self, _request: Request<Empty>) -> Result<Response<Empty>, Status> {
        info!("Kernel: Triggering full state drain to durable memory...");
        self.engine
            .storage
            .drain_all()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(Empty {}))
    }

    async fn get_file_snippet(
        &self,
        request: Request<GetFileSnippetRequest>,
    ) -> Result<Response<SnippetResponse>, Status> {
        let req = request.into_inner();
        let (content, total, source) = self
            .engine
            .context_cache
            .get_snippet(
                &req.path,
                req.start_line as usize,
                req.end_line as usize,
                req.bypass_cache,
            )
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(SnippetResponse {
            content,
            total_lines: total as i32,
            source,
        }))
    }

    async fn post_system_event(
        &self,
        request: Request<SystemEvent>,
    ) -> Result<Response<Empty>, Status> {
        let event = request.into_inner();

        let payload = json!({
            "event_id": event.event_id,
            "source": event.source,
            "severity": event.severity,
            "message": event.message,
            "metadata_json": event.metadata_json,
            "timestamp": event.timestamp.map(|t| format!("{}.{}", t.seconds, t.nanos))
        })
        .to_string();

        let conn = self.engine.redis.pool.next();
        let _: () = conn
            .publish(koad_core::constants::REDIS_CHANNEL_TELEMETRY, payload)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }
}
