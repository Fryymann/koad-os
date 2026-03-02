use std::sync::Arc;
use crate::engine::Engine;
use crate::engine::sandbox::{Sandbox, PolicyResult};
use fred::interfaces::{PubsubInterface, HashesInterface, StreamsInterface, EventInterface, SetsInterface};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use koad_core::intent::{Intent, SessionAction, SystemAction, ExecuteIntent};

pub struct DirectiveRouter {
    engine: Arc<Engine>,
}

impl DirectiveRouter {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    pub async fn start(&self) {
        let redis = self.engine.redis.clone();
        let mut message_stream = redis.subscriber.message_rx();

        if let Err(e) = redis.subscriber.subscribe("koad:commands").await {
            eprintln!("DirectiveRouter: Failed to subscribe to Redis: {}", e);
            return;
        }

        println!("DirectiveRouter: Listening for intents on 'koad:commands'...");

        while let Ok(message) = message_stream.recv().await {
            if message.channel != "koad:commands" { continue; }

            let payload_str = message.value.as_string().unwrap_or_default();
            if payload_str.is_empty() { continue; }

            let intent = match serde_json::from_str::<Intent>(&payload_str) {
                Ok(i) => {
                    println!("DirectiveRouter: Parsed Intent: {:?}", i);
                    i
                },
                Err(e) => {
                    println!("DirectiveRouter: JSON Parse Error: {}. Payload: {}", e, payload_str);
                    // Fallback to legacy/raw string as Execute intent
                    match serde_json::from_str::<serde_json::Value>(&payload_str) {
                        Ok(json) => {
                             println!("DirectiveRouter: Falling back to legacy JSON Execute");
                             Intent::Execute(ExecuteIntent {
                                 identity: json["identity"].as_str().unwrap_or("unknown").to_string(),
                                 command: json["command"].as_str().unwrap_or("").to_string(),
                                 args: vec![],
                                 working_dir: None,
                                 env_vars: std::collections::HashMap::new(),
                             })
                        }
                        Err(_) => {
                            println!("DirectiveRouter: Falling back to raw string Execute");
                            Intent::Execute(ExecuteIntent {
                                identity: "admin".to_string(),
                                command: payload_str,
                                args: vec![],
                                working_dir: None,
                                env_vars: std::collections::HashMap::new(),
                            })
                        }
                    }
                }
            };

            let engine = self.engine.clone();
            tokio::spawn(async move {
                Self::route_intent(engine, intent).await;
            });
        }
    }

    async fn route_intent(engine: Arc<Engine>, intent: Intent) {
        match intent {
            Intent::Execute(exec) => {
                let task_id = Uuid::new_v4().to_string();
                Self::execute_task(engine, task_id, exec.identity, exec.command).await;
            }
            Intent::Governance(gov) => {
                if let Err(e) = engine.kcm.handle_intent(gov).await {
                    eprintln!("DirectiveRouter: Governance Action Failed: {}", e);
                }
            }
            Intent::Session(session) => {
                match session.action {
                    SessionAction::Heartbeat => {
                        let _ = engine.asm.heartbeat(&session.session_id).await;
                    }
                    SessionAction::Stop => {
                        println!("DirectiveRouter: Stopping session {}", session.session_id);
                    }
                    _ => {
                        eprintln!("DirectiveRouter: Unhandled Session Action: {:?}", session.action);
                    }
                }
            }
            Intent::System(sys) => {
                match sys.action {
                    SystemAction::Reboot => {
                        println!("DirectiveRouter: System REBOOT initiated.");
                        std::process::exit(0);
                    }
                    _ => {
                        eprintln!("DirectiveRouter: Unhandled System Action: {:?}", sys.action);
                    }
                }
            }
            Intent::Skill(skill) => {
                eprintln!("DirectiveRouter: Skill intent routing not yet fully implemented: {:?}", skill);
            }
        }
    }

    async fn execute_task(engine: Arc<Engine>, task_id: String, identity: String, cmd_str: String) {
        let timestamp = Utc::now().timestamp();
        
        // 1. Sandbox Policy Check
        match Sandbox::evaluate(&identity, &cmd_str) {
            PolicyResult::Denied(reason) => {
                let error_state = json!({
                    "task_id": task_id,
                    "status": "FAILED",
                    "error": format!("Policy Violation ({}): {}", identity, reason),
                    "updated_at": timestamp
                });
                let _: () = engine.redis.client.hset(format!("koad:task:{}", task_id), ("state", error_state.to_string())).await.unwrap_or_default();
                
                let _: () = engine.redis.client.xadd(
                    "koad:events:stream", false, None, "*", 
                    vec![
                        ("source", "engine:sandbox"),
                        ("severity", "ERROR"),
                        ("message", "TASK_REJECTED"),
                        ("metadata", &error_state.to_string()),
                        ("timestamp", &timestamp.to_string())
                    ]
                ).await.unwrap_or_default();
                
                return;
            }
            PolicyResult::Allowed => {}
        }

        // 2. Initial State in Redis
        let initial_state = json!({
            "task_id": task_id,
            "command": cmd_str,
            "identity": identity,
            "status": "RUNNING",
            "updated_at": timestamp
        });
        let _: () = engine.redis.client.hset(format!("koad:task:{}", task_id), ("state", initial_state.to_string())).await.unwrap_or_default();
        let _: () = engine.redis.client.sadd("koad:active_tasks", task_id.clone()).await.unwrap_or_default();

        // 3. Broadcast START Event
        let _: () = engine.redis.client.xadd(
            "koad:events:stream", 
            false, 
            None, 
            "*", 
            vec![
                ("source", "engine:scheduler"),
                ("severity", "INFO"),
                ("message", "TASK_LIFECYCLE"),
                ("metadata", &initial_state.to_string()),
                ("timestamp", &timestamp.to_string())
            ]
        ).await.unwrap_or_default();

        // 4. Construct Environment
        let mut path = std::env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".to_string());
        let koad_paths = vec![
            "/home/ideans/.cargo/bin",
            "/home/ideans/.nvm/versions/node/v22.21.1/bin",
            "/home/ideans/.koad-os/bin"
        ];
        for p in koad_paths {
            if !path.contains(p) {
                path = format!("{}:{}", p, path);
            }
        }

        // 5. Execute Command
        let output = tokio::process::Command::new("bash")
            .arg("-c")
            .arg(&cmd_str)
            .env("PATH", path)
            .output()
            .await;

        let final_timestamp = Utc::now().timestamp();
        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let status_code = out.status.code().unwrap_or(-1);
                let final_status = if status_code == 0 { "SUCCESS" } else { "FAILED" };

                let final_state = json!({
                    "task_id": task_id,
                    "command": cmd_str,
                    "status": final_status,
                    "exit_code": status_code,
                    "stdout": stdout,
                    "stderr": stderr,
                    "updated_at": final_timestamp
                });
                let _: () = engine.redis.client.hset(format!("koad:task:{}", task_id), ("state", final_state.to_string())).await.unwrap_or_default();
                let _: () = engine.redis.client.srem("koad:active_tasks", task_id.clone()).await.unwrap_or_default();

                if let Err(e) = engine.redis.client.xadd::<String, _, _, _, _>(
                    "koad:events:stream", 
                    false, 
                    None, 
                    "*", 
                    vec![
                        ("source", "engine:scheduler"),
                        ("severity", if status_code == 0 { "INFO" } else { "ERROR" }),
                        ("message", "TASK_LIFECYCLE"),
                        ("metadata", &final_state.to_string()),
                        ("timestamp", &final_timestamp.to_string())
                    ]
                ).await {
                    eprintln!("DirectiveRouter: xadd failed: {}", e);
                }
            },
            Err(e) => {
                let _: () = engine.redis.client.srem("koad:active_tasks", task_id.clone()).await.unwrap_or_default();
                let _: () = engine.redis.client.xadd(
                    "koad:events:stream", 
                    false, 
                    None, 
                    "*", 
                    vec![
                        ("source", "engine:scheduler"),
                        ("severity", "ERROR"),
                        ("message", "TASK_SPAWN_FAILURE"),
                        ("metadata", &json!({ "task_id": task_id, "error": e.to_string() }).to_string()),
                        ("timestamp", &final_timestamp.to_string())
                    ]
                ).await.unwrap_or_default();
            }
        }
    }
}
