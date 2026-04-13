//! CASS Storage Layer — Tiered Memory Stack (L1-L4)
//!
//! - L1: `RedisTier`   — hot cache, TTL-bounded, fast reads
//! - L2: `SqliteTier`  — durable episodic store, authoritative for episodes & agent queries
//! - L3: `QdrantTier`  — semantic vector index, domain-filtered retrieval
//! - Orchestrator: `TieredStorage` — routes reads/writes across tiers

use anyhow::Result;
use async_trait::async_trait;
use koad_proto::cass::v1::{EpisodicMemory, FactCard, Pulse};

#[cfg(test)]
pub mod mock;
pub mod qdrant_tier;
pub mod redis_tier;
pub mod sqlite_tier;
pub mod tiered;

pub use qdrant_tier::QdrantTier;
pub use redis_tier::RedisTier;
pub use sqlite_tier::SqliteTier;
pub use tiered::TieredStorage;

/// Backward-compatible alias — existing code using `CassStorage` continues to compile.
pub type CassStorage = SqliteTier;

/// Core trait for all memory storage tiers.
#[async_trait]
pub trait MemoryTier: Send + Sync {
    async fn commit_fact(&self, fact: FactCard) -> Result<()>;
    async fn query_facts(&self, domain: &str, tags: &[String], limit: u32)
        -> Result<Vec<FactCard>>;
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

/// Backward-compatible alias for the storage trait.
pub use MemoryTier as Storage;

/// Trait for pulse storage — implemented by RedisTier only (L1 hot signals).
#[async_trait]
pub trait PulseTier: Send + Sync {
    async fn add_pulse(&self, pulse: Pulse) -> Result<()>;
    async fn get_active_pulses(&self, role: &str) -> Result<Vec<Pulse>>;
}
