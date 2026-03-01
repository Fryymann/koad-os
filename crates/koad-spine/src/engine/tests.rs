#[cfg(test)]
mod tests {
    use crate::engine::redis::RedisClient;
    use crate::engine::persistence::PersistenceManager;
    use crate::engine::commands::CommandProcessor;
    use std::sync::Arc;
    use std::path::Path;
    use fred::interfaces::{PubsubInterface, EventInterface};
    use fred::types::Message;

    #[tokio::test]
    async fn test_redis_lifecycle() {
        let config_path = "/home/ideans/.koad-os/config/redis.conf";
        let redis = RedisClient::new(config_path).await;
        assert!(redis.is_ok(), "RedisClient should initialize and connect");
    }

    #[tokio::test]
    async fn test_persistence_initialization() {
        let config_path = "/home/ideans/.koad-os/config/redis.conf";
        let redis = Arc::new(RedisClient::new(config_path).await.unwrap());
        let db_path = "/home/ideans/.koad-os/test_persistence.db";
        
        let pm = PersistenceManager::new(redis, db_path);
        assert!(pm.is_ok(), "PersistenceManager should initialize SQLite with WAL");
        
        if Path::new(db_path).exists() {
            let _ = std::fs::remove_file(db_path);
        }
    }

    #[tokio::test]
    async fn test_command_execution() {
        let config_path = "/home/ideans/.koad-os/config/redis.conf";
        let redis = Arc::new(RedisClient::new(config_path).await.unwrap());
        
        let engine = Arc::new(crate::engine::Engine {
            redis: redis.clone(),
            persistence: Arc::new(PersistenceManager::new(redis.clone(), "/tmp/test_cmd.db").unwrap()),
            diagnostics: Arc::new(crate::engine::diagnostics::ShipDiagnostics::new(redis.clone())),
        });

        let processor = CommandProcessor::new(engine.clone());
        let mut telemetry_rx = redis.subscriber.message_rx();
        let _: () = redis.subscriber.subscribe("koad:telemetry").await.unwrap();

        let proc_handle = tokio::spawn(async move {
            processor.start().await;
        });

        // Give the processor a moment to subscribe
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        // Publish a command via engine's client
        let _: () = redis.client.publish("koad:commands", "echo 'test_output'").await.unwrap();

        // Wait for telemetry response
        let mut found = false;
        for _ in 0..20 {
            if let Ok(res) = tokio::time::timeout(std::time::Duration::from_secs(1), telemetry_rx.recv()).await {
                if let Ok(msg) = res as Result<Message, _> {
                    let payload = msg.value.as_string().unwrap();
                    println!("Test Got Telemetry: {}", payload);
                    if payload.contains("test_output") {
                        found = true;
                        break;
                    }
                }
            }
        }

        assert!(found, "CommandProcessor should execute command and publish result to telemetry");
        proc_handle.abort();
    }
}
