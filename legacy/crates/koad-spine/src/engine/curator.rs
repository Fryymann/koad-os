use koad_core::intelligence::FactCard;
use koad_core::utils::redis::RedisClient;
use crate::engine::storage_bridge::KoadStorageBridge;
use chrono::Utc;
use fred::interfaces::HashesInterface;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{error, info};

pub struct CognitiveCurator {
    redis: Arc<RedisClient>,
    storage: Arc<KoadStorageBridge>,
}

impl CognitiveCurator {
    pub fn new(redis: Arc<RedisClient>, storage: Arc<KoadStorageBridge>) -> Self {
        Self { redis, storage }
    }

    pub async fn curate_intelligence(&self) -> anyhow::Result<()> {
        let all_state: HashMap<String, String> = self
            .redis
            .pool
            .next()
            .hgetall("koad:state")
            .await
            .unwrap_or_default();

        let mut active_sessions = Vec::new();
        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                    let data = if let Some(inner) = raw_json.get("data") {
                        inner
                    } else {
                        &raw_json
                    };
                    if let Ok(sess) = serde_json::from_value::<koad_core::session::AgentSession>(data.clone()) {
                        if sess.status == "active" {
                            active_sessions.push(sess);
                        }
                    }
                }
            }
        }

        for sess in active_sessions {
            let context_key = format!("koad:context:session:{}", sess.session_id);
            let hot_context: HashMap<String, String> = self
                .redis
                .pool
                .next()
                .hgetall(&context_key)
                .await
                .unwrap_or_default();

            for (chunk_id, val) in hot_context {
                if let Ok(chunk) = serde_json::from_str::<koad_core::types::HotContextChunk>(&val) {
                    // Promote high-signal chunks to durable L3 FactCards
                    if chunk.significance_score >= 0.8 {
                        info!("Cognitive Curator: Promoting chunk {} from session {} to L3 Memory Bank.", chunk_id, sess.session_id);
                        
                        let fact = FactCard {
                            id: uuid::Uuid::new_v4(),
                            source_agent: sess.identity.name.clone(),
                            session_id: sess.session_id.clone(),
                            domain: chunk.tags.first().cloned().unwrap_or_else(|| "general".to_string()),
                            content: chunk.content.clone(),
                            confidence: chunk.significance_score,
                            tags: chunk.tags.clone(),
                            created_at: Utc::now(),
                            ttl_seconds: 0, 
                        };

                        if let Err(e) = self.storage.save_fact(fact).await {
                            error!("Cognitive Curator Error: Failed to save fact card: {}", e);
                        } else {
                            let mut updated_chunk = chunk;
                            updated_chunk.tags.push("promoted".to_string());
                            updated_chunk.significance_score = 0.5;
                            if let Ok(payload) = serde_json::to_string(&updated_chunk) {
                                let _: () = self.redis.pool.next().hset(&context_key, (chunk_id, payload)).await?;
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn perform_cognitive_quicksave(&self) -> anyhow::Result<()> {
        info!("Cognitive Curator: Commencing periodic cognitive quicksave...");
        
        let all_state: HashMap<String, String> = self
            .redis
            .pool
            .next()
            .hgetall("koad:state")
            .await
            .unwrap_or_default();

        let mut active_sessions = Vec::new();
        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                    let data = if let Some(inner) = raw_json.get("data") {
                        inner
                    } else {
                        &raw_json
                    };
                    if let Ok(sess) = serde_json::from_value::<koad_core::session::AgentSession>(data.clone()) {
                        if sess.status == "active" {
                            active_sessions.push(sess);
                        }
                    }
                }
            }
        }

        for sess in active_sessions {
            let context_key = format!("koad:context:session:{}", sess.session_id);
            let hot_context: HashMap<String, String> = self
                .redis
                .pool
                .next()
                .hgetall(&context_key)
                .await
                .unwrap_or_default();

            if hot_context.is_empty() {
                continue;
            }

            let snapshot = json!({
                "session": sess,
                "hot_context": hot_context,
                "timestamp": Utc::now().timestamp()
            });

            if let Ok(json_str) = serde_json::to_string(&snapshot) {
                if let Err(e) = self.storage.save_context_snapshot(&sess.identity.name, &sess.session_id, json_str).await {
                    error!("Cognitive Curator: Failed to save context snapshot for {}: {}", sess.identity.name, e);
                } else {
                    info!("Cognitive Curator: Periodic quicksave complete for {}.", sess.identity.name);
                }
            }
        }

        Ok(())
    }
}
