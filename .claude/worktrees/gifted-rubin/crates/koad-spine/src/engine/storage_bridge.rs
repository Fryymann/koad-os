use koad_core::utils::redis::RedisClient;
use koad_core::intelligence::FactCard;
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

        // Ensure knowledge table exists (Legacy/CLI Durable Memory)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT,
                content TEXT,
                tags TEXT,
                origin_agent TEXT,
                timestamp TEXT
            )",
            [],
        )?;

        // Ensure identity_snapshots table exists
        conn.execute(
            "CREATE TABLE IF NOT EXISTS identity_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                trigger TEXT NOT NULL,
                notes TEXT,
                created_at TEXT NOT NULL,
                origin_agent TEXT NOT NULL
            )",
            [],
        )?;

        // Ensure intelligence_bank table exists (L3 Durable Memory)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS intelligence_bank (
                id TEXT PRIMARY KEY,
                source_agent TEXT NOT NULL,
                session_id TEXT NOT NULL,
                domain TEXT NOT NULL,
                content TEXT NOT NULL,
                confidence REAL NOT NULL,
                tags TEXT,
                created_at INTEGER NOT NULL,
                ttl_seconds INTEGER NOT NULL
            )",
            [],
        )?;

        // Ensure context_snapshots table exists (Rolling Quicksaves)
        conn.execute(
            "CREATE TABLE IF NOT EXISTS context_snapshots (
                id TEXT PRIMARY KEY,
                agent_name TEXT NOT NULL,
                session_id TEXT NOT NULL,
                snapshot_json TEXT NOT NULL,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        Ok(Self {
            redis,
            sqlite: Arc::new(Mutex::new(conn)),
            drain_interval: Duration::from_secs(30),
        })
    }

    pub async fn save_fact(&self, fact: FactCard) -> anyhow::Result<()> {
        let sqlite = self.sqlite.clone();
        let tags_json = serde_json::to_string(&fact.tags).unwrap_or_default();
        let created_at = fact.created_at.timestamp();

        tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            conn.execute(
                "INSERT OR REPLACE INTO intelligence_bank 
                 (id, source_agent, session_id, domain, content, confidence, tags, created_at, ttl_seconds) 
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    fact.id.to_string(),
                    fact.source_agent,
                    fact.session_id,
                    fact.domain,
                    fact.content,
                    fact.confidence,
                    tags_json,
                    created_at,
                    fact.ttl_seconds
                ],
            )
        })
        .await??;
        Ok(())
    }

    pub async fn save_knowledge(
        &self,
        category: &str,
        content: &str,
        tags: Option<String>,
        agent: &str,
    ) -> anyhow::Result<()> {
        let sqlite = self.sqlite.clone();
        let cat = category.to_string();
        let text = content.to_string();
        let t = tags;
        let origin = agent.to_string();
        let now = Utc::now().to_rfc3339();

        tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            conn.execute(
                "INSERT INTO knowledge (category, content, tags, timestamp, origin_agent) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![cat, text, t, now, origin],
            )
        })
        .await??;
        Ok(())
    }

    pub async fn save_context_snapshot(
        &self,
        agent_name: &str,
        session_id: &str,
        snapshot_json: String,
    ) -> anyhow::Result<()> {
        let sqlite = self.sqlite.clone();
        let name = agent_name.to_string();
        let sid = session_id.to_string();
        let now = Utc::now().timestamp();
        let id = uuid::Uuid::new_v4().to_string();

        tokio::task::spawn_blocking(move || {
            let mut conn = sqlite.blocking_lock();
            let tx = conn.transaction()?;

            // 1. Insert new snapshot
            tx.execute(
                "INSERT INTO context_snapshots (id, agent_name, session_id, snapshot_json, created_at) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, name, sid, snapshot_json, now],
            )?;

            // 2. Enforce retention (Keep last 2 per agent)
            tx.execute(
                "DELETE FROM context_snapshots 
                 WHERE agent_name = ?1 AND id NOT IN (
                    SELECT id FROM context_snapshots 
                    WHERE agent_name = ?1 
                    ORDER BY created_at DESC LIMIT 2
                 )",
                params![name],
            )?;

            tx.commit()?;
            Ok::<(), anyhow::Error>(())
        })
        .await??;
        Ok(())
    }

    pub async fn query_facts(&self, query: &str, limit: usize) -> anyhow::Result<Vec<FactCard>> {
        let sqlite = self.sqlite.clone();
        let query_str = format!("%{}%", query);
        
        tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            let mut stmt = conn.prepare(
                "SELECT id, source_agent, session_id, domain, content, confidence, tags, created_at, ttl_seconds 
                 FROM intelligence_bank 
                 WHERE content LIKE ?1 OR domain LIKE ?1 OR tags LIKE ?1 
                 ORDER BY created_at DESC LIMIT ?2"
            )?;
            
            let fact_iter = stmt.query_map(params![query_str, limit as i64], |row| {
                let id_str: String = row.get(0)?;
                let tags_json: String = row.get(6)?;
                let created_at_ts: i64 = row.get(7)?;
                
                Ok(FactCard {
                    id: uuid::Uuid::parse_str(&id_str).unwrap_or_default(),
                    source_agent: row.get(1)?,
                    session_id: row.get(2)?,
                    domain: row.get(3)?,
                    content: row.get(4)?,
                    confidence: row.get(5)?,
                    tags: serde_json::from_str(&tags_json).unwrap_or_default(),
                    created_at: chrono::DateTime::from_timestamp(created_at_ts, 0).unwrap_or_default().with_timezone(&Utc),
                    ttl_seconds: row.get(8)?,
                })
            })?;
            
            let mut results = Vec::new();
            for fact in fact_iter {
                results.push(fact?);
            }
            Ok(results)
        })
        .await?
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
            let rows = stmt.query_map([], |row: &rusqlite::Row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut results = Vec::new();
            for row in rows {
                results.push(row?);
            }
            Ok::<Vec<(String, String)>, anyhow::Error>(results)
        })
        .await??;

        for (k, v) in entries {
            let _: () = redis
                .pool
                .next()
                .hset::<(), _, _>("koad:state", (k, v))
                .await?;
        }

        // Set lighthouse key
        let _: () = redis
            .pool
            .next()
            .hset("koad:state", ("initialized", "true"))
            .await?;

        Ok(())
    }
}
