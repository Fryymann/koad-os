//! L3 — Qdrant-backed semantic storage tier.
//!
//! Facts are stored as 32-dim vector points derived from content hashing.
//! Queries use payload filtering by domain for deterministic retrieval.
//! Semantic vector similarity search is available as a future upgrade path.

use crate::storage::MemoryTier;
use anyhow::{Context, Result};
use async_trait::async_trait;
use koad_proto::cass::v1::{EpisodicMemory, FactCard};
use qdrant_client::qdrant::{
    Condition, CreateCollectionBuilder, Distance, Filter, PointStruct, ScrollPointsBuilder,
    UpsertPointsBuilder, VectorParamsBuilder, value::Kind,
};
use qdrant_client::Qdrant;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

const COLLECTION: &str = "fact_cards";
const VECTOR_DIM: u64 = 32;

pub struct QdrantTier {
    client: Option<Qdrant>,
}

impl QdrantTier {
    /// Create a no-op offline tier for degraded-mode boot (Qdrant unreachable).
    pub fn new_offline() -> Self {
        Self { client: None }
    }

    pub async fn new(url: &str) -> Result<Self> {
        let client = Qdrant::from_url(url)
            .build()
            .context("Failed to build Qdrant client")?;

        if !client.collection_exists(COLLECTION).await? {
            client
                .create_collection(
                    CreateCollectionBuilder::new(COLLECTION)
                        .vectors_config(VectorParamsBuilder::new(VECTOR_DIM, Distance::Cosine)),
                )
                .await
                .context("Failed to create Qdrant collection")?;
        }

        Ok(Self { client: Some(client) })
    }

    /// Deterministic u64 point ID from fact UUID string.
    fn point_id(fact_id: &str) -> u64 {
        let mut h = DefaultHasher::new();
        fact_id.hash(&mut h);
        h.finish()
    }

    /// 32-dim content fingerprint vector derived from content bytes.
    /// Not a real semantic embedding — serves as a stable placeholder
    /// until an embedding model is wired into InferenceRouter.
    fn content_vector(content: &str) -> Vec<f32> {
        let mut vec = vec![0.0f32; VECTOR_DIM as usize];
        for (i, &b) in content.as_bytes().iter().enumerate() {
            vec[i % VECTOR_DIM as usize] += b as f32;
        }
        let mag = (vec.iter().map(|x| x * x).sum::<f32>()).sqrt().max(1e-6);
        vec.iter().map(|x| x / mag).collect()
    }

    fn make_payload(fact: &FactCard) -> HashMap<String, qdrant_client::qdrant::Value> {
        use qdrant_client::qdrant::Value;
        let mut p = HashMap::new();
        p.insert("id".into(),           Value { kind: Some(Kind::StringValue(fact.id.clone())) });
        p.insert("domain".into(),       Value { kind: Some(Kind::StringValue(fact.domain.clone())) });
        p.insert("content".into(),      Value { kind: Some(Kind::StringValue(fact.content.clone())) });
        p.insert("source_agent".into(), Value { kind: Some(Kind::StringValue(fact.source_agent.clone())) });
        p.insert("session_id".into(),   Value { kind: Some(Kind::StringValue(fact.session_id.clone())) });
        p.insert("confidence".into(),   Value { kind: Some(Kind::DoubleValue(fact.confidence as f64)) });
        p.insert("tags".into(),         Value { kind: Some(Kind::StringValue(fact.tags.join(","))) });
        p
    }

    fn payload_to_fact(payload: &HashMap<String, qdrant_client::qdrant::Value>) -> Option<FactCard> {
        let get_str = |key: &str| -> Option<String> {
            match payload.get(key)?.kind.as_ref()? {
                Kind::StringValue(s) => Some(s.clone()),
                _ => None,
            }
        };
        let get_f64 = |key: &str| -> Option<f64> {
            match payload.get(key)?.kind.as_ref()? {
                Kind::DoubleValue(d) => Some(*d),
                _ => None,
            }
        };

        Some(FactCard {
            id:           get_str("id")?,
            domain:       get_str("domain")?,
            content:      get_str("content")?,
            source_agent: get_str("source_agent")?,
            session_id:   get_str("session_id")?,
            confidence:   get_f64("confidence")? as f32,
            tags:         get_str("tags")?.split(',').map(|s| s.to_string()).collect(),
            created_at:   None,
        })
    }
}

#[async_trait]
impl MemoryTier for QdrantTier {
    async fn commit_fact(&self, fact: FactCard) -> Result<()> {
        let Some(client) = &self.client else { return Ok(()); };
        let vector = Self::content_vector(&fact.content);
        let payload = Self::make_payload(&fact);
        let point = PointStruct::new(Self::point_id(&fact.id), vector, payload);

        client
            .upsert_points(UpsertPointsBuilder::new(COLLECTION, vec![point]))
            .await
            .context("QdrantTier: upsert failed")?;
        Ok(())
    }

    async fn query_facts(&self, domain: &str, _tags: &[String], limit: u32) -> Result<Vec<FactCard>> {
        let Some(client) = &self.client else { return Ok(vec![]); };
        let filter = Filter::must([Condition::matches("domain", domain.to_string())]);

        let result = client
            .scroll(
                ScrollPointsBuilder::new(COLLECTION)
                    .filter(filter)
                    .limit(limit)
                    .with_payload(true),
            )
            .await
            .context("QdrantTier: scroll failed")?;

        let facts = result
            .result
            .iter()
            .filter_map(|p| Self::payload_to_fact(&p.payload))
            .collect();

        Ok(facts)
    }

    async fn query_agent_facts(&self, _agent_name: &str, _limit: u32, _task_id: Option<&str>) -> Result<Vec<FactCard>> {
        // Qdrant tier defers agent-scoped queries to SQLite (L2).
        Ok(vec![])
    }

    async fn record_episode(&self, _episode: EpisodicMemory) -> Result<()> {
        // Episodes are structural records — persisted in SQLite (L2) only.
        Ok(())
    }

    async fn query_recent_episodes(&self, _agent_name: &str, _limit: u32, _task_id: Option<&str>) -> Result<Vec<EpisodicMemory>> {
        Ok(vec![])
    }
}
