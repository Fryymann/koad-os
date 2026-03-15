//! CASS Storage Layer

use anyhow::{Context, Result};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use koad_proto::cass::v1::{FactCard, EpisodicMemory};
use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn commit_fact(&self, fact: FactCard) -> Result<()>;
    async fn query_facts(&self, domain: &str, tags: &[String], limit: u32) -> Result<Vec<FactCard>>;
    async fn record_episode(&self, episode: EpisodicMemory) -> Result<()>;
}

pub struct CassStorage {
    conn: Arc<Mutex<Connection>>,
}

impl CassStorage {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute("CREATE TABLE IF NOT EXISTS fact_cards (id TEXT PRIMARY KEY, source_agent TEXT NOT NULL, session_id TEXT NOT NULL, domain TEXT NOT NULL, content TEXT NOT NULL, confidence REAL NOT NULL, tags TEXT NOT NULL, created_at TEXT NOT NULL)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS episodic_memories (session_id TEXT PRIMARY KEY, project_path TEXT NOT NULL, summary TEXT NOT NULL, turn_count INTEGER NOT NULL, timestamp TEXT NOT NULL, task_ids TEXT NOT NULL)", [])?;
        Ok(Self { conn: Arc::new(Mutex::new(conn)) })
    }
}

#[async_trait]
impl Storage for CassStorage {
    async fn commit_fact(&self, fact: FactCard) -> Result<()> {
        let conn = self.conn.lock().await;
        let tags = fact.tags.join(",");
        let created_at = Utc::now().to_rfc3339();
        conn.execute("INSERT OR REPLACE INTO fact_cards (id, source_agent, session_id, domain, content, confidence, tags, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![fact.id, fact.source_agent, fact.session_id, fact.domain, fact.content, fact.confidence, tags, created_at])?;
        Ok(())
    }

    async fn query_facts(&self, domain: &str, _tags: &[String], limit: u32) -> Result<Vec<FactCard>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, source_agent, session_id, domain, content, confidence, tags, created_at FROM fact_cards WHERE domain = ?1 LIMIT ?2")?;
        let rows = stmt.query_map(params![domain, limit], |row| {
            Ok(FactCard { id: row.get(0)?, source_agent: row.get(1)?, session_id: row.get(2)?, domain: row.get(3)?, content: row.get(4)?, confidence: row.get(5)?, tags: row.get::<_, String>(6)?.split(',').map(|s| s.to_string()).collect(), created_at: None })
        })?;
        let mut facts = Vec::new();
        for row in rows { facts.push(row?); }
        Ok(facts)
    }

    async fn record_episode(&self, episode: EpisodicMemory) -> Result<()> {
        let conn = self.conn.lock().await;
        let task_ids = episode.task_ids.join(",");
        let timestamp = Utc::now().to_rfc3339();
        conn.execute("INSERT OR REPLACE INTO episodic_memories (session_id, project_path, summary, turn_count, timestamp, task_ids) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![episode.session_id, episode.project_path, episode.summary, episode.turn_count, timestamp, task_ids])?;
        Ok(())
    }
}
pub mod mock;
