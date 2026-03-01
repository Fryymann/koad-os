use std::process::Command;
use std::sync::Arc;
use crate::engine::Engine;
use fred::interfaces::{PubsubInterface, EventInterface};
use serde_json::json;

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
            let cmd_str = message.value.as_string().unwrap_or_default();
            if cmd_str.is_empty() { continue; }

            let engine = self.engine.clone();
            tokio::spawn(async move {
                Self::execute_command(engine, cmd_str).await;
            });
        }
    }

    async fn execute_command(engine: Arc<Engine>, cmd_str: String) {
        println!("CommandProcessor: Executing [{}]", cmd_str);
        
        // Log start to telemetry
        let _: Result<(), _> = engine.redis.client.publish("koad:telemetry", json!({
            "source": "PROCESSOR",
            "message": format!("Executing: {}", cmd_str),
            "timestamp": chrono::Utc::now().timestamp(),
            "level": 0
        }).to_string()).await;

        // Execute via shell
        let output = Command::new("bash")
            .arg("-c")
            .arg(&cmd_str)
            .env("PATH", "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin:/home/ideans/.cargo/bin:/home/ideans/.nvm/versions/node/v22.21.1/bin")
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
                let status = out.status.code().unwrap_or(-1);

                let msg = if status == 0 {
                    if stdout.is_empty() { "Command completed successfully.".to_string() } else { stdout }
                } else {
                    format!("Error ({}): {}", status, stderr)
                };

                let _: Result<(), _> = engine.redis.client.publish("koad:telemetry", json!({
                    "source": "PROCESSOR",
                    "message": msg,
                    "timestamp": chrono::Utc::now().timestamp(),
                    "level": if status == 0 { 0 } else { 2 }
                }).to_string()).await;
            },
            Err(e) => {
                let _: Result<(), _> = engine.redis.client.publish("koad:telemetry", json!({
                    "source": "PROCESSOR",
                    "message": format!("Spawn Error: {}", e),
                    "timestamp": chrono::Utc::now().timestamp(),
                    "level": 2
                }).to_string()).await;
            }
        }
    }
}
