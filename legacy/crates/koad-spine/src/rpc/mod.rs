use crate::engine::Engine;
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

    fn get_session_id<T>(&self, request: &Request<T>) -> Result<String, Status> {
        request
            .metadata()
            .get("x-session-id")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .ok_or_else(|| Status::unauthenticated("No session ID in request metadata"))
    }

    async fn verify_session<T>(
        &self,
        request: &Request<T>,
    ) -> Result<koad_core::session::AgentSession, Status> {
        // 1. Check for System Key (Internal Service Bypass)
        if let Some(sys_key) = request.metadata().get("x-system-key").and_then(|v| v.to_str().ok()) {
            if sys_key == "citadel-core" {
                return Ok(koad_core::session::AgentSession::new(
                    "system".to_string(),
                    koad_core::identity::Identity {
                        name: "system".to_string(),
                        rank: koad_core::identity::Rank::Admiral,
                        permissions: vec!["all".to_string()],
                        access_keys: vec![],
                        tier: 1,
                    },
                    koad_core::types::EnvironmentType::Wsl,
                    koad_core::session::ProjectContext {
                        project_name: "system".to_string(),
                        root_path: "".to_string(),
                        allowed_paths: vec!["all".to_string()],
                        stack: vec![],
                    },
                    "kernel".to_string(),
                ));
            }
        }

        // 2. Standard Session Validation
        let sid = self.get_session_id(request)?;
        let session = self.engine
            .asm
            .get_session(&sid)
            .await
            .map_err(|e| Status::internal(format!("ASM Error: {}", e)))?;
        
        if let Some(s) = session {
            Ok(s)
        } else {
            error!("VerifySession Failed: Session ID '{}' not found in Spine ASM cache.", sid);
            Err(Status::unauthenticated("Invalid or expired session ID"))
        }
    }
}

#[tonic::async_trait]
impl SpineService for KoadSpine {
    // --- Core Execution ---

    async fn execute(
        &self,
        _request: Request<ExecuteRequest>,
    ) -> Result<Response<ExecuteResponse>, Status> {
        Err(Status::unimplemented("Execute RPC (Synchronous) is not yet active. Use DispatchTask for async execution via the neural bus."))
    }

    async fn heartbeat(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        if let Some(session_id) = request
            .metadata()
            .get("x-session-id")
            .and_then(|v| v.to_str().ok())
        {
            if let Ok(Some(session)) = self.engine.asm.get_session(session_id).await {
                let _ = self.engine.identity.heartbeat(&session.identity.name, session_id).await;
                let _ = self.engine.asm.heartbeat(session_id).await;
            }
        }
        Ok(Response::new(Empty {}))
    }

    // --- Task Management ---

    async fn dispatch_task(
        &self,
        request: Request<DispatchTaskRequest>,
    ) -> Result<Response<DispatchTaskResponse>, Status> {
        let session = self.verify_session(&request).await?;
        let req = request.into_inner();
        let cmd_str = req.command;

        let intent = Intent::Execute(ExecuteIntent {
            session_id: session.session_id.clone(),
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
        Err(Status::unimplemented("StreamTaskStatus is not yet functional in the v4 Spine. Monitor koad:events:stream for lifecycle updates."))
    }

    // --- System Telemetry & Monitoring ---

    type StreamSystemEventsStream = Pin<Box<dyn Stream<Item = Result<SystemEvent, Status>> + Send>>;

    async fn stream_system_events(
        &self,
        request: Request<StreamSystemEventsRequest>,
    ) -> Result<Response<Self::StreamSystemEventsStream>, Status> {
        let _session = self.verify_session(&request).await?;
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
                    "koad:telemetry:manifest",
                    "koad:telemetry:services",
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
        request: Request<GetSystemStateRequest>,
    ) -> Result<Response<GetSystemStateResponse>, Status> {
        let _session = self.verify_session(&request).await?;
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

        let version = self.engine.config.system.as_ref()
            .map(|s| s.version.clone())
            .unwrap_or_else(|| "4.0.0".to_string());

        Ok(Response::new(GetSystemStateResponse {
            identity_json: sessions_json,
            active_tasks: 0,
            version,
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
        let _session = self.verify_session(&request).await?;
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
        Err(Status::unimplemented("RegisterComponent is deprecated. Use InitializeSession for agentic registration."))
    }

    async fn initialize_session(
        &self,
        request: Request<InitializeSessionRequest>,
    ) -> Result<Response<SessionPackage>, Status> {
        let req = request.into_inner();
        let session_id = if !req.session_id.is_empty() {
            req.session_id.clone()
        } else {
            uuid::Uuid::new_v4().to_string()
        };
        let body_id = if req.body_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.body_id.clone()
        };

        let (rank, permissions, access_keys, bio) = {
            let config = self.engine.identity.get_config();
            let mut bio = None;
            let mut role_str = req.agent_role.clone();
            let mut rank_val = None;

            // 1. Check SQLite for Authoritative Identity
            if let Ok(Some((_name, role, b, tier))) = self.engine.storage.get_identity(&req.agent_name).await {
                bio = Some(b);
                role_str = role;
                rank_val = Some(match tier {
                    0 => Rank::Admiral,
                    1 => Rank::Captain,
                    2 => Rank::Officer,
                    _ => Rank::Crew,
                });
            }

            let identity_config = config.identities.get(&req.agent_name)
                .or_else(|| config.identities.get(&req.agent_name.to_lowercase()));

            if let Some(id_config) = identity_config {
                let r = rank_val.unwrap_or_else(|| match id_config.rank.to_lowercase().as_str() {
                    "admiral" | "dood" => Rank::Admiral,
                    "captain" | "admin" => Rank::Captain,
                    "officer" | "pm" => Rank::Officer,
                    _ => Rank::Crew,
                });
                let keys = id_config
                    .preferences
                    .as_ref()
                    .map(|p| p.access_keys.clone())
                    .unwrap_or_default();
                (r, vec!["all".to_string()], keys, bio)
            } else {
                let r = rank_val.unwrap_or_else(|| {
                    if req.agent_name.to_lowercase() == "dood"
                        || req.agent_name.to_lowercase() == "ian"
                    {
                        Rank::Admiral
                    } else {
                        match role_str.to_lowercase().as_str() {
                            "admiral" | "dood" => Rank::Admiral,
                            "captain" | "admin" => Rank::Captain,
                            "officer" | "pm" => Rank::Officer,
                            _ => Rank::Crew,
                        }
                    }
                });
                (r, vec!["all".to_string()], vec![], bio)
            }
        };

        let is_sovereign = rank == Rank::Admiral || rank == Rank::Captain;

        let lease = self
            .engine
            .identity
            .acquire_lease(
                &req.agent_name,
                &session_id,
                &req.driver_id,
                req.model_tier,
                &body_id,
                req.force,
                rank.clone(),
                is_sovereign,
            )
            .await
            .map_err(|e| Status::permission_denied(e.to_string()))?;

        let identity = koad_core::identity::Identity {
            name: req.agent_name.clone(),
            rank,
            permissions,
            access_keys,
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
            body_id.clone(),
        );

        if let Some(b) = bio {
            session.metadata.insert("bio".to_string(), b);
        } else if let Ok(Some(b)) = self.engine.storage.get_identity_bio(&req.agent_name).await {
            session.metadata.insert("bio".to_string(), b);
        }
        session
            .metadata
            .insert("model_name".to_string(), req.model_name.clone());
        session
            .metadata
            .insert("driver_id".to_string(), req.driver_id.clone());
        session
            .metadata
            .insert("body_id".to_string(), body_id.clone());

        // 0. Body Enforcement: Pre-empt previous sessions for this agent on the same driver/env
        let _ = self.engine.asm.prune_body_ghosts(&req.agent_name, &req.driver_id, environment, &session_id).await;

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
                    file_path: v["file_path"].as_str().unwrap_or_default().to_string(),
                    ttl_seconds: v["ttl_seconds"].as_i64().unwrap_or(0) as i32,
                    created_at: None,
                });
            }
        }

        // Fetch Pending Signals
        let core_signals: Vec<koad_core::signal::GhostSignal> = self.engine.signal.get_signals(&req.agent_name).await.unwrap_or_default();
        let pending_signals = core_signals.into_iter()
            .filter(|s| s.status == koad_core::signal::SignalStatus::Pending)
            .map(|s| {
                GhostSignal {
                    id: s.id,
                    source_agent: s.source_agent,
                    target_agent: s.target_agent,
                    message: s.message,
                    priority: match s.priority {
                        koad_core::signal::SignalPriority::Low => SignalPriority::Low as i32,
                        koad_core::signal::SignalPriority::Standard => SignalPriority::Standard as i32,
                        koad_core::signal::SignalPriority::High => SignalPriority::High as i32,
                        koad_core::signal::SignalPriority::Critical => SignalPriority::Critical as i32,
                    },
                    timestamp: Some(prost_types::Timestamp {
                        seconds: s.timestamp.timestamp(),
                        nanos: s.timestamp.timestamp_subsec_nanos() as i32,
                    }),
                    metadata: s.metadata,
                    status: SignalStatus::Pending as i32,
                }
            }).collect();

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
                pending_signals,
            }),
            lease: Some(lease),
        }))
    }

    async fn hydrate_context(
        &self,
        request: Request<HydrationRequest>,
    ) -> Result<Response<HydrationResponse>, Status> {
        let session = self.verify_session(&request).await?;
        let req = request.into_inner();
        let chunk = req
            .chunk
            .ok_or_else(|| Status::invalid_argument("Missing chunk"))?;

        match self
            .engine
            .hydration
            .hydrate(
                &session.session_id,
                &chunk.content,
                if chunk.file_path.is_empty() {
                    None
                } else {
                    Some(chunk.file_path)
                },
                chunk.ttl_seconds,
                Some(&session),
            )
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

    async fn flush_context(
        &self,
        request: Request<FlushContextRequest>,
    ) -> Result<Response<Empty>, Status> {
        let session = self.verify_session(&request).await?;
        let req = request.into_inner();
        let target_sid = req.session_id;

        // Authorization: Only the session itself or an Admiral/Captain can flush
        if target_sid != session.session_id
            && session.identity.rank != Rank::Admiral
            && session.identity.rank != Rank::Captain
        {
            return Err(Status::permission_denied(
                "Unauthorized: You cannot flush another agent's context.",
            ));
        }

        self.engine
            .hydration
            .flush_context(&target_sid)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn commit_knowledge(
        &self,
        request: Request<CommitKnowledgeRequest>,
    ) -> Result<Response<Empty>, Status> {
        let session = self.verify_session(&request).await?;
        let req = request.into_inner();

        // 2. Commit to durable memory bank
        let tags = if req.tags.is_empty() {
            None
        } else {
            Some(req.tags)
        };

        self.engine
            .storage
            .save_knowledge(&req.category, &req.content, tags, &session.identity.name)
            .await
            .map_err(|e| Status::internal(format!("Storage Error: {}", e)))?;

        info!(
            "CommitKnowledge: Successfully recorded '{}' for agent '{}'",
            req.category, session.identity.name
        );

        Ok(Response::new(Empty {}))
    }

    async fn terminate_session(
        &self,
        request: Request<TerminateSessionRequest>,
    ) -> Result<Response<Empty>, Status> {
        let caller_session = self.verify_session(&request).await?;
        let req = request.into_inner();
        let target_sid = req.session_id;

        // Authorization: Only the session itself or an Admiral/Captain can terminate
        if target_sid != caller_session.session_id
            && caller_session.identity.rank != Rank::Admiral
            && caller_session.identity.rank != Rank::Captain
        {
            return Err(Status::permission_denied(
                "Unauthorized: You cannot terminate another agent's session.",
            ));
        }

        info!("Kernel: Terminating session {}", target_sid);

        // 1. Resolve agent from session
        if let Some(mut session) = self.engine.asm.get_session(&target_sid).await
            .map_err(|e| Status::internal(e.to_string()))? 
        {
            let agent_name = session.identity.name.clone();
            
            // 2. Mark as dark
            session.status = "dark".to_string();
            let payload = serde_json::to_value(&session).map_err(|e| Status::internal(e.to_string()))?;
            let session_key = format!("koad:session:{}", target_sid);

            let _: () = self.engine.redis.pool.next()
                .hset("koad:state", (&session_key, payload.to_string()))
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            // 3. Release Lease
            let _ = self.engine.identity.release_lease(&agent_name, &target_sid).await;

            // 4. Broadcast Update
            let msg = json!({
                "type": "SESSION_TERMINATED",
                "session_id": target_sid
            });
            let _: () = self.engine.redis.pool.next()
                .publish("koad:sessions", msg.to_string())
                .await
                .map_err(|e| Status::internal(e.to_string()))?;
            
            // 5. Explicitly update ASM local cache
            let _ = self.engine.asm.remove_session(&target_sid).await;
        }

        Ok(Response::new(Empty {}))
    }

    async fn drain_all(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let session = self.verify_session(&request).await?;
        if session.identity.rank != Rank::Admiral && session.identity.rank != Rank::Captain {
            return Err(Status::permission_denied(
                "Unauthorized: Manual drain requires Sovereign rank.",
            ));
        }

        info!(
            "Kernel: Triggering full state drain to durable memory (Initiated by {})...",
            session.identity.name
        );
        self.engine
            .storage
            .drain_all()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(Empty {}))
    }

    async fn trigger_backup(
        &self,
        request: Request<TriggerBackupRequest>,
    ) -> Result<Response<TriggerBackupResponse>, Status> {
        let session = self.verify_session(&request).await?;
        if session.identity.rank != Rank::Admiral && session.identity.rank != Rank::Captain {
            return Err(Status::permission_denied(
                "Unauthorized: Manual backup requires Sovereign rank.",
            ));
        }

        info!("Kernel: Manual backup triggered by {}", session.identity.name);
        
        match self.engine.backup.perform_full_backup().await {
            Ok(_) => Ok(Response::new(TriggerBackupResponse {
                success: true,
                message: "Backup completed successfully.".to_string(),
                backup_id: chrono::Local::now().format("%Y%m%d-%H%M%S").to_string(),
            })),
            Err(e) => Ok(Response::new(TriggerBackupResponse {
                success: false,
                message: format!("Backup failed: {}", e),
                backup_id: "".to_string(),
            })),
        }
    }

    async fn reconnect_session(
        &self,
        request: Request<ReconnectSessionRequest>,
    ) -> Result<Response<SessionPackage>, Status> {
        let req = request.into_inner();
        let body_id = if req.body_id.is_empty() { None } else { Some(req.body_id.as_str()) };

        info!("Kernel: Reconnection requested for agent '{}' (SID: {:?}, Body: {:?})", req.agent_name, req.session_id, body_id);

        // 1. Find a recoverable session
        let session = if !req.session_id.is_empty() {
            // Level 0: Direct SID lookup
            self.engine.asm.get_session(&req.session_id).await
                .map_err(|e| Status::internal(format!("Session Lookup Failed: {}", e)))?
        } else {
            // Level 1/2: Discovery via name and body
            self.engine.asm.find_recoverable_session(&req.agent_name, body_id).await
                .map_err(|e| Status::internal(format!("Reconnection Discovery Failed: {}", e)))?
        };

        if let Some(mut sess) = session {
            info!("Kernel: Recovered session {} for agent {}", sess.session_id, req.agent_name);
            
            // 2. Refresh status to active
            sess.status = "active".to_string();
            let payload = serde_json::to_value(&sess).map_err(|e| Status::internal(e.to_string()))?;
            let session_key = format!("koad:session:{}", sess.session_id);

            let _: () = self.engine.redis.pool.next()
                .hset("koad:state", (&session_key, payload.to_string()))
                .await
                .map_err(|e| Status::internal(e.to_string()))?;

            // 3. Update local cache
            self.engine.asm.create_session(sess.clone()).await
                .map_err(|e| Status::internal(e.to_string()))?;

            // 4. Hydrate Intelligence
            let hydration = self.engine.asm.hydrate_session(&sess.session_id).await
                .map_err(|e| Status::internal(e.to_string()))?;

            let context_key = format!("koad:session:{}:hot_context", sess.session_id);
            let hot_context_raw: std::collections::HashMap<String, String> = self.engine.redis.pool.next()
                .hgetall(&context_key).await.unwrap_or_default();

            let mut hot_context = Vec::new();
            for val in hot_context_raw.values() {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(val) {
                    hot_context.push(HotContextChunk {
                        chunk_id: v["chunk_id"].as_str().unwrap_or_default().to_string(),
                        content: v["content"].as_str().unwrap_or_default().to_string(),
                        file_path: v["file_path"].as_str().unwrap_or_default().to_string(),
                        ttl_seconds: v["ttl_seconds"].as_i64().unwrap_or(0) as i32,
                        created_at: None,
                    });
                }
            }

            Ok(Response::new(SessionPackage {
                session_id: sess.session_id.clone(),
                identity_json: serde_json::to_string(&sess.identity).unwrap(),
                project_context_json: serde_json::to_string(&sess.context).unwrap(),
                intelligence: Some(IntelligencePackage {
                    mission_briefing: hydration["mission_briefing"].as_str().unwrap_or_default().to_string(),
                    active_tasks: vec![],
                    recent_events: vec![],
                    metadata: std::collections::HashMap::new(),
                    hot_context,
                    pending_signals: vec![], // Re-fetch logic omitted for brevity in reconnect
                }),
                lease: None, // Will be renewed by CLI heartbeat
            }))
        } else {
            Err(Status::not_found(format!("No recoverable session found for agent '{}'. Please perform a fresh boot.", req.agent_name)))
        }
    }

    async fn get_file_snippet(
        &self,
        request: Request<GetFileSnippetRequest>,
    ) -> Result<Response<SnippetResponse>, Status> {
        let session = self.verify_session(&request).await?;
        let req = request.into_inner();
        let (content, total, source) = self
            .engine
            .context_cache
            .get_snippet(
                &req.path,
                req.start_line as usize,
                req.end_line as usize,
                req.bypass_cache,
                Some(&session),
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
        let _session = self.verify_session(&request).await?;
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

    // --- Agent-to-Agent Signals (A2A-S) ---

    async fn send_signal(
        &self,
        request: Request<SendSignalRequest>,
    ) -> Result<Response<Empty>, Status> {
        let session = self.verify_session(&request).await?;
        let req = request.into_inner();

        // 2. Map priority
        let priority = match SignalPriority::try_from(req.priority).unwrap_or(SignalPriority::Standard) {
            SignalPriority::Low => koad_core::signal::SignalPriority::Low,
            SignalPriority::Standard => koad_core::signal::SignalPriority::Standard,
            SignalPriority::High => koad_core::signal::SignalPriority::High,
            SignalPriority::Critical => koad_core::signal::SignalPriority::Critical,
        };

        // 3. Send via engine
        let _ = self.engine.signal.send_signal(
            session.identity.name,
            req.target_agent,
            req.message,
            priority,
            req.metadata,
        ).await
        .map_err(|e: anyhow::Error| Status::internal(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }

    async fn get_signals(
        &self,
        request: Request<GetSignalsRequest>,
    ) -> Result<Response<GetSignalsResponse>, Status> {
        let session = self.verify_session(&request).await?;
        let core_signals: Vec<koad_core::signal::GhostSignal> = self
            .engine
            .signal
            .get_signals(&session.identity.name)
            .await
            .map_err(|e: anyhow::Error| Status::internal(e.to_string()))?;

        let signals = core_signals.into_iter().map(|s| {
            GhostSignal {
                id: s.id,
                source_agent: s.source_agent,
                target_agent: s.target_agent,
                message: s.message,
                priority: match s.priority {
                    koad_core::signal::SignalPriority::Low => SignalPriority::Low as i32,
                    koad_core::signal::SignalPriority::Standard => SignalPriority::Standard as i32,
                    koad_core::signal::SignalPriority::High => SignalPriority::High as i32,
                    koad_core::signal::SignalPriority::Critical => SignalPriority::Critical as i32,
                },
                timestamp: Some(prost_types::Timestamp {
                    seconds: s.timestamp.timestamp(),
                    nanos: s.timestamp.timestamp_subsec_nanos() as i32,
                }),
                metadata: s.metadata,
                status: match s.status {
                    koad_core::signal::SignalStatus::Pending => SignalStatus::Pending as i32,
                    koad_core::signal::SignalStatus::Read => SignalStatus::Read as i32,
                    koad_core::signal::SignalStatus::Archived => SignalStatus::Archived as i32,
                },
            }
        }).collect();

        Ok(Response::new(GetSignalsResponse { signals }))
    }

    async fn update_signal_status(
        &self,
        request: Request<UpdateSignalStatusRequest>,
    ) -> Result<Response<Empty>, Status> {
        let session = self.verify_session(&request).await?;
        let req = request.into_inner();

        let status = match SignalStatus::try_from(req.status).unwrap_or(SignalStatus::Pending) {
            SignalStatus::Pending => koad_core::signal::SignalStatus::Pending,
            SignalStatus::Read => koad_core::signal::SignalStatus::Read,
            SignalStatus::Archived => koad_core::signal::SignalStatus::Archived,
        };

        let _ = self.engine
            .signal
            .update_signal_status(&session.identity.name, &req.signal_id, status)
            .await
            .map_err(|e: anyhow::Error| Status::internal(e.to_string()))?;

        Ok(Response::new(Empty {}))
    }
}
