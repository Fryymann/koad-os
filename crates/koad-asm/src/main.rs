use anyhow::{Context, Result};
use chrono::Utc;
use fred::interfaces::{HashesInterface, PubsubInterface};
use fred::prelude::*;
use koad_core::config::KoadConfig;
use koad_core::logging::init_logging;
use koad_core::session::AgentSession;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info};

struct SessionManager {
    pool: RedisPool,
    subscriber: fred::clients::RedisClient,
}

impl SessionManager {
    async fn new(config: KoadConfig) -> Result<Self> {
        let redis_url = format!("redis+unix://{}", config.redis_socket.display());
        info!("ASM: Connecting to Redis at {}", redis_url);
        let redis_config = RedisConfig::from_url(&redis_url)?;

        let pool = RedisPool::new(redis_config.clone(), None, None, None, 6)?;
        pool.connect();
        pool.wait_for_connect().await?;

        let subscriber = fred::clients::RedisClient::new(redis_config, None, None, None);
        subscriber.connect();
        subscriber.wait_for_connect().await?;

        Ok(Self { pool, subscriber })
    }

    async fn run(&self) -> Result<()> {
        info!("ASM: Starting autonomous session monitor...");

        info!("ASM: Getting message stream...");
        let mut message_stream = self.subscriber.message_rx();

        info!("ASM: Subscribing to koad:sessions...");
        self.subscriber.subscribe("koad:sessions").await?;
        info!("ASM: Subscription successful. Real-time update listener ACTIVE.");

        let reaper_pool = self.pool.clone();
        tokio::spawn(async move {
            info!("ASM: Prune task spawned.");
            loop {
                if let Err(e) = SessionManager::static_prune(&reaper_pool).await {
                    error!("ASM: Prune cycle failed: {}", e);
                }
                sleep(Duration::from_secs(30)).await;
            }
        });

        while let Ok(message) = message_stream.recv().await {
            let payload_str = message.value.as_string().unwrap_or_default();
            info!(
                "ASM: Received raw bus message on {}: {}",
                message.channel, payload_str
            );
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&payload_str) {
                let msg_type = msg["type"].as_str().unwrap_or_default().to_uppercase();
                match msg_type.as_str() {
                    "SESSION_UPDATE" => {
                        if let Ok(session) =
                            serde_json::from_value::<AgentSession>(msg["data"].clone())
                        {
                            info!(
                                "ASM: Session Update -> KAI: '{}' (ID: {}, Status: {})",
                                session.identity.name, session.session_id, session.status
                            );
                        }
                    }
                    "SESSION_PRUNED" => {
                        if let Some(sid) = msg["session_id"].as_str() {
                            info!("ASM: Session Purged -> ID: {}", sid);
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    async fn static_prune(pool: &RedisPool) -> Result<()> {
        let all_state: HashMap<String, String> = pool.next().hgetall("koad:state").await?;
        // ... (rest of logic same as before, moved to static for spawn)
        let mut to_remove = Vec::new();
        let mut to_dark = Vec::new();

        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                    let data = if let Some(inner) = raw_json.get("data") {
                        inner
                    } else {
                        &raw_json
                    };
                    if let Ok(session) = serde_json::from_value::<AgentSession>(data.clone()) {
                        if session.identity.name == "Koad"
                            || session.identity.name == "Tyr"
                            || session.identity.name == "Dood"
                        {
                            continue;
                        }
                        let diff = Utc::now().signed_duration_since(session.last_heartbeat);
                        if diff.num_seconds() > 300 {
                            to_remove.push(key.clone());
                        } else if diff.num_seconds() > 60 && session.status != "dark" {
                            let mut updated = session.clone();
                            updated.status = "dark".to_string();
                            to_dark.push((key.clone(), updated));
                        }
                    }
                }
            }
        }

        for key in to_remove {
            let _: () = pool.next().hdel("koad:state", &key).await?;
            let sid = key.replace("koad:session:", "");
            let msg = serde_json::json!({ "type": "SESSION_PRUNED", "session_id": sid });
            let _: () = pool
                .next()
                .publish("koad:sessions", msg.to_string())
                .await?;
        }

        for (key, session) in to_dark {
            let payload = serde_json::to_value(&session)?;
            let _: () = pool
                .next()
                .hset("koad:state", (&key, payload.to_string()))
                .await?;
            let msg = serde_json::json!({ "type": "SESSION_UPDATE", "data": payload });
            let _: () = pool
                .next()
                .publish("koad:sessions", msg.to_string())
                .await?;
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = KoadConfig::load().context("Failed to load config")?;
    let log_dir = Some(config.home.join("logs"));
    let _guard = init_logging("koad-asm", log_dir);

    info!("KoadOS Agent Session Manager (ASM) starting...");

    let manager = SessionManager::new(config).await?;
    manager.run().await?;

    Ok(())
}
