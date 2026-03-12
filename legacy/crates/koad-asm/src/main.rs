use anyhow::{Context, Result};
use chrono::Utc;
use fred::interfaces::{HashesInterface, PubsubInterface};
use fred::prelude::*;
use koad_core::config::KoadConfig;
use koad_core::logging::init_logging;
use koad_core::session::AgentSession;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::Empty;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, warn};

struct SessionManager {
    pool: RedisPool,
    subscriber: fred::clients::RedisClient,
    config: KoadConfig,
}

fn system_request<T>(payload: T) -> tonic::Request<T> {
    let mut req = tonic::Request::new(payload);
    req.metadata_mut().insert("x-system-key", "citadel-core".parse().unwrap());
    req
}

impl SessionManager {
    async fn new(config: KoadConfig) -> Result<Self> {
        let redis_url = format!("redis+unix://{}", config.get_redis_socket().display());
        info!("ASM: Connecting to Redis at {}", redis_url);
        let redis_config = RedisConfig::from_url(&redis_url)?;

        let pool = RedisPool::new(redis_config.clone(), None, None, None, 6)?;
        pool.connect();
        pool.wait_for_connect().await?;

        let subscriber = fred::clients::RedisClient::new(redis_config, None, None, None);
        subscriber.connect();
        subscriber.wait_for_connect().await?;

        Ok(Self { pool, subscriber, config })
    }

    async fn run(&self) -> Result<()> {
        info!("ASM: Starting autonomous session monitor...");

        info!("ASM: Getting message stream...");
        let mut message_stream = self.subscriber.message_rx();

        info!("ASM: Subscribing to koad:sessions...");
        self.subscriber.subscribe("koad:sessions").await?;
        info!("ASM: Subscription successful. Real-time update listener ACTIVE.");

        let reaper_pool = self.pool.clone();
        let reaper_config = self.config.clone();
        tokio::spawn(async move {
            info!("ASM: Prune task spawned.");
            let interval = Duration::from_secs(reaper_config.sessions.dark_timeout_secs / 2);
            loop {
                if let Err(e) = SessionManager::static_prune(&reaper_pool, &reaper_config).await {
                    error!("ASM: Prune cycle failed: {}", e);
                }
                sleep(interval).await;
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

    async fn static_prune(pool: &RedisPool, config: &KoadConfig) -> Result<()> {
        let all_state: HashMap<String, String> = pool.next().hgetall("koad:state").await?;
        let mut to_remove = Vec::new();
        let mut to_dark = Vec::new();
        let mut leases_to_remove = Vec::new();
        let mut deadman_triggered = false;

        let now = Utc::now();

        // Map identity -> session_id from current leases
        let mut active_leases = HashMap::new();
        for (key, val) in &all_state {
            if key.starts_with("koad:kai:") && key.ends_with(":lease") {
                if let Ok(lease) = serde_json::from_str::<serde_json::Value>(val) {
                    if let (Some(name), Some(sid)) = (lease["identity_name"].as_str(), lease["session_id"].as_str()) {
                        active_leases.insert(name.to_string(), sid.to_string());
                    }
                }
            }
        }

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
                        let diff = now.signed_duration_since(session.last_heartbeat);
                        
                        // --- [Policy Resolution] ---
                        // Resolve per-agent overrides if they exist in identities/*.toml
                        let (deadman_timeout, dark_timeout, purge_timeout) = {
                            if let Some(id_config) = config.identities.get(&session.identity.name) {
                                if let Some(policy) = &id_config.session_policy {
                                    (
                                        policy.deadman_timeout_secs.unwrap_or(config.sessions.deadman_timeout_secs),
                                        policy.dark_timeout_secs.unwrap_or(config.sessions.dark_timeout_secs),
                                        policy.purge_timeout_secs.unwrap_or(config.sessions.purge_timeout_secs),
                                    )
                                } else {
                                    (config.sessions.deadman_timeout_secs, config.sessions.dark_timeout_secs, config.sessions.purge_timeout_secs)
                                }
                            } else {
                                (config.sessions.deadman_timeout_secs, config.sessions.dark_timeout_secs, config.sessions.purge_timeout_secs)
                            }
                        };

                        // Rule 0: Ghost Enforcement (Lease Synchronization)
                        if let Some(active_sid) = active_leases.get(&session.identity.name) {
                            if active_sid != &session.session_id {
                                info!("ASM Reaper: Purging ghost session {} (Newer lease exists for {})", session.session_id, session.identity.name);
                                to_remove.push(key.clone());
                                continue;
                            }
                        }

                        // Rule 1: Deadman Switch (Tier 1 Flatline)
                        if session.identity.tier == 1 && diff.num_seconds() > deadman_timeout as i64 && session.status == "active" {
                            warn!("ASM ALERT: Tier 1 Agent '{}' Flatline Detected (Age: {}s)!", session.identity.name, diff.num_seconds());
                            deadman_triggered = true;
                        }

                        // Rule 2: Stale Heartbeat -> Purge
                        if diff.num_seconds() > purge_timeout as i64 {
                            info!("ASM Reaper: Queuing purge for stale session {} (Age: {}s)", session.identity.name, diff.num_seconds());
                            to_remove.push(key.clone());
                        } 
                        // Rule 3: Inactive -> Mark Dark
                        else if diff.num_seconds() > dark_timeout as i64 && session.status != "dark" {
                            info!("ASM Reaper: Marking session {} as DARK", session.identity.name);
                            let mut updated = session.clone();
                            updated.status = "dark".to_string();
                            to_dark.push((key.clone(), updated));
                        }
                    },
                    Err(e) => {
                        // Rule 4: Corrupted Entry -> Aggressive Purge
                        warn!("ASM Reaper: Queuing purge for corrupted session entry {}: {}", key, e);
                        to_remove.push(key.clone());
                    }
                }
            } else if key.starts_with("koad:kai:") && key.ends_with(":lease") {
                if let Ok(lease) = serde_json::from_str::<serde_json::Value>(&val) {
                    if let Some(expires_str) = lease["expires_at"].as_str() {
                        if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(expires_str) {
                            if expires_at.with_timezone(&Utc) < now {
                                info!("ASM Reaper: Queuing purge for expired lease: {}", key);
                                leases_to_remove.push(key.clone());
                            }
                        }
                    }
                } else {
                    warn!("ASM Reaper: Queuing purge for corrupted lease entry: {}", key);
                    leases_to_remove.push(key.clone());
                }
            }
        }

        if deadman_triggered {
            warn!("ASM: Executing Autonomic Emergency Save...");
            match SpineServiceClient::connect(config.network.spine_grpc_addr.clone()).await {
                Ok(mut client) => {
                    if let Err(e) = client.drain_all(system_request(Empty {})).await {
                        error!("ASM: Emergency Save FAILED: {}", e);
                    } else {
                        info!("ASM: Emergency Save SUCCESSFUL. State persisted.");
                    }
                },
                Err(e) => error!("ASM: Failed to connect to Spine for emergency save: {}", e),
            }
        }

        for key in to_remove {
            info!("ASM Reaper: Purging session key {}", key);
            let _: () = pool.next().hdel("koad:state", &key).await?;
            let sid = key.replace("koad:session:", "");
            let msg = serde_json::json!({ "type": "SESSION_PRUNED", "session_id": sid });
            let _: () = pool
                .next()
                .publish("koad:sessions", msg.to_string())
                .await?;
        }

        for key in leases_to_remove {
            info!("ASM Reaper: Purging lease key {}", key);
            let _: () = pool.next().hdel("koad:state", &key).await?;
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
