#[cfg(test)]
mod tests {
    use crate::engine::redis::RedisClient;
    use crate::engine::router::DirectiveRouter;
    use crate::engine::Engine;
    use fred::interfaces::{
        ClientLike, EventInterface, KeysInterface, PubsubInterface, StreamsInterface,
    };
    use std::sync::Arc;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_redis_lifecycle() {
        let tdir = tempdir().unwrap();
        let home = tdir.path().to_str().unwrap();
        let redis = RedisClient::new(home).await.unwrap();
        let _: String = redis.client.ping().await.unwrap();
    }

    use koad_core::intent::{ExecuteIntent, Intent};

    #[tokio::test]
    async fn test_command_execution() {
        let tdir = tempdir().unwrap();
        let home = tdir.path().to_str().unwrap();
        let db_path = format!("{}/test_exec.db", home);
        let engine = Arc::new(Engine::new(home, &db_path).await.unwrap());

        let proc = DirectiveRouter::new(engine.clone());
        let _proc_handle = tokio::spawn(async move {
            proc.start().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let intent = Intent::Execute(ExecuteIntent {
            identity: "admin".to_string(),
            command: "echo 'hello_koad'".to_string(),
            args: vec![],
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
        });

        let payload = serde_json::to_string(&intent).unwrap();
        println!("Test: Publishing intent to koad:commands: {}", payload);
        let _: () = engine
            .redis
            .client
            .publish("koad:commands", payload)
            .await
            .unwrap();

        let mut found = false;
        // Wait for task completion
        for i in 0..10 {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            println!("Test: Checking events (attempt {})...", i + 1);

            let events: Vec<(String, std::collections::HashMap<String, String>)> = engine
                .redis
                .client
                .xrange("koad:events:stream", "-", "+", None)
                .await
                .unwrap_or_default();

            for msg in events {
                if let Some(msg_type) = msg.1.get("message") {
                    if msg_type == "TASK_LIFECYCLE" {
                        let meta = msg.1.get("metadata").unwrap();
                        if meta.contains("hello_koad") && meta.contains("SUCCESS") {
                            found = true;
                            break;
                        }
                    }
                }
            }
            if found {
                break;
            }
        }

        assert!(
            found,
            "DirectiveRouter should execute command and log to event stream"
        );
    }

    #[tokio::test]
    async fn test_path_integrity() {
        let tdir = tempdir().unwrap();
        let home = tdir.path().to_str().unwrap();
        let db_path = format!("{}/test_path.db", home);
        let engine = Arc::new(Engine::new(home, &db_path).await.unwrap());

        let proc = DirectiveRouter::new(engine.clone());
        let _proc_handle = tokio::spawn(async move {
            proc.start().await;
        });

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        // Test finding 'cargo' which is in a non-standard path
        let intent = Intent::Execute(ExecuteIntent {
            identity: "admin".to_string(),
            command: "which cargo".to_string(),
            args: vec![],
            working_dir: None,
            env_vars: std::collections::HashMap::new(),
        });

        let payload = serde_json::to_string(&intent).unwrap();
        println!("Test: Publishing path check intent: {}", payload);
        let _: () = engine
            .redis
            .client
            .publish("koad:commands", payload)
            .await
            .unwrap();

        let mut output_ok = false;
        for i in 0..10 {
            tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
            println!("Test: Checking path events (attempt {})...", i + 1);
            let events: Vec<(String, std::collections::HashMap<String, String>)> = engine
                .redis
                .client
                .xrange("koad:events:stream", "-", "+", None)
                .await
                .unwrap_or_default();

            for msg in events {
                if msg
                    .1
                    .get("message")
                    .map(|s| s == "TASK_LIFECYCLE")
                    .unwrap_or(false)
                {
                    let meta = msg.1.get("metadata").unwrap();
                    if meta.contains("cargo") && meta.contains("SUCCESS") {
                        output_ok = true;
                        break;
                    }
                }
            }
            if output_ok {
                break;
            }
        }

        assert!(
            output_ok,
            "DirectiveRouter should be able to find and execute 'cargo'"
        );
    }
}
