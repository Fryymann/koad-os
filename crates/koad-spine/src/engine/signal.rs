use crate::engine::storage_bridge::KoadStorageBridge;
use koad_core::signal::{GhostSignal, SignalPriority, SignalStatus};
use fred::interfaces::HashesInterface;
use std::sync::Arc;
use tracing::{info, error};

pub struct SignalManager {
    storage: Arc<KoadStorageBridge>,
}

impl SignalManager {
    pub fn new(storage: Arc<KoadStorageBridge>) -> Self {
        Self { storage }
    }

    pub async fn send_signal(
        &self,
        source: String,
        target: String,
        message: String,
        priority: SignalPriority,
        metadata: std::collections::HashMap<String, String>,
    ) -> anyhow::Result<()> {
        let mut signal = GhostSignal::new(source, target.clone(), message, priority);
        signal.metadata = metadata;

        let payload = serde_json::to_string(&signal)?;
        let key = format!("koad:mailbox:{}", target);

        let _: () = self.storage.redis.pool.next()
            .hset(&key, (&signal.id, payload))
            .await?;

        info!("Signal {} sent from {} to {}", signal.id, signal.source_agent, target);
        Ok(())
    }

    pub async fn get_signals(&self, agent_name: &str) -> anyhow::Result<Vec<GhostSignal>> {
        let key = format!("koad:mailbox:{}", agent_name);
        let data: std::collections::HashMap<String, String> = self.storage.redis.pool.next()
            .hgetall(&key)
            .await?;

        let mut signals = Vec::new();
        for (_, val) in data {
            if let Ok(sig) = serde_json::from_str::<GhostSignal>(&val) {
                signals.push(sig);
            }
        }

        // Sort by timestamp desc
        signals.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(signals)
    }

    pub async fn update_signal_status(
        &self,
        agent_name: &str,
        signal_id: &str,
        status: SignalStatus,
    ) -> anyhow::Result<()> {
        let key = format!("koad:mailbox:{}", agent_name);
        if let Some(val) = self.storage.redis.pool.next().hget::<Option<String>, _, _>(&key, signal_id).await? {
            let mut sig = serde_json::from_str::<GhostSignal>(&val)?;
            sig.status = status;
            let payload = serde_json::to_string(&sig)?;
            let _: () = self.storage.redis.pool.next().hset(&key, (signal_id, payload)).await?;
            info!("Signal {} status updated to {:?}", signal_id, sig.status);
        }
        Ok(())
    }
}
