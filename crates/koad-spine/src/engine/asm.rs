use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use koad_core::session::AgentSession;
use crate::engine::storage_bridge::KoadStorageBridge;
use koad_core::storage::StorageBridge;
use chrono::Utc;
use serde_json::json;
use fred::interfaces::{PubsubInterface, HashesInterface, StreamsInterface, SetsInterface, EventInterface};

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
        
        let payload = serde_json::to_value(&session)?;
        self.storage.set_state(&format!("koad:session:{}", session_id), payload.clone()).await?;
        
        sessions.insert(session_id, session);

        let msg = json!({
            "type": "SESSION_UPDATE",
            "payload": payload
        });
        let _: () = self.storage.redis.client.publish("koad:sessions", msg.to_string()).await?;
        
        Ok(())
    }

    pub async fn heartbeat(&self, session_id: &str) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.last_heartbeat = Utc::now();
            
            let payload = serde_json::to_value(&session)?;
            self.storage.set_state(&format!("koad:session:{}", session_id), payload.clone()).await?;

            let msg = json!({
                "type": "SESSION_UPDATE",
                "payload": payload
            });
            let _: () = self.storage.redis.client.publish("koad:sessions", msg.to_string()).await?;

            Ok(())
        } else {
            anyhow::bail!("Session not found")
        }
    }

    pub async fn list_active_sessions(&self) -> Vec<AgentSession> {
        let sessions = self.sessions.lock().await;
        sessions.values().cloned().collect()
    }

    pub async fn hydrate_session(&self, session_id: &str) -> anyhow::Result<serde_json::Value> {
        let sessions = self.sessions.lock().await;
        let session = sessions.get(session_id).ok_or_else(|| anyhow::anyhow!("Session not found"))?;

        let active_task_ids: Vec<String> = self.storage.redis.client.smembers("koad:active_tasks").await?;
        let mut active_tasks = Vec::new();
        for id in active_task_ids {
            if let Some(state_str) = self.storage.redis.client.hget::<Option<String>, _, _>(format!("koad:task:{}", id), "state").await? {
                active_tasks.push(serde_json::from_str::<serde_json::Value>(&state_str)?);
            }
        }

        let events: Vec<(String, HashMap<String, String>)> = self.storage.redis.client.xrevrange(
            "koad:events:stream", "+", "-", Some(10)
        ).await?;

        let briefing = format!(
            "Welcome, Agent {}. Role: {:?}. Current Project: {}. System Status: CONDITION GREEN. You have {} active tasks.",
            session.identity.name,
            session.identity.rank,
            session.context.project_name,
            active_tasks.len()
        );

        let package = json!({
            "session_id": session.session_id,
            "mission_briefing": briefing,
            "active_tasks": active_tasks,
            "recent_events": events.into_iter().map(|e| e.1).collect::<Vec<_>>(),
            "context": session.context,
            "identity": session.identity
        });

        self.storage.set_state(&format!("koad:session:{}", session_id), package.clone()).await?;

        Ok(package)
    }

    pub async fn prune_sessions(&self, timeout_secs: i64) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let original_count = sessions.len();
        
        sessions.retain(|_, s| s.is_active(timeout_secs));
        
        if sessions.len() < original_count {
            println!("ASM: Pruned {} inactive sessions.", original_count - sessions.len());
        }
        
        Ok(())
    }

    pub async fn start_session_monitor(&self) {
        println!("ASM: Session monitor active. Subscribing to 'koad:sessions'...");
        let mut message_stream = self.storage.redis.subscriber.message_rx();
        
        if let Err(e) = self.storage.redis.subscriber.subscribe("koad:sessions").await {
            eprintln!("ASM Error: Failed to subscribe to koad:sessions: {}", e);
            return;
        }

        while let Ok(message) = message_stream.recv().await {
            println!("ASM: Received message on channel '{}'", message.channel);
            let payload_str = message.value.as_string().unwrap_or_default();
            if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&payload_str) {
                if msg["type"] == "session_update" || msg["type"] == "SESSION_UPDATE" {
                    println!("ASM: Processing session update for {}", msg["data"]["session_id"]);
                    if let Ok(session) = serde_json::from_value::<AgentSession>(msg["data"].clone()) {
                        let mut sessions = self.sessions.lock().await;
                        if !sessions.contains_key(&session.session_id) {
                            println!("ASM: Registered external session: {}", session.session_id);
                            let sid = session.session_id.clone();
                            sessions.insert(sid.clone(), session);
                            
                            drop(sessions);
                            let _ = self.hydrate_session(&sid).await;
                        }
                    } else {
                        eprintln!("ASM: Failed to parse AgentSession from data: {:?}", msg["data"]);
                    }
                }
            }
        }
    }
}
