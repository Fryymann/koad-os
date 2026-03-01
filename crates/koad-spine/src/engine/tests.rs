#[cfg(test)]
mod tests {
    use crate::engine::redis::RedisClient;
    use crate::engine::commands::CommandProcessor;
    use crate::engine::Engine;
    use std::sync::Arc;
    use fred::interfaces::{PubsubInterface, EventInterface, StreamsInterface, ClientLike, KeysInterface};

    #[tokio::test]
    async fn test_redis_lifecycle() {
        let home = std::env::var("KOAD_HOME").unwrap_or_else(|_| "/home/ideans/.koad-os".to_string());
        let redis = RedisClient::new(&home).await.unwrap();
        let _: String = redis.client.ping().await.unwrap();
    }

    #[tokio::test]
    async fn test_command_execution() {
        let home = std::env::var("KOAD_HOME").unwrap_or_else(|_| "/home/ideans/.koad-os".to_string());
        let db_path = format!("{}/test_exec.db", home);
        let engine = Arc::new(Engine::new(&home, &db_path).await.unwrap());
        
        let proc = CommandProcessor::new(engine.clone());
        let proc_handle = tokio::spawn(async move {
            proc.start().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let payload = serde_json::json!({
            "identity": "admin",
            "command": "echo 'hello_koad'"
        });
        
        let _: () = engine.redis.client.publish("koad:commands", payload.to_string()).await.unwrap();

        let mut found = false;
        // Wait for task completion
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        
        let events: Vec<(String, std::collections::HashMap<String, String>)> = engine.redis.client.xrange(
            "koad:events:stream", "-", "+", None
        ).await.unwrap_or_default();

        for msg in events {
            if let Some(msg_type) = msg.1.get("message") {
                if msg_type == "TASK_LIFECYCLE" {
                    let meta = msg.1.get("metadata").unwrap();
                    if meta.contains("hello_koad") {
                        found = true;
                        break;
                    }
                }
            }
        }

        assert!(found, "CommandProcessor should execute command and log to event stream");
        proc_handle.abort();
        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_path_integrity() {
        let home = std::env::var("KOAD_HOME").unwrap_or_else(|_| "/home/ideans/.koad-os".to_string());
        let db_path = format!("{}/test_path.db", home);
        let engine = Arc::new(Engine::new(&home, &db_path).await.unwrap());
        
        let proc = CommandProcessor::new(engine.clone());
        let proc_handle = tokio::spawn(async move {
            proc.start().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Test finding 'cargo' which is in a non-standard path
        let payload = serde_json::json!({
            "identity": "admin",
            "command": "which cargo"
        });
        
        let _: () = engine.redis.client.publish("koad:commands", payload.to_string()).await.unwrap();

        let mut output_ok = false;
        for _ in 0..5 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let events: Vec<(String, std::collections::HashMap<String, String>)> = engine.redis.client.xrange(
                "koad:events:stream", "-", "+", None
            ).await.unwrap_or_default();

            for msg in events {
                if msg.1.get("message").map(|s| s == "TASK_LIFECYCLE").unwrap_or(false) {
                    let meta = msg.1.get("metadata").unwrap();
                    if meta.contains("cargo") && meta.contains("SUCCESS") {
                        output_ok = true;
                        break;
                    }
                }
            }
            if output_ok { break; }
        }

        assert!(output_ok, "CommandProcessor should be able to find and execute 'cargo'");
        proc_handle.abort();
        let _ = std::fs::remove_file(db_path);
    }
}
