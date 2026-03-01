pub mod redis;
pub mod diagnostics;
pub mod commands;
pub mod sandbox;
pub mod storage_bridge;
pub mod asm;
pub mod kernel;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use crate::engine::redis::RedisClient;
use crate::engine::diagnostics::ShipDiagnostics;
use crate::engine::storage_bridge::KoadStorageBridge;
use crate::engine::asm::AgentSessionManager;
use koad_core::storage::StorageBridge;

pub struct Engine {
    pub redis: Arc<RedisClient>,
    pub storage: Arc<KoadStorageBridge>,
    pub diagnostics: Arc<ShipDiagnostics>,
    pub asm: Arc<AgentSessionManager>,
}

impl Engine {
    pub async fn new(koad_home: &str, sqlite_path: &str) -> anyhow::Result<Self> {
        let redis = Arc::new(RedisClient::new(koad_home).await?);
        let storage = Arc::new(KoadStorageBridge::new(redis.clone(), sqlite_path)?);
        let diagnostics = Arc::new(ShipDiagnostics::new(redis.clone()));
        let asm = Arc::new(AgentSessionManager::new(storage.clone()));
        
        // Hydrate state from disk on boot
        storage.hydrate_all().await?;

        Ok(Self {
            redis,
            storage,
            diagnostics,
            asm,
        })
    }
}
