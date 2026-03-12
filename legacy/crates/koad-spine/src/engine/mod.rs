pub mod asm;
pub mod backup;
pub mod config;
pub mod context_cache;
pub mod curator;
pub mod diagnostics;
pub mod hydration;
pub mod identity;
pub mod kcm;
pub mod kernel;
pub mod router;
pub mod sandbox;
pub mod signal;
pub mod storage_bridge;
#[cfg(test)]
mod tests;

use crate::discovery::SkillRegistry;
use crate::engine::asm::AgentSessionManager;
use crate::engine::config::ConfigManager;
use crate::engine::context_cache::KoadContextCache;
use crate::engine::diagnostics::ShipDiagnostics;
use crate::engine::identity::KAILeaseManager;
use crate::engine::kcm::KoadComplianceManager;
use crate::engine::signal::SignalManager;
use koad_core::utils::redis::RedisClient;
use crate::engine::storage_bridge::KoadStorageBridge;

use koad_core::config::KoadConfig;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Engine {
    pub config: KoadConfig,
    pub redis: Arc<RedisClient>,
    pub config_manager: Arc<ConfigManager>,
    pub storage: Arc<KoadStorageBridge>,
    pub diagnostics: Arc<ShipDiagnostics>,
    pub asm: Arc<AgentSessionManager>,
    pub curator: Arc<curator::CognitiveCurator>,
    pub backup: Arc<backup::KoadBackupManager>,
    pub context_cache: Arc<KoadContextCache>,
    pub hydration: Arc<hydration::KoadHydrationManager>,
    pub identity: Arc<KAILeaseManager>,
    pub signal: Arc<SignalManager>,
    pub kcm: Arc<KoadComplianceManager>,
    pub skill_registry: Arc<Mutex<SkillRegistry>>,
}

impl Engine {
    pub async fn new(koad_home: &str, sqlite_path: &str) -> anyhow::Result<Self> {
        let config_local = KoadConfig::load()?;
        let redis_ptr = Arc::new(RedisClient::new(koad_home, true).await?);
        let redis = redis_ptr.clone();
        let config_manager = Arc::new(ConfigManager::new(redis.clone(), config_local.clone()));

        // Seed the "Hot Config" into Redis
        config_manager.seed().await?;

        let storage = Arc::new(KoadStorageBridge::new(
            redis.clone(),
            sqlite_path,
            config_local.storage.drain_interval_secs,
        )?);
        let asm = Arc::new(AgentSessionManager::new(storage.clone(), Arc::new(config_local.clone())));
        let curator = Arc::new(curator::CognitiveCurator::new(redis.clone(), storage.clone()));
        let backup = Arc::new(backup::KoadBackupManager::new(Arc::new(config_local.clone())));
        let context_cache = Arc::new(KoadContextCache::new(
            redis.clone(),
            Arc::new(config_local.clone()),
        ));
        let hydration = Arc::new(hydration::KoadHydrationManager::new(
            redis.clone(),
            Arc::new(config_local.clone()),
        ));
        let identity = Arc::new(KAILeaseManager::new(
            storage.clone(),
            Arc::new(config_local.clone()),
        ));
        let (signal_manager, _) = SignalManager::new(storage.clone());
        let signal = Arc::new(signal_manager);
        let kcm = Arc::new(KoadComplianceManager::new(storage.clone()));
        let skill_registry = Arc::new(Mutex::new(SkillRegistry::new()));

        let diagnostics = Arc::new(ShipDiagnostics::new(
            redis.clone(),
            storage.clone(),
            identity.clone(),
            curator.clone(),
            skill_registry.clone(),
            Arc::new(config_local.clone()),
        ));

        Ok(Self {
            config: config_local,
            redis: redis.clone(),
            config_manager,
            storage,
            diagnostics,
            asm,
            curator,
            backup,
            context_cache,
            hydration,
            identity,
            signal,
            kcm,
            skill_registry,
        })
    }
}
