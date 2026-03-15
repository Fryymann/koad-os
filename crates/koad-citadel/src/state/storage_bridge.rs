//! Citadel Storage Bridge
//!
//! Implements the CQRS dual-store pattern for the Citadel.

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use fred::interfaces::{HashesInterface, ClientLike};
use fred::types::{CustomCommand, ClusterHash};
use koad_core::storage::StorageBridge;
use koad_core::utils::redis::RedisClient;
use rusqlite::params;
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::{error, info};

use crate::auth::sanctuary;

/// Citadel storage bridge implementing the CQRS dual-store pattern.
pub struct CitadelStorageBridge {
    pub redis: Arc<RedisClient>,
    pub sqlite: Arc<Mutex<rusqlite::Connection>>,
    drain_interval: Duration,
}

impl CitadelStorageBridge {
    pub fn new(
        redis: Arc<RedisClient>,
        sqlite_path: &str,
        drain_interval_secs: u64,
    ) -> Result<Self> {
        let conn = rusqlite::Connection::open(sqlite_path)
            .with_context(|| format!("Failed to open Citadel DB at {}", sqlite_path))?;

        let _: String = conn.query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))
            .context("Failed to enable WAL mode")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS state_ledger (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        ).context("Failed to create state_ledger table")?;

        Ok(Self {
            redis,
            sqlite: Arc::new(Mutex::new(conn)),
            drain_interval: Duration::from_secs(drain_interval_secs),
        })
    }

    pub async fn start_drain_loop(&self) {
        info!("StorageBridge: Drain loop started (interval: {:?})", self.drain_interval);
        loop {
            if let Err(e) = self.drain_all().await {
                error!("StorageBridge: Drain failed: {}", e);
            }
            sleep(self.drain_interval).await;
        }
    }

    pub async fn enable_keyspace_notifications(&self) -> Result<()> {
        // Use ClusterHash::Random as a safe default for non-key-specific command
        let cmd = CustomCommand::new("CONFIG", ClusterHash::Random, false);
        let args = vec!["SET", "notify-keyspace-events", "KEA"];
        let _: () = self.redis.pool.custom(cmd, args).await
            .context("Failed to enable Redis keyspace notifications")?;
        info!("StorageBridge: Keyspace notifications enabled (KEA)");
        Ok(())
    }
}

#[async_trait]
impl StorageBridge for CitadelStorageBridge {
    async fn set_state(&self, key: &str, value: Value, caller_tier: Option<i32>) -> Result<()> {
        sanctuary::check_protected_key(key, caller_tier)
            .map_err(|e| anyhow::anyhow!("Permission denied: {}", e.message()))?;

        let payload = serde_json::to_string(&value).context("Failed to serialize state value")?;
        let _: () = self.redis.pool.hset("koad:state", (key, &payload)).await
            .context("Redis HSET failed")?;
        Ok(())
    }

    async fn get_state(&self, key: &str) -> Result<Option<Value>> {
        let cached: Option<String> = self.redis.pool.hget("koad:state", key).await
            .context("Redis HGET failed")?;

        if let Some(data) = cached {
            return Ok(Some(serde_json::from_str(&data).context("Failed to parse L1 state")?));
        }

        let sqlite = self.sqlite.lock().await;
        let result: Option<String> = sqlite.query_row(
                "SELECT value FROM state_ledger WHERE key = ?1",
                params![key.to_string()],
                |row| row.get(0),
            ).ok();

        if let Some(ref data) = result {
            let _: () = self.redis.pool.hset("koad:state", (key, data)).await
                .context("Failed to repopulate L1 cache")?;
            return Ok(Some(serde_json::from_str(data).context("Failed to parse L2 state")?));
        }
        Ok(None)
    }

    async fn drain_all(&self) -> Result<()> {
        let all_state: std::collections::HashMap<String, String> = self.redis.pool.hgetall("koad:state").await
            .context("Redis HGETALL failed during drain")?;

        if all_state.is_empty() { return Ok(()); }

        let sqlite = self.sqlite.lock().await;
        let now = Utc::now().timestamp();
        let tx = sqlite.unchecked_transaction().context("Failed to start L2 drain transaction")?;

        for (key, value) in &all_state {
            tx.execute(
                "INSERT OR REPLACE INTO state_ledger (key, value, updated_at) VALUES (?1, ?2, ?3)",
                params![key, value, now],
            ).context("L2 drain insert failed")?;
        }
        tx.commit().context("Failed to commit L2 drain")?;
        Ok(())
    }

    async fn hydrate_all(&self) -> Result<()> {
        info!("StorageBridge: Full hydration from SQLite...");
        let data = {
            let sqlite = self.sqlite.lock().await;
            let mut stmt = sqlite.prepare("SELECT key, value FROM state_ledger").context("Failed to prepare hydration statement")?;
            let rows = stmt.query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))).context("Hydration query failed")?;
            let mut collected = Vec::new();
            for row in rows { collected.push(row.context("Failed to read hydration row")?); }
            collected
        };

        for (key, value) in data {
            let _: () = self.redis.pool.hset("koad:state", (&key, &value)).await
                .context("Redis HSET failed during hydration")?;
        }
        info!("StorageBridge: Hydration complete.");
        Ok(())
    }
}
