use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use koad_core::session::AgentSession;
use crate::engine::storage_bridge::KoadStorageBridge;
use koad_core::storage::StorageBridge;
use chrono::Utc;

pub struct AgentSessionManager {
    storage: Arc<KoadStorageBridge>,
    sessions: Arc<Mutex<HashMap<String, AgentSession>>>,
}

use fred::interfaces::PubsubInterface;

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
        
        // 1. Persist to Storage Bridge
        let payload = serde_json::to_value(&session)?;
        self.storage.set_state(&format!("koad:session:{}", session_id), payload.clone()).await?;
        
        // 2. Add to active memory
        sessions.insert(session_id, session);

        // 3. Notify subscribers
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
            
            // Sync heartbeat to storage
            let payload = serde_json::to_value(&session)?;
            self.storage.set_state(&format!("koad:session:{}", session_id), payload.clone()).await?;

            // Notify subscribers
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

    pub async fn prune_sessions(&self, timeout_secs: i64) -> anyhow::Result<()> {
        let mut sessions = self.sessions.lock().await;
        let original_count = sessions.len();
        
        sessions.retain(|_, s| s.is_active(timeout_secs));
        
        if sessions.len() < original_count {
            println!("ASM: Pruned {} inactive sessions.", original_count - sessions.len());
        }
        
        Ok(())
    }
}
