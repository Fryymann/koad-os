//! Tiered Memory Orchestrator — L1 (Redis) → L2 (SQLite) → L3 (Qdrant).
//!
//! Write path: L1 + L2 synchronously; L3 via async enrichment queue.
//! Read path (query_facts): L1 first; fall through to L2 on cache miss.
//! Read path (episodes, agent facts): L2 only (authoritative durable store).

use crate::storage::{MemoryTier, QdrantTier, RedisTier, SqliteTier};
use anyhow::Result;
use async_trait::async_trait;
use koad_proto::cass::v1::{EpisodicMemory, FactCard};
use std::sync::Arc;
use tracing::warn;

pub struct TieredStorage {
    l1: Arc<RedisTier>,
    l2: Arc<SqliteTier>,
    l3: Arc<QdrantTier>,
}

/// Decide the semantic-search outcome from L3's scored candidates.
///
/// `Some(hits)` — candidates existed in the partition, so the vector tier's
/// threshold verdict is final. A `Some(vec![])` means everything scored below
/// `min_score`: precision is honored and there is NO fall-through to the
/// unscored L2 text match.
/// `None` — zero candidates (empty partition, or L3 offline reporting an
/// empty scored set): fall through to the L2 lexical safety net.
fn l3_semantic_verdict(
    scored: Vec<(f32, FactCard)>,
    limit: u32,
    min_score: f32,
) -> Option<Vec<FactCard>> {
    if scored.is_empty() {
        return None;
    }
    Some(QdrantTier::merge_scored(scored, limit, min_score))
}

impl TieredStorage {
    pub fn new(l1: Arc<RedisTier>, l2: Arc<SqliteTier>, l3: Arc<QdrantTier>) -> Self {
        Self { l1, l2, l3 }
    }
}

#[async_trait]
impl MemoryTier for TieredStorage {
    async fn commit_fact(&self, fact: FactCard) -> Result<()> {
        // L1: hot cache write (non-fatal on failure)
        if let Err(e) = self.l1.commit_fact(fact.clone()).await {
            warn!(error = %e, "TieredStorage: L1 write failed, continuing");
        }

        // Capture identifiers needed for the enrichment enqueue before L2 takes ownership.
        let fact_id = fact.id.clone();
        let source_agent = fact.source_agent.clone();

        // L2: durable write (authoritative)
        self.l2.commit_fact(fact).await?;

        // L3 indexing is async: the enrichment worker embeds + upserts Qdrant.
        // Enqueue failure is non-fatal — the memory is safe in L2 and the
        // backfill_embeddings binary can re-enqueue.
        if let Err(e) = self
            .l1
            .enqueue_enrichment("fact", &fact_id, &source_agent)
            .await
        {
            warn!(error = %e, "TieredStorage: enrichment enqueue failed (memory safe in L2)");
        }

        Ok(())
    }

    async fn query_facts(
        &self,
        domain: &str,
        tags: &[String],
        limit: u32,
    ) -> Result<Vec<FactCard>> {
        // Try L1 (hot cache)
        match self.l1.query_facts(domain, tags, limit).await {
            Ok(facts) if !facts.is_empty() => return Ok(facts),
            Err(e) => warn!(error = %e, "TieredStorage: L1 query failed, falling through"),
            Ok(_) => {} // cache miss — fall through
        }

        // Fall through to L2 (SQLite)
        self.l2.query_facts(domain, tags, limit).await
    }

    async fn query_agent_facts(
        &self,
        agent_name: &str,
        limit: u32,
        task_id: Option<&str>,
    ) -> Result<Vec<FactCard>> {
        self.l2.query_agent_facts(agent_name, limit, task_id).await
    }

    async fn record_episode(&self, episode: EpisodicMemory) -> Result<()> {
        self.l2.record_episode(episode.clone()).await?;

        if let Err(e) = self
            .l1
            .enqueue_enrichment("episode", &episode.session_id, &episode.session_id)
            .await
        {
            warn!(error = %e, "TieredStorage: enrichment enqueue failed (memory safe in L2)");
        }

        Ok(())
    }

    async fn query_recent_episodes(
        &self,
        agent_name: &str,
        limit: u32,
        task_id: Option<&str>,
    ) -> Result<Vec<EpisodicMemory>> {
        self.l2
            .query_recent_episodes(agent_name, limit, task_id)
            .await
    }

    async fn search_semantic(
        &self,
        query: &str,
        partition: &str,
        limit: u32,
        min_score: f32,
    ) -> Result<Vec<FactCard>> {
        // L3 (Qdrant vector search) first. When it produces candidates, its
        // threshold verdict is final — see l3_semantic_verdict. Only an L3
        // error (embedding model down, Qdrant RPC failure) or an empty
        // candidate set reaches the L2 text-match fallback.
        match self
            .l3
            .search_semantic_scored(query, partition, limit)
            .await
        {
            Ok(scored) => {
                if let Some(hits) = l3_semantic_verdict(scored, limit, min_score) {
                    return Ok(hits);
                }
            }
            Err(e) => {
                warn!(error = %e, "TieredStorage: L3 semantic search failed, falling through to L2 text match")
            }
        }
        self.l2
            .search_semantic(query, partition, limit, min_score)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{QdrantTier, RedisTier, SqliteTier};
    use koad_proto::cass::v1::FactCard;
    use std::sync::Arc;

    fn scored_fact(id: &str, score: f32) -> (f32, FactCard) {
        (
            score,
            FactCard {
                id: id.to_string(),
                domain: "test".to_string(),
                content: "c".to_string(),
                source_agent: "clyde".to_string(),
                session_id: "s".to_string(),
                confidence: 0.9,
                tags: vec![],
                created_at: None,
                metadata: None,
            },
        )
    }

    #[test]
    fn l3_verdict_zero_candidates_falls_through_to_l2() {
        assert!(
            l3_semantic_verdict(vec![], 5, 0.5).is_none(),
            "empty partition must keep the L2 lexical safety net"
        );
    }

    #[test]
    fn l3_verdict_threshold_empty_is_final_without_l2_fallback() {
        let verdict = l3_semantic_verdict(vec![scored_fact("low", 0.3)], 5, 0.5);
        assert_eq!(
            verdict,
            Some(vec![]),
            "candidates existed and failed the threshold — precision verdict is final"
        );
    }

    #[test]
    fn l3_verdict_passing_candidates_returned_filtered() {
        let verdict = l3_semantic_verdict(
            vec![scored_fact("low", 0.3), scored_fact("hi", 0.9)],
            5,
            0.5,
        )
        .expect("candidates existed");
        let ids: Vec<&str> = verdict.iter().map(|f| f.id.as_str()).collect();
        assert_eq!(ids, vec!["hi"]);
    }

    async fn make_tiered() -> anyhow::Result<TieredStorage> {
        let sqlite = Arc::new(SqliteTier::new(":memory:")?);
        // Use live local Redis and Qdrant for integration validation
        let redis_client = koad_core::utils::redis::RedisClient::new(
            &std::env::var("KOADOS_HOME").unwrap_or_else(|_| {
                format!(
                    "{}/.koad-os",
                    std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string())
                )
            }),
            false,
        )
        .await?;
        let l1 = Arc::new(RedisTier::new(redis_client.pool.clone()));
        let l3 = Arc::new(match QdrantTier::new("http://127.0.0.1:6334", None).await {
            Ok(q) => q,
            Err(_) => QdrantTier::new_offline(),
        });
        Ok(TieredStorage::new(l1, sqlite, l3))
    }

    #[tokio::test]
    #[ignore = "requires live services (redis, qdrant)"]
    async fn test_tiered_write_and_read() -> anyhow::Result<()> {
        let storage = make_tiered().await?;

        let fact = FactCard {
            id: "tiered-test-001".to_string(),
            domain: "test-tiered".to_string(),
            content: "Tiered memory write-through verified".to_string(),
            source_agent: "clyde".to_string(),
            session_id: "S-TEST".to_string(),
            confidence: 0.9,
            tags: vec!["test".to_string()],
            created_at: None,
            metadata: None,
        };

        // Write through all tiers
        storage.commit_fact(fact.clone()).await?;

        // Read back (should hit L1 Redis cache)
        let results = storage.query_facts("test-tiered", &[], 5).await?;
        assert!(!results.is_empty(), "Expected fact from L1/L2");
        assert_eq!(results[0].content, fact.content);

        Ok(())
    }

    /// Full pipeline: requires live Qdrant + Ollama (nomic-embed-text).
    /// Drives QdrantTier directly to validate semantic (non-substring) recall.
    #[tokio::test]
    #[ignore = "requires live services (qdrant, ollama with nomic-embed-text)"]
    async fn test_semantic_recall_paraphrase() -> anyhow::Result<()> {
        let intelligence = Arc::new(koad_intelligence::router::InferenceRouter::new_default()?);
        let qdrant = QdrantTier::new("http://127.0.0.1:6334", Some(intelligence)).await?;

        let fact = FactCard {
            id: "semantic-test-001".to_string(),
            domain: "test-semantic".to_string(),
            content: "The deployment failed because the systemd service kept running the old binary from memory".to_string(),
            source_agent: "clyde-semantic-test".to_string(),
            session_id: "S-SEM".to_string(),
            confidence: 0.9,
            tags: vec!["test".to_string()],
            created_at: None,
            metadata: None,
        };
        qdrant.commit_fact(fact).await?;

        // Paraphrased query with minimal keyword overlap — substring/LIKE
        // matching would miss it; real embeddings must rank it first.
        let results = qdrant
            .search_semantic(
                "why did the service restart not pick up the new build",
                "clyde-semantic-test",
                3,
                0.0,
            )
            .await?;
        assert!(
            results.iter().any(|f| f.id == "semantic-test-001"),
            "semantic search must recall the paraphrased fact; got: {:?}",
            results.iter().map(|f| &f.id).collect::<Vec<_>>()
        );
        Ok(())
    }

    /// Episode partition keying: exact-partition recall + cross-partition isolation.
    /// Requires live Qdrant + Ollama (nomic-embed-text), like the paraphrase test above.
    #[tokio::test]
    #[ignore = "requires live services (qdrant, ollama with nomic-embed-text)"]
    async fn test_episode_recall_by_partition() -> anyhow::Result<()> {
        let intelligence = Arc::new(koad_intelligence::router::InferenceRouter::new_default()?);
        let qdrant = QdrantTier::new("http://127.0.0.1:6334", Some(intelligence)).await?;

        let episode = EpisodicMemory {
            session_id: "SID-eptest-0001".into(),
            project_path: "/tmp/eptest".into(),
            summary: "Investigated Redis stream lag in the enrichment worker and fixed the consumer group offset".into(),
            turn_count: 3,
            timestamp: Some(prost_types::Timestamp {
                seconds: chrono::Utc::now().timestamp(),
                nanos: 0,
            }),
            task_ids: vec![],
            metadata: None,
            partition: "eptest_TestHost_tester".into(),
        };
        qdrant.record_episode(episode).await?;

        // Paraphrased query, correct partition: must recall.
        let hits = qdrant
            .search_semantic(
                "why was the queue consumer falling behind",
                "eptest_TestHost_tester",
                3,
                0.0,
            )
            .await?;
        assert!(
            hits.iter().any(|f| f.session_id == "SID-eptest-0001"),
            "episode must be recalled under its own partition; got: {:?}",
            hits.iter().map(|f| &f.id).collect::<Vec<_>>()
        );

        // Same query, different partition: must NOT leak.
        let misses = qdrant
            .search_semantic(
                "why was the queue consumer falling behind",
                "other_Host_user",
                3,
                0.0,
            )
            .await?;
        assert!(
            !misses.iter().any(|f| f.session_id == "SID-eptest-0001"),
            "episode must not leak across partitions"
        );
        Ok(())
    }
}
