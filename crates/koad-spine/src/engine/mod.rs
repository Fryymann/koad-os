pub mod asm;
pub mod context_cache;
pub mod diagnostics;
pub mod identity;
pub mod kcm;
pub mod kernel;
pub mod redis;
pub mod router;
pub mod sandbox;
pub mod storage_bridge;
#[cfg(test)]
mod tests;

use crate::discovery::SkillRegistry;
use crate::engine::asm::AgentSessionManager;
use crate::engine::context_cache::KoadContextCache;
use crate::engine::diagnostics::ShipDiagnostics;
use crate::engine::identity::KAILeaseManager;
use crate::engine::kcm::KoadComplianceManager;
use crate::engine::redis::RedisClient;
use crate::engine::storage_bridge::KoadStorageBridge;

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Engine {
    pub redis: Arc<RedisClient>,
    pub storage: Arc<KoadStorageBridge>,
    pub diagnostics: Arc<ShipDiagnostics>,
    pub asm: Arc<AgentSessionManager>,
    pub context_cache: Arc<KoadContextCache>,
    pub identity: Arc<KAILeaseManager>,
    pub kcm: Arc<KoadComplianceManager>,
    pub skill_registry: Arc<Mutex<SkillRegistry>>,
}

impl Engine {
    pub async fn new(koad_home: &str, sqlite_path: &str) -> anyhow::Result<Self> {
        let redis = Arc::new(RedisClient::new(koad_home).await?);
        let storage = Arc::new(KoadStorageBridge::new(redis.clone(), sqlite_path)?);
        let asm = Arc::new(AgentSessionManager::new(storage.clone()));
        let context_cache = Arc::new(KoadContextCache::new(redis.clone()));
        let identity = Arc::new(KAILeaseManager::new(storage.clone()));
        let kcm = Arc::new(KoadComplianceManager::new(storage.clone()));
        let skill_registry = Arc::new(Mutex::new(SkillRegistry::new()));

        let diagnostics = Arc::new(ShipDiagnostics::new(redis.clone(), storage.clone(), identity.clone(), skill_registry.clone()));

        Ok(Self {
            redis: redis.clone(),
            storage,
            diagnostics,
            asm,
            context_cache,
            identity,
            kcm,
            skill_registry,
        })
    }
}
