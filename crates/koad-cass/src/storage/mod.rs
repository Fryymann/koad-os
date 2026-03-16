//! CASS Storage Layer

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use koad_proto::cass::v1::{EpisodicMemory, FactCard};
use rusqlite::{params, Connection};
use std::sync::Arc;
use tokio::sync::Mutex;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn commit_fact(&self, fact: FactCard) -> Result<()>;
    async fn query_facts(&self, domain: &str, tags: &[String], limit: u32) -> Result<Vec<FactCard>>;
    async fn query_agent_facts(
        &self,
        agent_name: &str,
        limit: u32,
        task_id: Option<&str>,
    ) -> Result<Vec<FactCard>>;
    async fn record_episode(&self, episode: EpisodicMemory) -> Result<()>;
    async fn query_recent_episodes(
        &self,
        agent_name: &str,
        limit: u32,
        task_id: Option<&str>,
    ) -> Result<Vec<EpisodicMemory>>;
}

pub struct CassStorage {
    conn: Arc<Mutex<Connection>>,
}

impl CassStorage {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute("CREATE TABLE IF NOT EXISTS fact_cards (id TEXT PRIMARY KEY, source_agent TEXT NOT NULL, session_id TEXT NOT NULL, domain TEXT NOT NULL, content TEXT NOT NULL, confidence REAL NOT NULL, tags TEXT NOT NULL, created_at TEXT NOT NULL, task_ids TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS episodic_memories (session_id TEXT PRIMARY KEY, project_path TEXT NOT NULL, summary TEXT NOT NULL, turn_count INTEGER NOT NULL, timestamp TEXT NOT NULL, task_ids TEXT NOT NULL)", [])?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }
}

#[async_trait]
impl Storage for CassStorage {
    async fn commit_fact(&self, fact: FactCard) -> Result<()> {
        let conn = self.conn.lock().await;
        let tags = fact.tags.join(",");
        let created_at = Utc::now().to_rfc3339();
        // Placeholder for task_ids in fact_cards - we'll leave it NULL for now as it's not in the proto
        conn.execute("INSERT OR REPLACE INTO fact_cards (id, source_agent, session_id, domain, content, confidence, tags, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)", params![fact.id, fact.source_agent, fact.session_id, fact.domain, fact.content, fact.confidence, tags, created_at])?;
        Ok(())
    }

    async fn query_facts(
        &self,
        domain: &str,
        _tags: &[String],
        limit: u32,
    ) -> Result<Vec<FactCard>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare("SELECT id, source_agent, session_id, domain, content, confidence, tags, created_at FROM fact_cards WHERE domain = ?1 ORDER BY confidence DESC LIMIT ?2")?;
        let rows = stmt.query_map(params![domain, limit], |row| {
            Ok(FactCard {
                id: row.get(0)?,
                source_agent: row.get(1)?,
                session_id: row.get(2)?,
                domain: row.get(3)?,
                content: row.get(4)?,
                confidence: row.get(5)?,
                tags: row
                    .get::<_, String>(6)?
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                created_at: None,
            })
        })?;
        let mut facts = Vec::new();
        for row in rows {
            facts.push(row?);
        }
        Ok(facts)
    }

    async fn query_agent_facts(
        &self,
        agent_name: &str,
        limit: u32,
        task_id: Option<&str>,
    ) -> Result<Vec<FactCard>> {
        let conn = self.conn.lock().await;
        let mut facts = Vec::new();

        if let Some(tid) = task_id.filter(|t| !t.is_empty()) {
            let mut stmt = conn.prepare("SELECT id, source_agent, session_id, domain, content, confidence, tags, created_at FROM fact_cards WHERE source_agent = ?1 AND task_ids LIKE '%' || ?2 || '%' ORDER BY confidence DESC LIMIT ?3")?;
            let rows = stmt.query_map(params![agent_name, tid, limit], |row| {
                Ok(FactCard {
                    id: row.get(0)?,
                    source_agent: row.get(1)?,
                    session_id: row.get(2)?,
                    domain: row.get(3)?,
                    content: row.get(4)?,
                    confidence: row.get(5)?,
                    tags: row.get::<_, String>(6)?.split(',').map(|s| s.to_string()).collect(),
                    created_at: None,
                })
            })?;
            for row in rows { facts.push(row?); }
        } else {
            let mut stmt = conn.prepare("SELECT id, source_agent, session_id, domain, content, confidence, tags, created_at FROM fact_cards WHERE source_agent = ?1 ORDER BY confidence DESC LIMIT ?2")?;
            let rows = stmt.query_map(params![agent_name, limit], |row| {
                Ok(FactCard {
                    id: row.get(0)?,
                    source_agent: row.get(1)?,
                    session_id: row.get(2)?,
                    domain: row.get(3)?,
                    content: row.get(4)?,
                    confidence: row.get(5)?,
                    tags: row.get::<_, String>(6)?.split(',').map(|s| s.to_string()).collect(),
                    created_at: None,
                })
            })?;
            for row in rows { facts.push(row?); }
        }
        
        Ok(facts)
    }

    async fn record_episode(&self, episode: EpisodicMemory) -> Result<()> {
        let conn = self.conn.lock().await;
        let task_ids = episode.task_ids.join(",");
        let timestamp = Utc::now().to_rfc3339();
        conn.execute("INSERT OR REPLACE INTO episodic_memories (session_id, project_path, summary, turn_count, timestamp, task_ids) VALUES (?1, ?2, ?3, ?4, ?5, ?6)", params![episode.session_id, episode.project_path, episode.summary, episode.turn_count, timestamp, task_ids])?;
        Ok(())
    }

    async fn query_recent_episodes(
        &self,
        _agent_name: &str,
        limit: u32,
        task_id: Option<&str>,
    ) -> Result<Vec<EpisodicMemory>> {
        let conn = self.conn.lock().await;
        let mut episodes = Vec::new();

        if let Some(tid) = task_id.filter(|t| !t.is_empty()) {
            let mut stmt = conn.prepare("SELECT session_id, project_path, summary, turn_count, timestamp, task_ids FROM episodic_memories WHERE task_ids LIKE '%' || ?1 || '%' ORDER BY timestamp DESC LIMIT ?2")?;
            let rows = stmt.query_map(params![tid, limit], |row| {
                Ok(EpisodicMemory {
                    session_id: row.get(0)?,
                    project_path: row.get(1)?,
                    summary: row.get(2)?,
                    turn_count: row.get(3)?,
                    timestamp: None,
                    task_ids: row.get::<_, String>(5)?.split(',').map(|s| s.to_string()).collect(),
                })
            })?;
            for row in rows { episodes.push(row?); }
        } else {
            let mut stmt = conn.prepare("SELECT session_id, project_path, summary, turn_count, timestamp, task_ids FROM episodic_memories ORDER BY timestamp DESC LIMIT ?1")?;
            let rows = stmt.query_map(params![limit], |row| {
                Ok(EpisodicMemory {
                    session_id: row.get(0)?,
                    project_path: row.get(1)?,
                    summary: row.get(2)?,
                    turn_count: row.get(3)?,
                    timestamp: None,
                    task_ids: row.get::<_, String>(5)?.split(',').map(|s| s.to_string()).collect(),
                })
            })?;
            for row in rows { episodes.push(row?); }
        }
        
        Ok(episodes)
    }
}
pub mod mock;

#[cfg(test)]
mod tests {
    use super::*;
    use koad_proto::cass::v1::EpisodicMemory;

    #[tokio::test]
    async fn test_storage_filters_by_task() -> Result<()> {
        let storage = CassStorage::new(":memory:")?;
        
        // 1. Record episodes with different tasks
        storage.record_episode(EpisodicMemory {
            session_id: "S1".to_string(),
            project_path: "/root".to_string(),
            summary: "Task A work".to_string(),
            turn_count: 5,
            timestamp: None,
            task_ids: vec!["task-a".to_string()],
        }).await?;

        storage.record_episode(EpisodicMemory {
            session_id: "S2".to_string(),
            project_path: "/root".to_string(),
            summary: "Task B work".to_string(),
            turn_count: 5,
            timestamp: None,
            task_ids: vec!["task-b".to_string()],
        }).await?;

        // 2. Query filtered by Task A
        let results = storage.query_recent_episodes("tyr", 10, Some("task-a")).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].session_id, "S1");

        // 3. Query unfiltered
        let all = storage.query_recent_episodes("tyr", 10, None).await?;
        assert_eq!(all.len(), 2);

        Ok(())
    }
}
