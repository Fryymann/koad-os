use crate::engine::redis::RedisClient;
use async_trait::async_trait;
use chrono::Utc;
use fred::interfaces::HashesInterface;
use koad_core::storage::StorageBridge;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

pub struct KoadStorageBridge {
    pub redis: Arc<RedisClient>,
    pub sqlite: Arc<Mutex<Connection>>,
    drain_interval: Duration,
}

impl KoadStorageBridge {
    pub fn new(redis: Arc<RedisClient>, sqlite_path: &str) -> anyhow::Result<Self> {
        let conn = Connection::open(sqlite_path)?;
        // Enable WAL mode
        let _: String = conn.query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))?;

        // Ensure state table exists for arbitrary key-value storage
        conn.execute(
            "CREATE TABLE IF NOT EXISTS state_ledger (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            redis,
            sqlite: Arc::new(Mutex::new(conn)),
            drain_interval: Duration::from_secs(30),
        })
    }

    /// Starts the background task that "drains" volatile metrics into the database.
    pub async fn start_drain_loop(&self) {
        println!(
            "StorageBridge: Drain loop active (Interval: {:?}).",
            self.drain_interval
        );
        loop {
            sleep(self.drain_interval).await;
            if let Err(e) = self.drain_all().await {
                eprintln!(
                    "StorageBridge Error: Failed to drain state to SQLite: {}",
                    e
                );
            }
        }
    }

    pub async fn get_identity_bio(&self, name: &str) -> anyhow::Result<Option<String>> {
        let sqlite = self.sqlite.clone();
        let name_str = name.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            let mut stmt = conn.prepare("SELECT bio FROM identities WHERE id = ?1 OR name = ?1")?;
            let mut rows = stmt.query(params![name_str])?;
            if let Some(row) = rows.next()? {
                Ok::<Option<String>, anyhow::Error>(Some(row.get(0)?))
            } else {
                Ok::<Option<String>, anyhow::Error>(None)
            }
        })
        .await?
    }
}

const SOVEREIGN_KEYS: &[&str] = &[
    "identities",
    "identity_roles",
    "knowledge",
    "principles",
    "canon_rules",
];

#[async_trait]
impl StorageBridge for KoadStorageBridge {
    async fn set_state(
        &self,
        key: &str,
        value: Value,
        caller_tier: Option<i32>,
    ) -> anyhow::Result<()> {
        let val_str = value.to_string();
        let now = Utc::now().timestamp();
        let tier = caller_tier.unwrap_or(3); // Default to restricted Guest

        // CIP: Cognitive Integrity Protocol Enforcement
        if tier > 1 {
            for sovereign in SOVEREIGN_KEYS {
                if key.starts_with(sovereign) {
                    anyhow::bail!("Cognitive Protection: Model Tier {} is not authorized to modify sovereign state '{}'.", tier, key);
                }
            }
        }

        // 1. Update Redis (Hot Path)
        let _: () = self
            .redis
            .pool
            .hset::<(), _, _>("koad:state", (key, val_str.clone()))
            .await?;

        // 2. Immediate persistent update for critical state (can be moved to drain later)
        let sqlite = self.sqlite.clone();
        let key_clone = key.to_string();
        tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            conn.execute(
                "INSERT INTO state_ledger (key, value, updated_at) 
                 VALUES (?1, ?2, ?3) 
                 ON CONFLICT(key) DO UPDATE SET value=?2, updated_at=?3",
                params![key_clone, val_str, now],
            )
        })
        .await??;

        Ok(())
    }

    async fn get_state(&self, key: &str) -> anyhow::Result<Option<Value>> {
        // 1. Check Redis (Hot Path)
        let res: Option<String> = self.redis.pool.hget("koad:state", key).await?;
        if let Some(s) = res {
            return Ok(Some(serde_json::from_str(&s)?));
        }

        // 2. Fallback to SQLite (Cold Path)
        let sqlite = self.sqlite.clone();
        let key_clone = key.to_string();
        let res: Option<String> = tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            let mut stmt = conn.prepare("SELECT value FROM state_ledger WHERE key = ?1")?;
            let mut rows = stmt.query(params![key_clone])?;
            if let Some(row) = rows.next()? {
                Ok::<Option<String>, anyhow::Error>(Some(row.get(0)?))
            } else {
                Ok::<Option<String>, anyhow::Error>(None)
            }
        })
        .await??;

        if let Some(s) = res {
            let val: Value = serde_json::from_str(&s)?;
            // Hydrate back to Redis
            let _: () = self
                .redis
                .pool
                .hset::<(), _, _>("koad:state", (key, s))
                .await?;
            return Ok(Some(val));
        }

        Ok(None)
    }

    async fn drain_all(&self) -> anyhow::Result<()> {
        // Scan Redis for all keys in 'koad:state' and sync to SQLite
        let redis = self.redis.clone();
        let sqlite = self.sqlite.clone();

        let state: std::collections::HashMap<String, String> =
            redis.pool.hgetall("koad:state").await?;
        let now = Utc::now().timestamp();

        if !state.is_empty() {
            tokio::task::spawn_blocking(move || {
                let mut conn = sqlite.blocking_lock();
                let tx = conn.transaction()?;
                {
                    let mut stmt = tx.prepare(
                        "INSERT INTO state_ledger (key, value, updated_at) 
                         VALUES (?1, ?2, ?3) 
                         ON CONFLICT(key) DO UPDATE SET value=?2, updated_at=?3",
                    )?;
                    for (k, v) in state {
                        stmt.execute(params![k, v, now])?;
                    }
                }
                tx.commit()?;
                Ok::<(), anyhow::Error>(())
            })
            .await??;
        }

        Ok(())
    }

    async fn hydrate_all(&self) -> anyhow::Result<()> {
        let sqlite = self.sqlite.clone();
        let redis = self.redis.clone();

        let entries: Vec<(String, String)> = tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            let mut stmt = conn.prepare("SELECT key, value FROM state_ledger")?;
            let rows = stmt.query_map([], |row: &rusqlite::Row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok::<Vec<(String, String)>, anyhow::Error>(results)
        })
        .await??;

        for (k, v) in entries {
            let _: () = redis.pool.next().hset::<(), _, _>("koad:state", (k, v)).await?;
        }

        // Set lighthouse key
        let _: () = redis.pool.next().hset("koad:state", ("initialized", "true")).await?;

        Ok(())
    }
}
