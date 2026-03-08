use crate::engine::storage_bridge::KoadStorageBridge;
use chrono::Utc;
use fred::interfaces::{
    EventInterface, HashesInterface, PubsubInterface, SetsInterface, StreamsInterface,
};
use koad_core::session::AgentSession;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct AgentSessionManager {
    storage: Arc<KoadStorageBridge>,
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
}

impl AgentSessionManager {
    pub fn new(storage: Arc<KoadStorageBridge>) -> Self {
        Self {
            storage,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Passive creation: Just updates the local cache.
    /// Real creation is handled by the agent directly via Redis/ASM daemon.
    pub async fn create_session(&self, session: AgentSession) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        info!(
            "ASM (Watcher): Local cache update for KAI '{}'",
            session.identity.name
        );
        sessions.insert(session.session_id.clone(), session);
        Ok(())
    }

    pub async fn heartbeat(&self, session_id: &str) -> anyhow::Result<()> {
        // Spine no longer authoritative for heartbeats.
        // Agents should heartbeat directly to Redis/ASM.
        // We just update local cache if we have it.
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_heartbeat = Utc::now();
        }
        Ok(())
    }

    pub async fn list_active_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    pub async fn hydrate_from_db(&self) -> anyhow::Result<()> {
        info!("ASM: Hydrating active sessions from Redis...");
        let mut sessions = self.sessions.lock().await;

        if let Ok(all_state) = self
            .storage
            .redis
            .pool
            .next()
            .hgetall::<std::collections::HashMap<String, String>, _>("koad:state")
            .await
        {
            for (key, val) in all_state {
                if key.starts_with("koad:session:") {
                    if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                        let data = if let Some(inner) = raw_json.get("data") {
                            inner
                        } else {
                            &raw_json
                        };
                        if let Ok(session) = serde_json::from_value::<AgentSession>(data.clone()) {
                            if session.status == "active" {
                                sessions.insert(session.session_id.clone(), session);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub async fn hydrate_session(&self, session_id: &str) -> anyhow::Result<serde_json::Value> {
        let sessions = self.sessions.lock().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session {} not found in local cache", session_id))?;

        let active_task_ids: Vec<String> = self
            .storage
            .redis
            .pool
            .next()
            .smembers("koad:active_tasks")
            .await?;
        let mut active_tasks = Vec::new();
        for id in active_task_ids {
            if let Some(state_str) = self
                .storage
                .redis
                .pool
                .next()
                .hget::<Option<String>, _, _>(format!("koad:task:{}", id), "state")
                .await?
            {
                active_tasks.push(serde_json::from_str::<serde_json::Value>(&state_str)?);
            }
        }

        let events: Vec<(String, HashMap<String, String>)> = self
            .storage
            .redis
            .pool
            .next()
            .xrevrange("koad:events:stream", "+", "-", Some(10))
            .await?;

        let bio = session
            .metadata
            .get("bio")
            .cloned()
            .unwrap_or_else(|| "General Purpose Agent".to_string());

        let briefing = format!(
            "Welcome, Agent {}. Persona: {}. Role: {:?}. Current Project: {}. System Status: CONDITION GREEN. You have {} active tasks.",
            session.identity.name,
            bio,
            session.identity.rank,
            session.context.project_name,
            active_tasks.len()
        );

        // Fetch Hot Context Chunks
        let context_key = format!("koad:context:session:{}", session_id);
        let hot_context_raw: HashMap<String, String> = self
            .storage
            .redis
            .pool
            .next()
            .hgetall(&context_key)
            .await
            .unwrap_or_default();

        let mut hot_context = Vec::new();
        for (_, val) in hot_context_raw {
            if let Ok(chunk) = serde_json::from_str::<koad_core::types::HotContextChunk>(&val) {
                hot_context.push(chunk);
            }
        }

        let mut package = serde_json::to_value(session)?;
        if let Some(obj) = package.as_object_mut() {
            obj.insert("mission_briefing".to_string(), json!(briefing));
            obj.insert("active_tasks".to_string(), json!(active_tasks));
            obj.insert(
                "recent_events".to_string(),
                json!(events.into_iter().map(|e| e.1).collect::<Vec<_>>()),
            );
            obj.insert("hot_context".to_string(), json!(hot_context));
        }

        Ok(package)
    }

    pub async fn prune_sessions(&self, _timeout_secs: i64) -> anyhow::Result<()> {
        // Spine no longer authoritative for pruning. koad-asm daemon handles this.
        Ok(())
    }

    pub async fn start_session_monitor(&self) {
        info!("ASM: Session monitor active. Subscribing to 'koad:sessions'...");

        let _ = self.hydrate_from_db().await;

        let mut message_stream = self.storage.redis.subscriber.message_rx();

        if let Err(e) = self
            .storage
            .redis
            .subscriber
            .subscribe("koad:sessions")
            .await
        {
            error!("ASM Watcher Error: Failed to subscribe: {}", e);
            return;
        }

        while let Ok(message) = message_stream.recv().await {
            let payload_str = message.value.as_string().unwrap_or_default();
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&payload_str) {
                let msg_type = msg["type"].as_str().unwrap_or_default().to_uppercase();

                match msg_type.as_str() {
                    "SESSION_UPDATE" => {
                        if let Ok(session) =
                            serde_json::from_value::<AgentSession>(msg["data"].clone())
                        {
                            let sid = session.session_id.clone();
                            let mut sessions = self.sessions.lock().await;
                            sessions.insert(sid, session);
                        }
                    }
                    "SESSION_PRUNED" => {
                        if let Some(sid) = msg["session_id"].as_str() {
                            let mut sessions = self.sessions.lock().await;
                            sessions.remove(sid);
                            info!("ASM (Watcher): Session {} purged from cache.", sid);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}
