use crate::engine::storage_bridge::KoadStorageBridge;
use chrono::Utc;
use fred::interfaces::{
    EventInterface, HashesInterface, PubsubInterface, SetsInterface, StreamsInterface,
};
use koad_core::session::AgentSession;
use koad_core::storage::StorageBridge;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

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

    pub async fn create_session(&self, session: AgentSession) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let session_id = session.session_id.clone();
        let tier = session.identity.tier;

        let payload = serde_json::to_value(&session)?;
        self.storage
            .set_state(
                &format!("koad:session:{}", session_id),
                payload.clone(),
                Some(tier),
            )
            .await?;

        sessions.insert(session_id, session);

        let msg = json!({
            "type": "SESSION_UPDATE",
            "data": payload
        });
        let _: () = self
            .storage
            .redis
            .client
            .publish("koad:sessions", msg.to_string())
            .await?;

        Ok(())
    }

    pub async fn heartbeat(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_heartbeat = Utc::now();
            let tier = session.identity.tier;

            let payload = serde_json::to_value(&session)?;
            self.storage
                .set_state(
                    &format!("koad:session:{}", session_id),
                    payload.clone(),
                    Some(tier),
                )
                .await?;

            let msg = json!({
                "type": "SESSION_UPDATE",
                "data": payload
            });
            let _: () = self
                .storage
                .redis
                .client
                .publish("koad:sessions", msg.to_string())
                .await?;

            Ok(())
        } else {
            anyhow::bail!("Session not found")
        }
    }

    pub async fn list_active_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    pub async fn hydrate_from_db(&self) -> anyhow::Result<()> {
        println!("ASM: Hydrating active sessions from database...");
        // Note: In v4.1, we rely on the storage bridge to fetch from SQLite if Redis is empty.
        // But for the ASM memory, we'll perform a direct query to find 'active' sessions.
        let mut sessions = self.sessions.lock().await;
        
        // We'll use the storage bridge's internal redis client to fetch all keys starting with koad:session
        if let Ok(all_state) = self.storage.redis.client.hgetall::<std::collections::HashMap<String, String>, _>("koad:state").await {
            for (key, val) in all_state {
                if key.starts_with("koad:session:") {
                    if let Ok(mut raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                        let data = if let Some(inner) = raw_json.get("data") { inner } else { &raw_json };
                        if data["status"] == "active" {
                            if let Ok(session) = serde_json::from_value::<AgentSession>(data.clone()) {
                                println!("ASM: Restored active session from state: {}", session.session_id);
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
            .ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let active_task_ids: Vec<String> = self
            .storage
            .redis
            .client
            .smembers("koad:active_tasks")
            .await?;
        let mut active_tasks = Vec::new();
        for id in active_task_ids {
            if let Some(state_str) = self
                .storage
                .redis
                .client
                .hget::<Option<String>, _, _>(format!("koad:task:{}", id), "state")
                .await?
            {
                active_tasks.push(serde_json::from_str::<serde_json::Value>(&state_str)?);
            }
        }

        let events: Vec<(String, HashMap<String, String>)> = self
            .storage
            .redis
            .client
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

        let mut package = serde_json::to_value(session)?;
        if let Some(obj) = package.as_object_mut() {
            obj.insert("mission_briefing".to_string(), json!(briefing));
            obj.insert("active_tasks".to_string(), json!(active_tasks));
            obj.insert(
                "recent_events".to_string(),
                json!(events.into_iter().map(|e| e.1).collect::<Vec<_>>()),
            );
        }

        self.storage
            .set_state(
                &format!("koad:session:{}", session_id),
                package.clone(),
                Some(session.identity.tier),
            )
            .await?;

        Ok(package)
    }

    pub async fn prune_sessions(&self, timeout_secs: i64) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let mut to_update = Vec::new();
        let mut to_remove = Vec::new();

        for (sid, session) in sessions.iter_mut() {
            let diff = Utc::now().signed_duration_since(session.last_heartbeat);
            if diff.num_seconds() > timeout_secs {
                // Remove entirely if very old (5 minutes inactivity)
                if diff.num_seconds() > 300 {
                    to_remove.push(sid.clone());
                } else if session.status != "dark" {
                    // Mark as dark
                    session.status = "dark".to_string();
                    to_update.push((sid.clone(), session.clone()));
                }
            } else if session.status == "dark" {
                // Reactivate if heartbeat received
                session.status = "active".to_string();
                to_update.push((sid.clone(), session.clone()));
            }
        }

        for sid in to_remove {
            println!("ASM: Pruning abandoned session: {}", sid);
            sessions.remove(&sid);
            let _: () = self
                .storage
                .redis
                .client
                .hdel("koad:state", format!("koad:session:{}", sid))
                .await?;
        }

        for (sid, session) in to_update {
            let payload = serde_json::to_value(&session)?;
            self.storage
                .set_state(
                    &format!("koad:session:{}", sid),
                    payload.clone(),
                    Some(session.identity.tier),
                )
                .await?;
            let msg = json!({ "type": "SESSION_UPDATE", "data": payload });
            let _: () = self
                .storage
                .redis
                .client
                .publish("koad:sessions", msg.to_string())
                .await?;
        }

        Ok(())
    }

    pub async fn start_session_monitor(&self) {
        println!("ASM: Session monitor active. Subscribing to 'koad:sessions'...");
        
        // v4.1.1 Lighthouse: Hydrate existing sessions so they aren't lost on restart
        let _ = self.hydrate_from_db().await;

        let mut message_stream = self.storage.redis.subscriber.message_rx();

        if let Err(e) = self
            .storage
            .redis
            .subscriber
            .subscribe("koad:sessions")
            .await
        {
            eprintln!("ASM Error: Failed to subscribe to koad:sessions: {}", e);
            return;
        }

        while let Ok(message) = message_stream.recv().await {
            println!("ASM: Received message on channel '{}'", message.channel);
            let payload_str = message.value.as_string().unwrap_or_default();
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&payload_str) {
                if msg["type"] == "session_update" || msg["type"] == "SESSION_UPDATE" {
                    println!(
                        "ASM: Processing session update for {}",
                        msg["data"]["session_id"]
                    );
                    if let Ok(session) = serde_json::from_value::<AgentSession>(msg["data"].clone())
                    {
                        let sid = session.session_id.clone();
                        let mut sessions = self.sessions.lock().await;

                        println!("ASM: Registered/Updated session: {}", sid);
                        sessions.insert(sid.clone(), session);

                        drop(sessions);
                        let _ = self.hydrate_session(&sid).await;
                    } else {
                        eprintln!(
                            "ASM: Failed to parse AgentSession from data: {:?}",
                            msg["data"]
                        );
                    }
                }
            }
        }
    }
}
