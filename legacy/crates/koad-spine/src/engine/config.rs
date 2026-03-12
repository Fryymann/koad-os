use koad_core::utils::redis::RedisClient;
use fred::interfaces::PubsubInterface;
use fred::prelude::*;
use koad_core::config::KoadConfig;
use koad_core::constants::REDIS_KEY_CONFIG;
use std::sync::Arc;
use tracing::info;

pub struct ConfigManager {
    redis: Arc<RedisClient>,
    initial_config: KoadConfig,
}

impl ConfigManager {
    pub fn new(redis: Arc<RedisClient>, config: KoadConfig) -> Self {
        Self {
            redis,
            initial_config: config,
        }
    }

    /// Seeds the local configuration into Redis if it doesn't already exist.
    pub async fn seed(&self) -> anyhow::Result<()> {
        let exists: bool = self.redis.pool.next().exists(REDIS_KEY_CONFIG).await?;
        if !exists {
            let json = self.initial_config.to_json()?;
            let _: () = self
                .redis
                .pool
                .next()
                .set(REDIS_KEY_CONFIG, json, None, None, false)
                .await?;
            info!("ConfigManager: Hot Config seeded in Redis.");
        } else {
            info!("ConfigManager: Hot Config already present in Redis.");
        }
        Ok(())
    }

    /// Retrieves the current "Hot Config" from Redis.
    pub async fn get_config(&self) -> anyhow::Result<KoadConfig> {
        let json: Option<String> = self.redis.pool.next().get(REDIS_KEY_CONFIG).await?;
        match json {
            Some(j) => KoadConfig::from_json(&j),
            None => Ok(self.initial_config.clone()),
        }
    }

    /// Updates the "Hot Config" in Redis and broadcasts a change event.
    pub async fn update_config(&self, config: KoadConfig) -> anyhow::Result<()> {
        let json = config.to_json()?;
        let _: () = self
            .redis
            .pool
            .next()
            .set(REDIS_KEY_CONFIG, json, None, None, false)
            .await?;

        // Broadcast the update via PubSub for reactive clients
        let _: () = self
            .redis
            .pool
            .next()
            .publish("koad:config:updates", "REFRESH")
            .await?;

        info!("ConfigManager: Hot Config updated in Redis.");
        Ok(())
    }
}
