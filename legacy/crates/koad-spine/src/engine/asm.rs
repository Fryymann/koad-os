use crate::engine::storage_bridge::KoadStorageBridge;
use chrono::Utc;
use fred::interfaces::{
    EventInterface, HashesInterface, PubsubInterface, SetsInterface, StreamsInterface,
};
use koad_core::config::KoadConfig;
use koad_core::session::AgentSession;
use koad_core::types::HotContextChunk;
use koad_core::intelligence::ContextSummary;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct AgentSessionManager {
    storage: Arc<KoadStorageBridge>,
    config: Arc<KoadConfig>,
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
}

impl AgentSessionManager {
    pub fn new(storage: Arc<KoadStorageBridge>, config: Arc<KoadConfig>) -> Self {
        Self {
            storage,
            config,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_config(&self) -> Arc<KoadConfig> {
        self.config.clone()
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

    pub async fn get_session(&self, session_id: &str) -> anyhow::Result<Option<AgentSession>> {
        let sessions = self.sessions.lock().await;
        Ok(sessions.get(session_id).cloned())
    }

    pub async fn remove_session(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        sessions.remove(session_id);
        Ok(())
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
            "Welcome, Agent {}. Persona: {}. Role: {:?}. Current Project: {}. System Status: CONDITION GREEN. You have {} active tasks.

### Mission Context (Deterministic)
- GitHub Organization: {}
- GitHub Project: #{}
- GitHub Repository: {}
- Authorization: {}
",
            session.identity.name,
            bio,
            session.identity.rank,
            session.context.project_name,
            active_tasks.len(),
            env::var("GITHUB_OWNER").unwrap_or_else(|_| koad_core::constants::DEFAULT_GITHUB_OWNER.to_string()),
            env::var("GITHUB_PROJECT_NUMBER").unwrap_or_else(|_| "2".to_string()),
            env::var("GITHUB_REPO").unwrap_or_else(|_| koad_core::constants::DEFAULT_GITHUB_REPO.to_string()),
            env::var("GITHUB_PAT").map(|p| format!("{}...", &p[..12])).unwrap_or_else(|_| "NOT_FOUND".to_string())
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

        let mut raw_chunks = Vec::new();
        for (_, val) in hot_context_raw {
            if let Ok(chunk) = serde_json::from_str::<koad_core::types::HotContextChunk>(&val) {
                raw_chunks.push(chunk);
            }
        }

        // --- Intelligence Layer: Ranked L1/L2 Assembly ---
        let max_budget = self.resolve_token_budget(session);
        let (hot_context, summary) = self.rank_and_prune_context(raw_chunks, max_budget);

        // --- Intelligence Layer: Shared L3 Memory Bus ---
        let shared_knowledge = self
            .storage
            .query_facts(&session.context.project_name, 5)
            .await
            .unwrap_or_default();

        let mut package = serde_json::to_value(session)?;
        if let Some(obj) = package.as_object_mut() {
            obj.insert("mission_briefing".to_string(), json!(briefing));
            obj.insert("active_tasks".to_string(), json!(active_tasks));
            obj.insert(
                "recent_events".to_string(),
                json!(events.into_iter().map(|e| e.1).collect::<Vec<_>>()),
            );
            obj.insert("hot_context".to_string(), json!(hot_context));
            obj.insert("shared_knowledge".to_string(), json!(shared_knowledge));
            if let Some(s) = summary {
                obj.insert("living_summary".to_string(), json!(s));
            }
        }

        Ok(package)
    }

    fn resolve_token_budget(&self, session: &AgentSession) -> usize {
        if let Some(custom) = session.metadata.get("max_context_tokens") {
            if let Ok(val) = custom.parse::<usize>() {
                return val;
            }
        }

        // Default budgets by Rank
        match session.identity.rank {
            koad_core::identity::Rank::Admiral => 128_000,
            koad_core::identity::Rank::Captain => 32_000,
            koad_core::identity::Rank::Officer => 16_000,
            koad_core::identity::Rank::Crew => 8_000,
        }
    }

    fn rank_and_prune_context(
        &self,
        mut chunks: Vec<HotContextChunk>,
        max_budget: usize,
    ) -> (Vec<HotContextChunk>, Option<ContextSummary>) {
        if chunks.is_empty() {
            return (Vec::new(), None);
        }

        // 1. Ranking (Significance desc, Time desc)
        chunks.sort_by(|a, b| {
            b.significance_score
                .partial_cmp(&a.significance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.created_at.cmp(&a.created_at))
        });

        // 2. Budget Enforcement (Approximate 4 chars = 1 token)
        let mut total_est_tokens = 0;
        let mut accepted = Vec::new();
        let mut rejected = Vec::new();

        for chunk in chunks {
            let chunk_tokens = chunk.content.len() / 4;
            if total_est_tokens + chunk_tokens <= max_budget {
                total_est_tokens += chunk_tokens;
                accepted.push(chunk);
            } else {
                rejected.push(chunk);
            }
        }

        // 3. Summarization (If we rejected significant chunks)
        let summary = if !rejected.is_empty() {
            let high_signal_rejected: Vec<_> = rejected
                .iter()
                .filter(|c| c.significance_score > 0.5)
                .collect();
            
            if !high_signal_rejected.is_empty() {
                Some(ContextSummary {
                    session_id: "".to_string(), // Filled by caller if needed
                    summary: format!(
                        "Note: {} high-signal context chunks were omitted due to budget constraints.",
                        high_signal_rejected.len()
                    ),
                    turn_count: rejected.len(),
                    last_message_id: "".to_string(),
                    updated_at: Utc::now(),
                })
            } else {
                None
            }
        } else {
            None
        };

        (accepted, summary)
    }

    pub async fn prune_body_ghosts(
        &self,
        agent_name: &str,
        driver_id: &str,
        environment: koad_core::types::EnvironmentType,
        new_session_id: &str,
    ) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let mut ghosts = Vec::new();

        for sess in sessions.values() {
            // Find sessions from same agent, driver, and environment that aren't the new one
            if sess.session_id != new_session_id
                && sess.identity.name == agent_name
                && sess.metadata.get("driver_id").map(|d| d.as_str()).unwrap_or("") == driver_id
                && sess.environment == environment
            {
                ghosts.push(sess.session_id.clone());
            }
        }

        for sid in ghosts {
            if let Some(sess) = sessions.get_mut(&sid) {
                info!("ASM (Body Enforcement): Pre-empting ghost session {} for Agent {} on driver {}", sid, agent_name, driver_id);
                sess.status = "dark".to_string();
                
                // Authoritative update in Redis
                let payload = serde_json::to_value(&sess)?;
                let session_key = format!("koad:session:{}", sid);
                let _: () = self.storage.redis.pool.next()
                    .hset("koad:state", (&session_key, payload.to_string()))
                    .await?;

                // Broadcast to data plane
                let msg = json!({ "type": "SESSION_UPDATE", "data": payload });
                let _: () = self.storage.redis.pool.next()
                    .publish("koad:sessions", msg.to_string())
                    .await?;
            }
        }

        Ok(())
    }

    pub async fn prune_sessions(&self, _timeout_secs: i64) -> anyhow::Result<()> {
        // Legacy: Replaced by start_reaper logic
        Ok(())
    }

    pub async fn start_reaper(&self, config: Arc<KoadConfig>) {
        info!("ASM Reaper: Integrated reaper loop starting...");
        let interval = config.sessions.reaper_interval_secs;
        let lease_duration = config.sessions.lease_duration_secs;
        let dark_timeout = config.sessions.dark_timeout_secs;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(interval)).await;

            let now = Utc::now();
            let mut sessions_to_prune = Vec::new();
            let mut leases_to_expire = Vec::new();

            // 1. Scan Redis for all active sessions/leases
            let all_state: HashMap<String, String> = match self.storage.redis.pool.hgetall("koad:state").await {
                Ok(data) => data,
                Err(e) => {
                    error!("ASM Reaper Error: Failed to scan koad:state: {}", e);
                    continue;
                }
            };

            for (key, val) in all_state {
                // Handle Leases (Expiration -> DARK)
                if key.starts_with("koad:kai:") && key.ends_with(":lease") {
                    if let Ok(lease) = serde_json::from_str::<serde_json::Value>(&val) {
                        if let Some(expires_str) = lease["expires_at"].as_str() {
                            if let Ok(expires_at) = chrono::DateTime::parse_from_rfc3339(expires_str) {
                                if expires_at.with_timezone(&Utc) < now {
                                    leases_to_expire.push(key.clone());
                                }
                            }
                        }
                    }
                }

                // Handle Sessions (Dark -> DEAD/PURGE)
                if key.starts_with("koad:session:") {
                    if let Ok(session_json) = serde_json::from_str::<serde_json::Value>(&val) {
                        let data = if let Some(inner) = session_json.get("data") {
                            inner
                        } else {
                            &session_json
                        };

                        if let Ok(session) = serde_json::from_value::<AgentSession>(data.clone()) {
                            let last_seen = session.last_heartbeat;
                            let age = (now - last_seen).num_seconds();

                            if age > dark_timeout as i64 && session.status != "dark" {
                                info!("ASM Reaper: Marking session {} as DARK (Age: {}s)", session.identity.name, age);
                                let mut dark_session = session.clone();
                                dark_session.status = "dark".to_string();
                                let _ = self.update_session_status(dark_session).await;
                            } else if age > (dark_timeout + lease_duration) as i64 {
                                info!("ASM Reaper: Queuing purge for stale session {} (Age: {}s)", session.identity.name, age);
                                sessions_to_prune.push(key.clone());
                            }
                        }
                    }
                }
            }

            // Execute Pruning
            for key in leases_to_expire {
                info!("ASM Reaper: Purging expired lease: {}", key);
                let _: () = self.storage.redis.pool.next().hdel("koad:state", key).await.unwrap_or(());
            }

            for key in sessions_to_prune {
                let sid = key.replace("koad:session:", "");
                let _: () = self.storage.redis.pool.next().hdel("koad:state", &key).await.unwrap_or(());
                let msg = json!({ "type": "SESSION_PRUNED", "session_id": sid });
                let _: () = self.storage.redis.pool.next().publish::<(), _, _>("koad:sessions", msg.to_string()).await.unwrap_or(());
            }
        }
    }

    async fn update_session_status(&self, session: AgentSession) -> anyhow::Result<()> {
        let session_key = format!("koad:session:{}", session.session_id);
        let payload = serde_json::to_string(&session)?;
        
        let _: () = self.storage.redis.pool.next().hset("koad:state", (session_key, &payload)).await?;
        
        let msg = json!({ "type": "SESSION_UPDATE", "data": session });
        let _: () = self.storage.redis.pool.next().publish("koad:sessions", msg.to_string()).await?;
        
        Ok(())
    }


    pub async fn find_recoverable_session(&self, agent_name: &str, body_id: Option<&str>) -> anyhow::Result<Option<AgentSession>> {
        let all_state: HashMap<String, String> = self.storage.redis.pool.next().hgetall("koad:state").await?;
        
        let mut candidate: Option<AgentSession> = None;

        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                    let data = if let Some(inner) = raw_json.get("data") {
                        inner
                    } else {
                        &raw_json
                    };

                    if let Ok(session) = serde_json::from_value::<AgentSession>(data.clone()) {
                        if session.identity.name.to_lowercase() == agent_name.to_lowercase() {
                            // If body_id provided, it's a strong match
                            if let Some(bid) = body_id {
                                if session.body_id == bid {
                                    return Ok(Some(session));
                                }
                            }
                            
                            // Otherwise, prefer 'active' then 'dark'
                            if session.status == "active" {
                                candidate = Some(session);
                            } else if candidate.is_none() && session.status == "dark" {
                                candidate = Some(session);
                            }
                        }
                    }
                }
            }
        }
        
        Ok(candidate)
    }

    pub async fn start_session_monitor(&self) {

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
