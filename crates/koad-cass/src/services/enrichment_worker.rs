//! Async enrichment worker — consumes the `cass:enrichment` Redis stream.
//!
//! Per entry: load record from L2 → LLM metadata enrichment (retry ×3, then
//! degrade to embed-only) → embed + upsert Qdrant L3 → persist metadata to
//! L2, refresh L1 → XACK. Embedding failure leaves the entry pending (no
//! ack); it is re-read from the pending list, so nothing is lost and no fake
//! vectors are ever written.

use crate::services::enrichment::{
    build_enrichment_prompt, is_enriched, merge_enrichment, parse_enrichment_output,
};
use crate::storage::redis_tier::ENRICHMENT_STREAM;
use crate::storage::{MemoryTier, QdrantTier, RedisTier, SqliteTier};
use anyhow::Result;
use fred::clients::RedisPool;
use fred::interfaces::StreamsInterface;
use fred::types::XReadResponse;
use koad_intelligence::router::{InferenceRouter, InferenceTask};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tracing::{error, info, warn};

pub const ENRICHMENT_GROUP: &str = "cass-enrichers";
pub const CONSUMER_NAME: &str = "cass-worker-1";
/// Poison entries that fail `MAX_DELIVERIES` consecutive deliveries are
/// copied here and acked, so one bad entry cannot block the queue forever.
pub const DEADLETTER_STREAM: &str = "cass:enrichment:deadletter";
const MAX_DELIVERIES: u32 = 5;
const CHAT_RETRIES: usize = 3;
const CHAT_BACKOFF_SECS: [u64; 3] = [1, 5, 15];
const FAILURE_PAUSE_SECS: u64 = 30;
const READ_BLOCK_MS: u64 = 5000;
const READ_COUNT: u64 = 1;

pub struct EnrichmentWorker {
    pool: RedisPool,
    l1: Arc<RedisTier>,
    l2: Arc<SqliteTier>,
    l3: Arc<QdrantTier>,
    intelligence: Arc<InferenceRouter>,
    enrich_model: String,
}

impl EnrichmentWorker {
    pub fn new(
        pool: RedisPool,
        l1: Arc<RedisTier>,
        l2: Arc<SqliteTier>,
        l3: Arc<QdrantTier>,
        intelligence: Arc<InferenceRouter>,
    ) -> Self {
        let enrich_model =
            std::env::var("KOADOS_INTEL_MODEL").unwrap_or_else(|_| "granite3.3:2b".to_string());
        Self {
            pool,
            l1,
            l2,
            l3,
            intelligence,
            enrich_model,
        }
    }

    /// Main loop. Never returns; all errors are logged and retried.
    pub async fn run(self) {
        loop {
            match self.ensure_group().await {
                Ok(()) => break,
                Err(e) => {
                    warn!(error = %e, "EnrichmentWorker: cannot create consumer group; retrying in 30s");
                    tokio::time::sleep(Duration::from_secs(FAILURE_PAUSE_SECS)).await;
                }
            }
        }
        info!(
            "EnrichmentWorker: online (stream {}, group {})",
            ENRICHMENT_STREAM, ENRICHMENT_GROUP
        );

        // "0" reads this consumer's own pending entries (crash recovery),
        // ">" reads new entries. Start in recovery mode.
        let mut read_pending = true;
        // Consecutive-failure count per entry id. In-memory only: it resets
        // on restart, so each boot allows MAX_DELIVERIES more attempts —
        // accepted trade-off, still bounded per boot.
        let mut failures: HashMap<String, u32> = HashMap::new();
        loop {
            let id = if read_pending { "0" } else { ">" };
            match self.read_one(id).await {
                Ok(Some((entry_id, fields))) => match self.process(&fields).await {
                    Ok(()) => {
                        failures.remove(&entry_id);
                        self.ack(&entry_id).await;
                    }
                    Err(e) => {
                        let kind = fields.get("kind").map(String::as_str).unwrap_or("");
                        let mem_id = fields.get("id").map(String::as_str).unwrap_or("");
                        let count = {
                            let c = failures.entry(entry_id.clone()).or_insert(0);
                            *c += 1;
                            *c
                        };
                        if count >= MAX_DELIVERIES {
                            // Poison pill: dead-letter and ack to unblock the queue.
                            error!(error = %e, entry = %entry_id, kind = %kind, id = %mem_id,
                                deliveries = count,
                                "EnrichmentWorker: entry exhausted deliveries; dead-lettering");
                            match self.dead_letter(&fields).await {
                                Ok(()) => {
                                    self.ack(&entry_id).await;
                                    failures.remove(&entry_id);
                                }
                                Err(de) => {
                                    // Keep the counter — next failure retries the dead-letter.
                                    warn!(error = %de, entry = %entry_id,
                                        "EnrichmentWorker: dead-letter XADD failed; entry stays pending");
                                    read_pending = true;
                                    tokio::time::sleep(Duration::from_secs(FAILURE_PAUSE_SECS))
                                        .await;
                                }
                            }
                        } else {
                            warn!(error = %e, entry = %entry_id, kind = %kind, id = %mem_id,
                                deliveries = count,
                                "EnrichmentWorker: processing failed; leaving pending, pausing");
                            read_pending = true;
                            tokio::time::sleep(Duration::from_secs(FAILURE_PAUSE_SECS)).await;
                        }
                    }
                },
                Ok(None) => {
                    // Pending drained (or block timeout on new entries).
                    read_pending = false;
                }
                Err(e) => {
                    warn!(error = %e, "EnrichmentWorker: stream read failed; pausing");
                    tokio::time::sleep(Duration::from_secs(FAILURE_PAUSE_SECS)).await;
                }
            }
        }
    }

    /// Acknowledge one entry; a failed or no-op XACK is logged, never ignored.
    async fn ack(&self, entry_id: &str) {
        match self
            .pool
            .xack::<u64, _, _, _>(ENRICHMENT_STREAM, ENRICHMENT_GROUP, entry_id)
            .await
        {
            Ok(0) => warn!(entry = %entry_id, "EnrichmentWorker: XACK acknowledged 0 entries"),
            Ok(_) => {}
            Err(e) => warn!(error = %e, entry = %entry_id, "EnrichmentWorker: XACK failed"),
        }
    }

    /// Copy a poison entry's fields to the dead-letter stream (capped like
    /// the main stream). The caller acks the original only on success.
    async fn dead_letter(&self, fields: &HashMap<String, String>) -> Result<()> {
        let pairs: Vec<(String, String)> =
            fields.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let _: String = self
            .pool
            .xadd(
                DEADLETTER_STREAM,
                false,
                ("MAXLEN", "~", 100_000),
                "*",
                pairs,
            )
            .await?;
        Ok(())
    }

    async fn ensure_group(&self) -> Result<()> {
        // MKSTREAM=true creates the stream if absent. BUSYGROUP = group exists — fine.
        match self
            .pool
            .xgroup_create::<(), _, _, _>(ENRICHMENT_STREAM, ENRICHMENT_GROUP, "$", true)
            .await
        {
            Ok(()) => Ok(()),
            Err(e) if e.details().contains("BUSYGROUP") => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Read a single entry. `id` is "0" (own pending) or ">" (new, blocks 5s).
    async fn read_one(&self, id: &str) -> Result<Option<(String, HashMap<String, String>)>> {
        let block = if id == ">" { Some(READ_BLOCK_MS) } else { None };
        let resp: XReadResponse<String, String, String, String> = self
            .pool
            .xreadgroup_map(
                ENRICHMENT_GROUP,
                CONSUMER_NAME,
                Some(READ_COUNT),
                block,
                false,
                ENRICHMENT_STREAM,
                id,
            )
            .await?;
        // Response shape: { stream_key: [(entry_id, {field: value})] }
        for (_stream, entries) in resp {
            if let Some((entry_id, fields)) = entries.into_iter().next() {
                return Ok(Some((entry_id, fields)));
            }
        }
        Ok(None)
    }

    async fn process(&self, fields: &HashMap<String, String>) -> Result<()> {
        let kind = fields.get("kind").map(String::as_str).unwrap_or("");
        let id = fields.get("id").map(String::as_str).unwrap_or("");
        if id.is_empty() {
            // Malformed entry — ack it away rather than looping forever.
            warn!("EnrichmentWorker: malformed entry (missing id), dropping");
            return Ok(());
        }
        match kind {
            "fact" => self.process_fact(id).await,
            "episode" => self.process_episode(id).await,
            other => {
                warn!(kind = %other, "EnrichmentWorker: unknown kind, dropping");
                Ok(())
            }
        }
    }

    async fn process_fact(&self, id: &str) -> Result<()> {
        let Some(mut fact) = self.l2.get_fact_by_id(id).await? else {
            warn!(id = %id, "EnrichmentWorker: fact vanished from L2, dropping");
            return Ok(());
        };

        if !is_enriched(&fact.metadata) {
            if let Some(out) = self.enrich_with_retries(&fact.content).await {
                let mut md = fact.metadata.take().unwrap_or_default();
                merge_enrichment(&mut md, &out, fact.confidence, &self.enrich_model);
                self.l2.update_fact_metadata(id, &md).await?;
                fact.metadata = Some(md);
            }
            // None => degraded: token-only metadata, still embed (spec §3).
        }

        // Embed + upsert L3. Error propagates → entry stays pending, retried.
        self.l3.commit_fact(fact.clone()).await?;

        // L1 refresh (non-fatal — hot cache only).
        if let Err(e) = self.l1.commit_fact(fact).await {
            warn!(error = %e, "EnrichmentWorker: L1 refresh failed");
        }
        info!(id = %id, "EnrichmentWorker: fact enriched + indexed");
        Ok(())
    }

    async fn process_episode(&self, session_id: &str) -> Result<()> {
        let Some(mut ep) = self.l2.get_episode_by_session(session_id).await? else {
            warn!(id = %session_id, "EnrichmentWorker: episode vanished from L2, dropping");
            return Ok(());
        };

        if !is_enriched(&ep.metadata) {
            if let Some(out) = self.enrich_with_retries(&ep.summary).await {
                let mut md = ep.metadata.take().unwrap_or_default();
                // Episodes have no confidence field; 1.0 = "only overwrite empty salience".
                merge_enrichment(&mut md, &out, 1.0, &self.enrich_model);
                self.l2.update_episode_metadata(session_id, &md).await?;
                ep.metadata = Some(md);
            }
        }

        self.l3.record_episode(ep).await?;
        // No L1 refresh: episodes are not L1-cached (RedisTier::record_episode
        // is a stub). Revisit if episodes ever get L1 caching.
        info!(id = %session_id, "EnrichmentWorker: episode enriched + indexed");
        Ok(())
    }

    /// LLM enrichment with bounded retries. Returns None after exhausting
    /// retries — caller degrades to embed-only, never blocks the queue on a
    /// model that keeps emitting garbage. Degenerate-but-parseable replies
    /// (e.g. `{}`) are treated as failures too: merging them would set the
    /// enriched marker with zero signal and permanently suppress retries.
    async fn enrich_with_retries(
        &self,
        content: &str,
    ) -> Option<crate::services::enrichment::EnrichmentOutput> {
        let prompt = build_enrichment_prompt(content);
        for (attempt, backoff) in CHAT_BACKOFF_SECS.iter().enumerate().take(CHAT_RETRIES) {
            match self
                .intelligence
                .select(InferenceTask::Evaluation)
                .chat(&prompt)
                .await
            {
                Ok(raw) => match parse_enrichment_output(&raw) {
                    Ok(out) if out.is_meaningful() => return Some(out),
                    Ok(_) => {
                        warn!(
                            attempt = attempt + 1,
                            "EnrichmentWorker: degenerate (empty) LLM output"
                        );
                    }
                    Err(e) => {
                        warn!(error = %e, attempt = attempt + 1, "EnrichmentWorker: unparseable LLM output");
                    }
                },
                Err(e) => {
                    warn!(error = %e, attempt = attempt + 1, "EnrichmentWorker: LLM chat failed");
                }
            }
            // No point sleeping after the final attempt — caller degrades now.
            if attempt + 1 < CHAT_RETRIES {
                tokio::time::sleep(Duration::from_secs(*backoff)).await;
            }
        }
        None
    }
}
