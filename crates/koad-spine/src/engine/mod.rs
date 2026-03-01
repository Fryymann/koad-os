pub mod redis;
pub mod persistence;
pub mod diagnostics;
#[cfg(test)]
mod tests;

use std::sync::Arc;
use crate::engine::redis::RedisClient;
use crate::engine::persistence::PersistenceManager;
use crate::engine::diagnostics::ShipDiagnostics;

pub struct Engine {
    pub redis: Arc<RedisClient>,
    pub persistence: Arc<PersistenceManager>,
    pub diagnostics: Arc<ShipDiagnostics>,
}

impl Engine {
    pub async fn new(redis_config_path: &str, sqlite_path: &str) -> anyhow::Result<Self> {
        let redis = Arc::new(RedisClient::new(redis_config_path).await?);
        let persistence = Arc::new(PersistenceManager::new(redis.clone(), sqlite_path)?);
        let diagnostics = Arc::new(ShipDiagnostics::new(redis.clone()));
        
        Ok(Self {
            redis,
            persistence,
            diagnostics,
        })
    }
}
