//! L1 — Redis-backed hot cache tier.
//!
//! Facts are stored as JSON strings with a 1-hour TTL.
//! Domain membership is tracked via Redis sets for O(1) lookup.

use crate::storage::MemoryTier;
use anyhow::Result;
use async_trait::async_trait;
use fred::clients::RedisPool;
use fred::interfaces::{KeysInterface, SetsInterface};
use koad_proto::cass::v1::{EpisodicMemory, FactCard};
use tracing::warn;

pub struct RedisTier {
    pool: RedisPool,
}

impl RedisTier {
    pub fn new(pool: RedisPool) -> Self {
        Self { pool }
    }

    fn fact_key(id: &str) -> String {
        format!("cass:fact:{}", id)
    }

    fn domain_set_key(domain: &str) -> String {
        format!("cass:domain:{}", domain)
    }

    fn fact_to_json(fact: &FactCard) -> String {
        serde_json::json!({
            "id": fact.id,
            "domain": fact.domain,
            "content": fact.content,
            "source_agent": fact.source_agent,
            "session_id": fact.session_id,
            "confidence": fact.confidence,
            "tags": fact.tags.join(","),
        })
        .to_string()
    }

    fn json_to_fact(json: &str) -> Option<FactCard> {
        let v: serde_json::Value = serde_json::from_str(json).ok()?;
        Some(FactCard {
            id: v["id"].as_str()?.to_string(),
            domain: v["domain"].as_str()?.to_string(),
            content: v["content"].as_str()?.to_string(),
            source_agent: v["source_agent"].as_str()?.to_string(),
            session_id: v["session_id"].as_str()?.to_string(),
            confidence: v["confidence"].as_f64()? as f32,
            tags: v["tags"].as_str()?.split(',').map(|s| s.to_string()).collect(),
            created_at: None,
        })
    }
}

const TTL: i64 = 3600;

#[async_trait]
impl MemoryTier for RedisTier {
    async fn commit_fact(&self, fact: FactCard) -> Result<()> {
        let client = self.pool.next();
        let json = Self::fact_to_json(&fact);
        let fact_key = Self::fact_key(&fact.id);
        let domain_key = Self::domain_set_key(&fact.domain);

        client.set::<(), _, _>(&fact_key, &json, Some(fred::types::Expiration::EX(TTL)), None, false).await?;
        client.sadd::<(), _, _>(&domain_key, fact.id.as_str()).await?;
        client.expire::<(), _>(&domain_key, TTL).await?;
        Ok(())
    }

    async fn query_facts(&self, domain: &str, _tags: &[String], limit: u32) -> Result<Vec<FactCard>> {
        let client = self.pool.next();
        let domain_key = Self::domain_set_key(domain);

        let ids: Vec<String> = client.smembers(&domain_key).await.unwrap_or_default();
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let mut facts = Vec::new();
        for id in ids.iter().take(limit as usize) {
            let json: Option<String> = client.get(Self::fact_key(id)).await.unwrap_or(None);
            if let Some(json) = json {
                match Self::json_to_fact(&json) {
                    Some(fact) => facts.push(fact),
                    None => warn!(id = %id, "RedisTier: Failed to deserialize fact"),
                }
            }
        }

        facts.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
        Ok(facts)
    }

    async fn query_agent_facts(&self, _agent_name: &str, _limit: u32, _task_id: Option<&str>) -> Result<Vec<FactCard>> {
        // Redis tier does not index by agent; L2 (SQLite) is authoritative for agent queries.
        Ok(vec![])
    }

    async fn record_episode(&self, _episode: EpisodicMemory) -> Result<()> {
        // Redis tier does not persist episodes.
        Ok(())
    }

    async fn query_recent_episodes(&self, _agent_name: &str, _limit: u32, _task_id: Option<&str>) -> Result<Vec<EpisodicMemory>> {
        Ok(vec![])
    }
}
