use std::sync::Arc;
use crate::engine::Engine;
use crate::engine::sandbox::{Sandbox, PolicyResult};
use fred::interfaces::{PubsubInterface, HashesInterface, StreamsInterface, EventInterface};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use koad_core::intent::Intent;

pub struct CommandProcessor {
    engine: Arc<Engine>,
}

impl CommandProcessor {
    pub fn new(engine: Arc<Engine>) -> Self {
        Self { engine }
    }

    pub async fn start(&self) {
        let redis = self.engine.redis.clone();
        let mut message_stream = redis.subscriber.message_rx();

        if let Err(e) = redis.subscriber.subscribe("koad:commands").await {
            eprintln!("CommandProcessor: Failed to subscribe to Redis: {}", e);
            return;
        }

        println!("CommandProcessor: Listening for commands on 'koad:commands'...");

        while let Ok(message) = message_stream.recv().await {
            if message.channel != "koad:commands" { continue; }

            let payload_str = message.value.as_string().unwrap_or_default();
            if payload_str.is_empty() { continue; }

            // Parse the Intent Payload
            // 1. Try modern Intent enum (strongly-typed)
            // 2. Fallback to legacy JSON format
            // 3. Fallback to raw string (admin)
            let (identity, cmd_str) = match serde_json::from_str::<Intent>(&payload_str) {
                Ok(Intent::Execute(exec)) => (exec.identity, exec.command),
                Ok(other) => {
                    // Log but skip for now until Skill/Session handlers are added
                    eprintln!("CommandProcessor: Received non-execute intent: {:?}", other);
                    continue;
                }
                Err(_) => {
                    // Fallback path
                    match serde_json::from_str::<serde_json::Value>(&payload_str) {
                        Ok(json) => {
                            let id = json["identity"].as_str().unwrap_or("unknown").to_string();
                            let cmd = json["command"].as_str().unwrap_or("").to_string();
                            (id, cmd)
                        }
                        Err(_) => {
                            // Raw string fallback
                            ("admin".to_string(), payload_str)
                        }
                    }
                }
            };

            if cmd_str.is_empty() { continue; }

            let engine = self.engine.clone();
            tokio::spawn(async move {
                let task_id = Uuid::new_v4().to_string();
                Self::execute_task(engine, task_id, identity, cmd_str).await;
            });
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
                
                return; // Abort Execution
            }
            PolicyResult::Allowed => {} // Proceed
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

        // 3. Broadcast START Event to Stream
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
        // We explicitly inject a robust PATH to ensure systemd and other restricted 
        // environments can find the necessary binaries.
        let mut path = std::env::var("PATH").unwrap_or_else(|_| "/usr/local/bin:/usr/bin:/bin".to_string());
        
        // Ensure Koad-critical paths are present
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
        let output = tokio::process::Command::new("/usr/bin/bash")
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

                // 5. Update State in Redis
                let final_state = json!({
                    "task_id": task_id,
                    "status": final_status,
                    "exit_code": status_code,
                    "stdout": stdout,
                    "stderr": stderr,
                    "updated_at": final_timestamp
                });
                let _: () = engine.redis.client.hset(format!("koad:task:{}", task_id), ("state", final_state.to_string())).await.unwrap_or_default();

                // 6. Broadcast END Event to Stream
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
                    eprintln!("CommandProcessor: xadd failed: {}", e);
                }
            },
            Err(e) => {
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

