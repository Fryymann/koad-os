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
        let mut to_remove = Vec::new();
        let mut to_dark = Vec::new();

        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                let parse_result = serde_json::from_str::<serde_json::Value>(&val).and_then(|raw_json| {
                    let data = if let Some(inner) = raw_json.get("data") {
                        inner
                    } else {
                        &raw_json
                    };
                    serde_json::from_value::<AgentSession>(data.clone()).map_err(|e| e.into())
                });

                match parse_result {
                    Ok(session) => {
                        let diff = Utc::now().signed_duration_since(session.last_heartbeat);
                        // Rule 1: Stale Heartbeat (> 5 minutes) -> Purge
                        if diff.num_seconds() > 300 {
                            info!("ASM Reaper: Purging stale session {} (Heartbeat age: {}s)", session.identity.name, diff.num_seconds());
                            to_remove.push(key.clone());
                        } 
                        // Rule 2: Inactive (> 1 minute) -> Mark Dark
                        else if diff.num_seconds() > 60 && session.status != "dark" {
                            info!("ASM Reaper: Marking session {} as DARK", session.identity.name);
                            let mut updated = session.clone();
                            updated.status = "dark".to_string();
                            to_dark.push((key.clone(), updated));
                        }
                    },
                    Err(e) => {
                        // Rule 3: Corrupted Entry -> Aggressive Purge
                        warn!("ASM Reaper: Purging corrupted session entry {}: {}", key, e);
                        to_remove.push(key.clone());
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
