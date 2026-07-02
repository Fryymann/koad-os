# CASS Semantic Enrichment Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Real semantic embeddings in Qdrant via `nomic-embed-text` plus async LLM metadata enrichment of agent memories via a Redis Stream worker.

**Architecture:** `memory.commit` stays fast (token metadata → L1/L2), then enqueues to Redis stream `cass:enrichment`. A tokio worker inside koad-cass consumes the stream: a lightweight LLM (`granite3.3:2b`) fills judgment metadata, `nomic-embed-text` produces the vector, and the worker upserts Qdrant L3 + updates L2/L1. Fingerprint vectors are eliminated. Spec: `docs/superpowers/specs/2026-07-01-cass-semantic-enrichment-design.md`.

**Tech Stack:** Rust (tonic/tokio workspace), fred 9 (Redis streams), qdrant-client 1.17, rusqlite, Ollama HTTP API.

**Working rules for the executor:**
- Workspace root: `/home/ideans/koados-citadel`. All paths below are relative to it.
- After every code step run `cargo check -p <crate>` and fix compile errors before moving on. fred generic signatures are finicky — if a fred call doesn't compile, adjust the turbofish/type annotations per the compiler hint; the semantics in this plan are correct.
- Tests: `cargo test -p <crate>` (unit). Tests marked `#[ignore]` need live services — do NOT run them unless the step says so.
- Commit after each task with the message given. Do not push.

---

## File Structure

| File | Action | Responsibility |
|---|---|---|
| `crates/koad-intelligence/src/router.rs` | Modify | `InferenceTask::Embedding`, second Ollama client, drop cloud embed fallback |
| `crates/koad-intelligence/src/tests/router_tests.rs` | Modify | Routing test for embed client |
| `crates/koad-cass/src/storage/qdrant_tier.rs` | Modify | Result-based embeddings, deferred dim detection, `embedding_model` payload, delete fingerprint |
| `crates/koad-cass/src/storage/redis_tier.rs` | Modify | `enqueue_enrichment` via XADD |
| `crates/koad-cass/src/storage/tiered.rs` | Modify | Remove L3 fire-and-forget writes; enqueue instead |
| `crates/koad-cass/src/storage/sqlite_tier.rs` | Modify | Get/update single fact & episode by id |
| `crates/koad-cass/src/services/enrichment.rs` | Create | Pure logic: prompt build, JSON parse, metadata merge |
| `crates/koad-cass/src/services/enrichment_worker.rs` | Create | Stream consumer loop |
| `crates/koad-cass/src/services/mod.rs` | Modify | Register new modules |
| `crates/koad-cass/src/main.rs` | Modify | Spawn worker, share SqliteTier Arc |
| `crates/koad-cass/src/bin/backfill_embeddings.rs` | Create | One-shot re-embed migration |
| `.env.template` | Modify | `KOADOS_INTEL_MODEL`, `KOADOS_EMBED_MODEL` |

---

### Task 1: Router model split (koad-intelligence)

**Files:**
- Modify: `crates/koad-intelligence/src/router.rs`
- Test: `crates/koad-intelligence/src/tests/router_tests.rs`

- [ ] **Step 1.1: Write the failing test**

Append to `crates/koad-intelligence/src/tests/router_tests.rs` (reuse the existing `MockInferenceClient` pattern at the top of the file; add a dimension-parameterized mock):

```rust
struct DimMockClient {
    dim: usize,
}

#[async_trait]
impl InferenceClient for DimMockClient {
    async fn chat(&self, _prompt: &str) -> Result<String> {
        Ok("chat".to_string())
    }
    async fn summarize(&self, _text: &str) -> Result<String> {
        Ok("summary".to_string())
    }
    async fn score_significance(&self, _text: &str) -> Result<f32> {
        Ok(0.5)
    }
    async fn embed(&self, _text: &str) -> Result<Vec<f32>> {
        Ok(vec![0.5f32; self.dim])
    }
}

#[tokio::test]
async fn test_embedding_task_routes_to_embed_client() -> Result<()> {
    let chat_client = Arc::new(DimMockClient { dim: 4 });
    let embed_client = Arc::new(DimMockClient { dim: 8 });

    let router = InferenceRouter::new(chat_client).with_embed_client(embed_client);

    // embed() must use the dedicated embed client (dim 8), not the chat client (dim 4)
    let vec = router.embed("hello").await?;
    assert_eq!(vec.len(), 8);
    Ok(())
}

#[tokio::test]
async fn test_embed_defaults_to_local_client_without_embed_client() -> Result<()> {
    let chat_client = Arc::new(DimMockClient { dim: 4 });
    let router = InferenceRouter::new(chat_client);
    let vec = router.embed("hello").await?;
    assert_eq!(vec.len(), 4);
    Ok(())
}
```

- [ ] **Step 1.2: Run test to verify it fails**

Run: `cargo test -p koad-intelligence test_embedding_task_routes -- --nocapture`
Expected: FAIL — `no method named 'with_embed_client'`.

- [ ] **Step 1.3: Implement router changes**

In `crates/koad-intelligence/src/router.rs`:

a) Add variant to `InferenceTask` (after `Reasoning`):

```rust
    /// Dense vector embedding generation (dedicated local embedding model).
    Embedding,
```

b) Replace the `InferenceRouter` struct and the `new` / `new_default` / `select` / `embed` methods with:

```rust
/// A router that selects the appropriate [`InferenceClient`] based on task and availability.
pub struct InferenceRouter {
    local_client: Arc<dyn InferenceClient>,
    embed_client: Arc<dyn InferenceClient>,
}

impl InferenceRouter {
    /// Create a new router with the specified client for all tasks.
    pub fn new(local_client: Arc<dyn InferenceClient>) -> Self {
        Self {
            embed_client: local_client.clone(),
            local_client,
        }
    }

    /// Override the client used for `InferenceTask::Embedding`.
    pub fn with_embed_client(mut self, embed_client: Arc<dyn InferenceClient>) -> Self {
        self.embed_client = embed_client;
        self
    }

    /// Create a new router with default clients (Local Ollama).
    ///
    /// Chat/enrichment model from `KOADOS_INTEL_MODEL` (default `granite3.3:2b`).
    /// Embedding model from `KOADOS_EMBED_MODEL` (default `nomic-embed-text`).
    ///
    /// # Errors
    /// Returns an error if the HTTP client cannot be built.
    pub fn new_default() -> Result<Self> {
        let model =
            std::env::var("KOADOS_INTEL_MODEL").unwrap_or_else(|_| "granite3.3:2b".to_string());
        let embed_model =
            std::env::var("KOADOS_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());
        info!(model = %model, embed_model = %embed_model, "InferenceRouter: Initializing Ollama clients.");
        Ok(
            Self::new(Arc::new(OllamaClient::new(Some(&model), None)?))
                .with_embed_client(Arc::new(OllamaClient::new(Some(&embed_model), None)?)),
        )
    }

    /// Select a client for the given task.
    pub fn select(&self, task: InferenceTask) -> Arc<dyn InferenceClient> {
        match task {
            InferenceTask::Embedding => self.embed_client.clone(),
            _ => self.local_client.clone(),
        }
    }

    /// Generate a dense vector embedding. Local-only by design: cloud embedding
    /// models emit different dimensions than the local Qdrant collections, so a
    /// cloud fallback vector would be unusable. Errors propagate to the caller
    /// (the enrichment worker retries).
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.select(InferenceTask::Embedding).embed(text).await
    }
}
```

c) **Delete** the entire `embed_gemini` and `embed_openrouter` private functions (the cloud embed fallback is removed per spec). Keep `summarize` and `score` convenience methods and the `IntelligenceRouter` trait impl unchanged.

- [ ] **Step 1.4: Run tests**

Run: `cargo test -p koad-intelligence`
Expected: PASS (all, including the two new tests and the pre-existing `test_router_selects_and_summarizes`).

- [ ] **Step 1.5: Verify workspace still compiles**

Run: `cargo check --workspace`
Expected: clean. (Callers use `new_default()` / `new(client)` — both signatures preserved.)

- [ ] **Step 1.6: Commit**

```bash
git add crates/koad-intelligence/src/router.rs crates/koad-intelligence/src/tests/router_tests.rs
git commit -m "feat(intelligence): dedicated embedding client + InferenceTask::Embedding

KOADOS_EMBED_MODEL (default nomic-embed-text) routes embed() to its own
Ollama client. Chat default flips mistral -> granite3.3:2b. Cloud embed
fallback removed for dimension consistency."
```

---

### Task 2: QdrantTier — real embeddings only

**Files:**
- Modify: `crates/koad-cass/src/storage/qdrant_tier.rs`

- [ ] **Step 2.1: Write the failing tests**

In the `#[cfg(test)] mod tests` block at the bottom of `crates/koad-cass/src/storage/qdrant_tier.rs`, add:

```rust
    #[tokio::test]
    async fn offline_tier_get_vector_errors() {
        let tier = QdrantTier::new_offline();
        let res = tier.get_vector("hello world").await;
        assert!(res.is_err(), "offline tier must not fabricate vectors");
    }

    #[test]
    fn payload_carries_embedding_model() {
        let fact = sample_fact(None); // existing test helper
        let payload = QdrantTier::make_payload(&fact, "nomic-embed-text");
        let model = payload.get("embedding_model").and_then(|v| v.kind.as_ref());
        match model {
            Some(Kind::StringValue(s)) => assert_eq!(s, "nomic-embed-text"),
            other => panic!("expected embedding_model string, got {:?}", other),
        }
    }
```

Note: `sample_fact` is the existing helper in that test module (signature `fn sample_fact(md: Option<MemoryMetadata>) -> FactCard`). If its name differs slightly, use whatever helper the existing round-trip tests use.

- [ ] **Step 2.2: Run tests to verify failure**

Run: `cargo test -p koad-cass qdrant -- --nocapture`
Expected: FAIL — `make_payload` takes 1 argument, `get_vector` is private/infallible.

- [ ] **Step 2.3: Rework QdrantTier**

In `crates/koad-cass/src/storage/qdrant_tier.rs`:

a) Delete the line `const VECTOR_DIM: u64 = 32;`.

b) Replace the struct definition:

```rust
pub struct QdrantTier {
    client: Option<Qdrant>,
    intelligence: Option<Arc<InferenceRouter>>,
    /// 0 = not yet detected (embedding model unreachable at boot). Detection is
    /// retried lazily by ensure_ready() on first use.
    vector_dim: tokio::sync::RwLock<u64>,
    embed_model: String,
}
```

c) Replace `new_offline` and `new`:

```rust
    /// Create a no-op offline tier for degraded-mode boot (Qdrant unreachable).
    pub fn new_offline() -> Self {
        Self {
            client: None,
            intelligence: None,
            vector_dim: tokio::sync::RwLock::new(0),
            embed_model: default_embed_model(),
        }
    }

    pub async fn new(url: &str, intelligence: Option<Arc<InferenceRouter>>) -> Result<Self> {
        let client = Qdrant::from_url(url)
            .build()
            .context("Failed to build Qdrant client")?;

        // Connectivity probe: keeps the degraded-boot path in main.rs working
        // (an unreachable Qdrant must make new() return Err).
        client
            .collection_exists(COLLECTION)
            .await
            .context("Qdrant unreachable")?;

        let tier = Self {
            client: Some(client),
            intelligence,
            vector_dim: tokio::sync::RwLock::new(0),
            embed_model: default_embed_model(),
        };

        // Best-effort dimension detection at boot. Failure is non-fatal:
        // the tier starts in deferred mode and ensure_ready() retries on first use.
        if let Err(e) = tier.ensure_ready().await {
            tracing::warn!(
                "QdrantTier: embedding model unavailable at boot ({}). Deferred mode — no writes until it returns.",
                e
            );
        }

        Ok(tier)
    }

    /// Ensure the embedding dimension is known and collections exist.
    /// Returns the dimension, or an error if the embedding model is unreachable.
    async fn ensure_ready(&self) -> Result<u64> {
        {
            let d = *self.vector_dim.read().await;
            if d > 0 {
                return Ok(d);
            }
        }
        let Some(client) = &self.client else {
            return Err(anyhow::anyhow!("QdrantTier: offline"));
        };
        let Some(intel) = &self.intelligence else {
            return Err(anyhow::anyhow!("QdrantTier: no embedding client configured"));
        };

        let mut guard = self.vector_dim.write().await;
        if *guard > 0 {
            return Ok(*guard);
        }

        let probe = intel
            .embed("dimension probe")
            .await
            .context("embedding model unavailable")?;
        let dim = probe.len() as u64;
        if dim == 0 {
            return Err(anyhow::anyhow!("embedding probe returned empty vector"));
        }

        for c in [COLLECTION, EPISODE_COLLECTION] {
            if !client.collection_exists(c).await? {
                client
                    .create_collection(
                        CreateCollectionBuilder::new(c)
                            .vectors_config(VectorParamsBuilder::new(dim, Distance::Cosine)),
                    )
                    .await
                    .with_context(|| format!("Failed to create Qdrant collection {c}"))?;
            }
        }

        *guard = dim;
        tracing::info!("QdrantTier: ready (embedding dim {})", dim);
        Ok(dim)
    }

    /// Drop and recreate both collections at the detected dimension.
    /// Used by the backfill_embeddings migration binary only.
    pub async fn recreate_collections(&self) -> Result<()> {
        let dim = self.ensure_ready().await?;
        let Some(client) = &self.client else {
            return Err(anyhow::anyhow!("QdrantTier: offline"));
        };
        for c in [COLLECTION, EPISODE_COLLECTION] {
            if client.collection_exists(c).await? {
                client.delete_collection(c).await?;
            }
            client
                .create_collection(
                    CreateCollectionBuilder::new(c)
                        .vectors_config(VectorParamsBuilder::new(dim, Distance::Cosine)),
                )
                .await
                .with_context(|| format!("Failed to recreate Qdrant collection {c}"))?;
        }
        Ok(())
    }
```

And add a free function near `metadata_to_json`:

```rust
fn default_embed_model() -> String {
    std::env::var("KOADOS_EMBED_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string())
}
```

d) **Delete** the `content_vector` method entirely. Replace `get_vector` with (note: `pub(crate)` so the worker test in Task 2.1 can call it):

```rust
    /// Generate an embedding vector for the given content.
    /// Errors propagate — no fingerprint fallback. Fabricated vectors poison
    /// the semantic space; callers (enrichment worker) retry instead.
    pub(crate) async fn get_vector(&self, text: &str) -> Result<Vec<f32>> {
        let dim = self.ensure_ready().await?;
        let intel = self
            .intelligence
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("QdrantTier: no embedding client configured"))?;
        let vec = intel.embed(text).await.context("embedding failed")?;
        if vec.len() as u64 != dim {
            return Err(anyhow::anyhow!(
                "embedding dimension mismatch (expected {}, got {})",
                dim,
                vec.len()
            ));
        }
        Ok(vec)
    }
```

e) Change `make_payload` and `make_episode_payload` signatures to take the model name, and add the payload field. For `make_payload`:

```rust
    fn make_payload(
        fact: &FactCard,
        embedding_model: &str,
    ) -> HashMap<String, qdrant_client::qdrant::Value> {
```

…and at the end of the function, before `p`, insert:

```rust
        p.insert(
            "embedding_model".into(),
            Value {
                kind: Some(Kind::StringValue(embedding_model.to_string())),
            },
        );
```

Apply the identical change to `make_episode_payload`.

f) Update the write paths — errors now propagate:

`commit_fact` (in `impl MemoryTier`):

```rust
    async fn commit_fact(&self, fact: FactCard) -> Result<()> {
        let Some(client) = &self.client else {
            return Err(anyhow::anyhow!("QdrantTier: offline, cannot index fact"));
        };
        let vector = self.get_vector(&fact.content).await?;
        let payload = Self::make_payload(&fact, &self.embed_model);
        let point = PointStruct::new(Self::point_id(&fact.id), vector, payload);

        client
            .upsert_points(UpsertPointsBuilder::new(COLLECTION, vec![point]))
            .await
            .context("QdrantTier: upsert failed")?;
        Ok(())
    }
```

`commit_facts` (batch, inherent impl): same pattern — `let vector = self.get_vector(&fact.content).await?;` and `Self::make_payload(&fact, &self.embed_model)`; keep the `Ok(())` early return for the empty-vec case but change the offline case to the same `Err` as above.

`record_episode`: `let vector = self.get_vector(&episode.summary).await?;` and `Self::make_episode_payload(&episode, &self.embed_model)`; offline case → `Err` as above.

`search_semantic`: keep the `Ok(vec![])` early return when `client` is `None` (read path degrades gracefully — TieredStorage falls through to L2), but change the vector line to:

```rust
        let vector = self.get_vector(query).await?;
```

(An `Err` here also falls through to L2 in `TieredStorage::search_semantic` — that is the designed degradation.)

g) Fix the existing round-trip tests: they call `make_payload(...)` — add the new `"test-model"` argument wherever the compiler complains.

- [ ] **Step 2.4: Run tests**

Run: `cargo test -p koad-cass qdrant`
Expected: PASS — new tests plus existing round-trip tests.

- [ ] **Step 2.5: Commit**

```bash
git add crates/koad-cass/src/storage/qdrant_tier.rs
git commit -m "feat(cass): QdrantTier real embeddings only — no fingerprint fallback

get_vector returns Result; deferred dim detection when Ollama is down at
boot; embedding_model recorded in point payloads; recreate_collections
added for the backfill migration."
```

---

### Task 3: Enrichment queue — RedisTier XADD + TieredStorage stops writing L3

**Files:**
- Modify: `crates/koad-cass/src/storage/redis_tier.rs`
- Modify: `crates/koad-cass/src/storage/tiered.rs`

- [ ] **Step 3.1: Add stream constants and enqueue method to RedisTier**

In `crates/koad-cass/src/storage/redis_tier.rs`, extend the imports:

```rust
use fred::interfaces::{KeysInterface, SetsInterface, StreamsInterface};
```

Add above `pub struct RedisTier`:

```rust
/// Redis stream fed by TieredStorage at commit time, consumed by the
/// enrichment worker (consumer group `cass-enrichers`).
pub const ENRICHMENT_STREAM: &str = "cass:enrichment";
```

Add inside `impl RedisTier` (after `new`):

```rust
    /// Enqueue a memory for async enrichment (LLM metadata + embedding).
    /// `kind` is "fact" or "episode"; `id` is the fact id or episode session_id.
    pub async fn enqueue_enrichment(&self, kind: &str, id: &str, partition: &str) -> Result<()> {
        let fields = vec![
            ("kind", kind.to_string()),
            ("id", id.to_string()),
            ("partition", partition.to_string()),
        ];
        let _: String = self
            .pool
            .xadd(ENRICHMENT_STREAM, false, ("MAXLEN", "~", 100_000), "*", fields)
            .await?;
        Ok(())
    }
```

Cap rationale (review finding): XACK removes entries from the consumer group's
PEL but never from the stream itself — an uncapped stream grows forever even
with a healthy worker. Approximate MAXLEN (`"~"`) bounds it cheaply; under
extreme backlog trimming can evict un-acked entries, which are recoverable via
`backfill_embeddings --enqueue` (L2 authoritative).

fred note: if the cap tuple fails inference, follow compiler hints toward `fred::types::XCap`; if `"*"` fails for the id argument, use `fred::types::XID::Auto`.

Run: `cargo check -p koad-cass`
Expected: clean.

- [ ] **Step 3.2: Replace L3 fire-and-forget writes in TieredStorage**

In `crates/koad-cass/src/storage/tiered.rs`:

a) `commit_fact`: replace the L3 block

```rust
        // L3: semantic index — fire-and-forget
        let l3 = self.l3.clone();
        tokio::spawn(async move {
            if let Err(e) = l3.commit_fact(fact).await {
                error!(error = %e, "TieredStorage: L3 write failed");
            }
        });
```

with:

```rust
        // L3 indexing is async: the enrichment worker embeds + upserts Qdrant.
        // Enqueue failure is non-fatal — the memory is safe in L2 and the
        // backfill_embeddings binary can re-enqueue.
        if let Err(e) = self
            .l1
            .enqueue_enrichment("fact", &fact.id, &fact.source_agent)
            .await
        {
            warn!(error = %e, "TieredStorage: enrichment enqueue failed (memory safe in L2)");
        }
```

b) `record_episode`: replace the equivalent `tokio::spawn` L3 block with:

```rust
        if let Err(e) = self
            .l1
            .enqueue_enrichment("episode", &episode.session_id, &episode.session_id)
            .await
        {
            warn!(error = %e, "TieredStorage: enrichment enqueue failed (memory safe in L2)");
        }
```

c) Remove the now-unused `error` import from the `use tracing::{error, warn};` line (becomes `use tracing::warn;`) if the compiler flags it.

d) Update the module doc comment on line 3: `//! Write path: L1 + L2 synchronously; L3 via async enrichment queue.`

e) In the `#[ignore]` test `test_tiered_write_and_read`, delete the two lines:

```rust
        // Small delay for L3 fire-and-forget
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
```

- [ ] **Step 3.3: Verify**

Run: `cargo test -p koad-cass --lib`
Expected: PASS (ignored integration tests skipped).

- [ ] **Step 3.4: Commit**

```bash
git add crates/koad-cass/src/storage/redis_tier.rs crates/koad-cass/src/storage/tiered.rs
git commit -m "feat(cass): route L3 indexing through cass:enrichment Redis stream

Commit path no longer writes Qdrant directly (was the fingerprint-vector
source). XADD is non-fatal; L2 stays authoritative."
```

---

### Task 4: SqliteTier single-record accessors

**Files:**
- Modify: `crates/koad-cass/src/storage/sqlite_tier.rs`

- [ ] **Step 4.1: Write the failing test**

In the `#[cfg(test)]` tests module of `crates/koad-cass/src/storage/sqlite_tier.rs` (follow the style of the existing tests there, which build `SqliteTier::new(":memory:")`):

```rust
    #[tokio::test]
    async fn get_and_update_fact_by_id() -> Result<()> {
        let tier = SqliteTier::new(":memory:")?;
        let fact = FactCard {
            id: "acc-test-001".to_string(),
            source_agent: "clyde".to_string(),
            session_id: "S-ACC".to_string(),
            domain: "test:accessors".to_string(),
            content: "accessor round trip".to_string(),
            confidence: 0.8,
            tags: vec!["t1".to_string()],
            created_at: None,
            metadata: None,
        };
        tier.commit_fact(fact.clone()).await?;

        let loaded = tier.get_fact_by_id("acc-test-001").await?.expect("fact exists");
        assert_eq!(loaded.content, "accessor round trip");
        assert!(tier.get_fact_by_id("no-such-id").await?.is_none());

        let mut md = koad_proto::cass::v1::MemoryMetadata::default();
        md.summary = "enriched summary".to_string();
        tier.update_fact_metadata("acc-test-001", &md).await?;

        let reloaded = tier.get_fact_by_id("acc-test-001").await?.expect("fact exists");
        assert_eq!(reloaded.metadata.expect("metadata").summary, "enriched summary");
        Ok(())
    }
```

- [ ] **Step 4.2: Run test to verify it fails**

Run: `cargo test -p koad-cass get_and_update_fact_by_id`
Expected: FAIL — methods don't exist.

- [ ] **Step 4.3: Implement accessors**

Add to `impl SqliteTier` (inherent impl, NOT the `MemoryTier` trait impl) in `crates/koad-cass/src/storage/sqlite_tier.rs`. The row mappings are copied exactly from the existing `query_facts` / `query_recent_episodes` closures:

```rust
    /// Load a single fact by primary key. Used by the enrichment worker.
    pub async fn get_fact_by_id(&self, id: &str) -> Result<Option<FactCard>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT id, source_agent, session_id, domain, content, confidence, tags, metadata_json
             FROM fact_cards WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |row| {
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
                metadata: metadata_from_json(row.get::<_, Option<String>>(7)?),
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Load a single episode by session_id. Used by the enrichment worker.
    pub async fn get_episode_by_session(
        &self,
        session_id: &str,
    ) -> Result<Option<EpisodicMemory>> {
        let conn = self.conn.lock().await;
        let mut stmt = conn.prepare(
            "SELECT session_id, project_path, summary, turn_count, timestamp, task_ids, metadata_json
             FROM episodic_memories WHERE session_id = ?1",
        )?;
        let mut rows = stmt.query_map(params![session_id], |row| {
            Ok(EpisodicMemory {
                session_id: row.get(0)?,
                project_path: row.get(1)?,
                summary: row.get(2)?,
                turn_count: row.get(3)?,
                timestamp: None,
                task_ids: row
                    .get::<_, String>(5)?
                    .split(',')
                    .map(|s| s.to_string())
                    .collect(),
                metadata: metadata_from_json(row.get::<_, Option<String>>(6)?),
            })
        })?;
        match rows.next() {
            Some(row) => Ok(Some(row?)),
            None => Ok(None),
        }
    }

    /// Persist enriched metadata for a fact. Content and other columns untouched.
    pub async fn update_fact_metadata(
        &self,
        id: &str,
        md: &koad_proto::cass::v1::MemoryMetadata,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        let json = serde_json::to_string(md)?;
        conn.execute(
            "UPDATE fact_cards SET metadata_json = ?1 WHERE id = ?2",
            params![json, id],
        )?;
        Ok(())
    }

    /// Persist enriched metadata for an episode.
    pub async fn update_episode_metadata(
        &self,
        session_id: &str,
        md: &koad_proto::cass::v1::MemoryMetadata,
    ) -> Result<()> {
        let conn = self.conn.lock().await;
        let json = serde_json::to_string(md)?;
        conn.execute(
            "UPDATE episodic_memories SET metadata_json = ?1 WHERE session_id = ?2",
            params![json, session_id],
        )?;
        Ok(())
    }
```

- [ ] **Step 4.4: Run tests**

Run: `cargo test -p koad-cass --lib`
Expected: PASS.

- [ ] **Step 4.5: Commit**

```bash
git add crates/koad-cass/src/storage/sqlite_tier.rs
git commit -m "feat(cass): SqliteTier single-record get/update accessors for enrichment worker"
```

---

### Task 5: Enrichment logic module (pure, unit-tested)

**Files:**
- Create: `crates/koad-cass/src/services/enrichment.rs`
- Modify: `crates/koad-cass/src/services/mod.rs`

- [ ] **Step 5.1: Register the module**

In `crates/koad-cass/src/services/mod.rs` add (alphabetical order with the existing `pub mod` lines):

```rust
pub mod enrichment;
```

- [ ] **Step 5.2: Create the module with tests (TDD — the tests define the contract; write the whole file, then run)**

Create `crates/koad-cass/src/services/enrichment.rs`:

```rust
//! Pure enrichment logic: LLM prompt construction, strict-JSON output parsing,
//! and fill-only-empty metadata merging. No I/O — fully unit-testable.
//!
//! Merge policy (spec §3): agent-supplied metadata is never overwritten. A
//! deterministic default written at commit time (salience == confidence) is
//! treated as overwritable. LLM booleans for privacy only escalate (false→true).

use anyhow::{Context, Result};
use koad_proto::cass::v1::{MemoryMetadata, PrivacyMetadata, RetrievalMetadata};
use serde::Deserialize;

/// Marker key inside metadata_json. Presence = already enriched; worker and
/// backfill skip the LLM pass (but still ensure the embedding exists).
pub const ENRICHMENT_KEY: &str = "enrichment";

#[derive(Debug, Deserialize)]
pub struct EnrichmentOutput {
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub salience: f32,
    #[serde(default)]
    pub volatility: String,
    #[serde(default)]
    pub audience: String,
    #[serde(default)]
    pub sensitivity: String,
    #[serde(default)]
    pub contains_pii: bool,
    #[serde(default)]
    pub contains_secret: bool,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Build the single-shot enrichment prompt. Output contract is strict JSON.
pub fn build_enrichment_prompt(content: &str) -> String {
    format!(
        "You are a memory metadata annotator for an AI agent memory system. \
Analyze the memory content below and respond with ONLY a single JSON object \
(no prose, no markdown fences) exactly matching this schema:\n\
{{\"summary\": \"one-sentence summary\", \
\"salience\": 0.0, \
\"volatility\": \"stable|mutable|ephemeral\", \
\"audience\": \"self|agent|team|global\", \
\"sensitivity\": \"public|internal|private|secret-adjacent\", \
\"contains_pii\": false, \
\"contains_secret\": false, \
\"tags\": [\"lowercase-kebab-tag\"]}}\n\
Rules: salience is 0.0-1.0 (1.0 = architectural decision, resolved bug, or \
user preference; 0.0 = noise). volatility: stable = identity/conventions, \
mutable = project state, ephemeral = transient status. tags: 1-5 topical \
tags.\n\nMEMORY CONTENT:\n{}",
        content
    )
}

/// Parse the model's reply. Tolerates leading/trailing prose and markdown
/// fences by extracting the first '{' .. last '}' span.
pub fn parse_enrichment_output(raw: &str) -> Result<EnrichmentOutput> {
    let start = raw.find('{').context("no JSON object in model output")?;
    let end = raw.rfind('}').context("no closing brace in model output")?;
    if end < start {
        anyhow::bail!("malformed JSON span in model output");
    }
    let mut out: EnrichmentOutput =
        serde_json::from_str(&raw[start..=end]).context("model output is not valid JSON")?;
    out.salience = out.salience.clamp(0.0, 1.0);
    Ok(out)
}

/// Merge LLM output into metadata. Fill-only-empty; `confidence` identifies
/// the deterministic salience default written by `default_metadata` at commit.
/// Records the enrichment marker (model + tags + raw salience) in metadata_json.
pub fn merge_enrichment(
    md: &mut MemoryMetadata,
    out: &EnrichmentOutput,
    confidence: f32,
    model: &str,
) {
    if md.summary.is_empty() && !out.summary.is_empty() {
        md.summary = out.summary.clone();
    }

    let rt = md.retrieval.get_or_insert_with(RetrievalMetadata::default);
    // salience == confidence is the deterministic default from commit time — overwritable.
    if rt.salience == 0.0 || (rt.salience - confidence).abs() < f32::EPSILON {
        if out.salience > 0.0 {
            rt.salience = out.salience;
        }
    }
    if rt.volatility.is_empty() && !out.volatility.is_empty() {
        rt.volatility = out.volatility.clone();
    }
    if rt.audience.is_empty() && !out.audience.is_empty() {
        rt.audience = out.audience.clone();
    }

    let pv = md.privacy.get_or_insert_with(PrivacyMetadata::default);
    if pv.sensitivity.is_empty() && !out.sensitivity.is_empty() {
        pv.sensitivity = out.sensitivity.clone();
    }
    // Escalate-only: LLM can flag PII/secrets, never clear an agent's flag.
    pv.contains_pii = pv.contains_pii || out.contains_pii;
    pv.contains_secret = pv.contains_secret || out.contains_secret;

    // Marker + tags in the metadata_json escape hatch.
    let mut extra: serde_json::Value = serde_json::from_str(&md.metadata_json)
        .unwrap_or_else(|_| serde_json::json!({}));
    if !extra.is_object() {
        extra = serde_json::json!({});
    }
    extra[ENRICHMENT_KEY] = serde_json::json!({
        "model": model,
        "at": chrono::Utc::now().to_rfc3339(),
        "tags": out.tags,
        "llm_salience": out.salience,
    });
    md.metadata_json = extra.to_string();
}

/// True if this metadata already carries the enrichment marker.
pub fn is_enriched(md: &Option<MemoryMetadata>) -> bool {
    md.as_ref()
        .and_then(|m| serde_json::from_str::<serde_json::Value>(&m.metadata_json).ok())
        .map(|v| v.get(ENRICHMENT_KEY).is_some())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_output() -> EnrichmentOutput {
        parse_enrichment_output(
            r#"{"summary": "Qdrant uses cosine distance", "salience": 0.9,
                "volatility": "stable", "audience": "team",
                "sensitivity": "internal", "contains_pii": false,
                "contains_secret": false, "tags": ["qdrant", "vectors"]}"#,
        )
        .unwrap()
    }

    #[test]
    fn parses_json_wrapped_in_prose_and_fences() {
        let raw = "Sure! Here is the JSON:\n```json\n{\"summary\": \"s\", \"salience\": 1.7}\n```";
        let out = parse_enrichment_output(raw).unwrap();
        assert_eq!(out.summary, "s");
        assert_eq!(out.salience, 1.0); // clamped
    }

    #[test]
    fn rejects_output_without_json() {
        assert!(parse_enrichment_output("I cannot help with that.").is_err());
    }

    #[test]
    fn merge_fills_empty_fields_and_sets_marker() {
        let mut md = MemoryMetadata::default();
        merge_enrichment(&mut md, &sample_output(), 0.8, "granite3.3:2b");
        assert_eq!(md.summary, "Qdrant uses cosine distance");
        let rt = md.retrieval.as_ref().unwrap();
        assert_eq!(rt.salience, 0.9);
        assert_eq!(rt.volatility, "stable");
        assert_eq!(rt.audience, "team");
        assert!(is_enriched(&Some(md)));
    }

    #[test]
    fn merge_never_overwrites_agent_supplied_values() {
        let mut md = MemoryMetadata::default();
        md.summary = "agent wrote this".to_string();
        md.retrieval = Some(RetrievalMetadata {
            salience: 0.42, // != confidence 0.8 → agent-supplied, keep
            volatility: "ephemeral".to_string(),
            audience: "self".to_string(),
            ..Default::default()
        });
        md.privacy = Some(PrivacyMetadata {
            contains_pii: true, // escalate-only: stays true
            ..Default::default()
        });

        merge_enrichment(&mut md, &sample_output(), 0.8, "granite3.3:2b");

        assert_eq!(md.summary, "agent wrote this");
        let rt = md.retrieval.as_ref().unwrap();
        assert_eq!(rt.salience, 0.42);
        assert_eq!(rt.volatility, "ephemeral");
        assert_eq!(rt.audience, "self");
        assert!(md.privacy.as_ref().unwrap().contains_pii);
    }

    #[test]
    fn merge_overwrites_deterministic_salience_default() {
        let mut md = MemoryMetadata::default();
        md.retrieval = Some(RetrievalMetadata {
            salience: 0.8, // == confidence → deterministic default, overwritable
            ..Default::default()
        });
        merge_enrichment(&mut md, &sample_output(), 0.8, "granite3.3:2b");
        assert_eq!(md.retrieval.as_ref().unwrap().salience, 0.9);
    }

    #[test]
    fn is_enriched_false_for_fresh_metadata() {
        assert!(!is_enriched(&Some(MemoryMetadata::default())));
        assert!(!is_enriched(&None));
    }
}
```

Note: if `chrono` is not already a dependency of koad-cass, check `crates/koad-cass/Cargo.toml` — `sqlite_tier.rs` uses `chrono::Utc`, so it is present.

- [ ] **Step 5.3: Run tests**

Run: `cargo test -p koad-cass enrichment`
Expected: PASS — 6 tests.

- [ ] **Step 5.4: Commit**

```bash
git add crates/koad-cass/src/services/enrichment.rs crates/koad-cass/src/services/mod.rs
git commit -m "feat(cass): enrichment logic — prompt, strict-JSON parse, fill-only-empty merge"
```

---

### Task 6: Enrichment worker (stream consumer)

**Files:**
- Create: `crates/koad-cass/src/services/enrichment_worker.rs`
- Modify: `crates/koad-cass/src/services/mod.rs`

- [ ] **Step 6.1: Register the module**

In `crates/koad-cass/src/services/mod.rs`:

```rust
pub mod enrichment_worker;
```

- [ ] **Step 6.2: Create the worker**

Create `crates/koad-cass/src/services/enrichment_worker.rs`:

```rust
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
use tracing::{info, warn};

pub const ENRICHMENT_GROUP: &str = "cass-enrichers";
pub const CONSUMER_NAME: &str = "cass-worker-1";
const CHAT_RETRIES: usize = 3;
const CHAT_BACKOFF_SECS: [u64; 3] = [1, 5, 15];
const FAILURE_PAUSE_SECS: u64 = 30;

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
        if let Err(e) = self.ensure_group().await {
            warn!(error = %e, "EnrichmentWorker: cannot create consumer group; retrying in 30s");
            tokio::time::sleep(Duration::from_secs(FAILURE_PAUSE_SECS)).await;
            return Box::pin(self.run()).await;
        }
        info!("EnrichmentWorker: online (stream {}, group {})", ENRICHMENT_STREAM, ENRICHMENT_GROUP);

        // "0" reads this consumer's own pending entries (crash recovery),
        // ">" reads new entries. Start in recovery mode.
        let mut read_pending = true;
        loop {
            let id = if read_pending { "0" } else { ">" };
            match self.read_one(id).await {
                Ok(Some((entry_id, fields))) => {
                    match self.process(&fields).await {
                        Ok(()) => {
                            let _: Result<u64, _> = self
                                .pool
                                .xack(ENRICHMENT_STREAM, ENRICHMENT_GROUP, entry_id.as_str())
                                .await;
                        }
                        Err(e) => {
                            warn!(error = %e, entry = %entry_id,
                                "EnrichmentWorker: processing failed; leaving pending, pausing");
                            read_pending = true;
                            tokio::time::sleep(Duration::from_secs(FAILURE_PAUSE_SECS)).await;
                        }
                    }
                }
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

    async fn ensure_group(&self) -> Result<()> {
        // MKSTREAM=true creates the stream if absent. BUSYGROUP = group exists — fine.
        match self
            .pool
            .xgroup_create::<(), _, _>(ENRICHMENT_STREAM, ENRICHMENT_GROUP, "$", true)
            .await
        {
            Ok(()) => Ok(()),
            Err(e) if e.details().contains("BUSYGROUP") => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    /// Read a single entry. `id` is "0" (own pending) or ">" (new, blocks 5s).
    async fn read_one(&self, id: &str) -> Result<Option<(String, HashMap<String, String>)>> {
        let block = if id == ">" { Some(5000) } else { None };
        let resp: XReadResponse<String, String, String, String> = self
            .pool
            .xreadgroup_map(ENRICHMENT_GROUP, CONSUMER_NAME, Some(1), block, false,
                ENRICHMENT_STREAM, id)
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
                fact.metadata = Some(md);
                self.l2
                    .update_fact_metadata(id, fact.metadata.as_ref().unwrap())
                    .await?;
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
                ep.metadata = Some(md);
                self.l2
                    .update_episode_metadata(session_id, ep.metadata.as_ref().unwrap())
                    .await?;
            }
        }

        self.l3.record_episode(ep).await?;
        info!(id = %session_id, "EnrichmentWorker: episode enriched + indexed");
        Ok(())
    }

    /// LLM enrichment with bounded retries. Returns None after exhausting
    /// retries — caller degrades to embed-only, never blocks the queue on a
    /// model that keeps emitting garbage.
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
                        warn!(attempt = attempt + 1, "EnrichmentWorker: degenerate (empty) LLM output");
                    }
                    Err(e) => {
                        warn!(error = %e, attempt = attempt + 1, "EnrichmentWorker: unparseable LLM output");
                    }
                },
                Err(e) => {
                    warn!(error = %e, attempt = attempt + 1, "EnrichmentWorker: LLM chat failed");
                }
            }
            tokio::time::sleep(Duration::from_secs(*backoff)).await;
        }
        None
    }
}
```

fred notes for this file (same caveat as Task 3): `xreadgroup_map` argument order is `(group, consumer, count, block, noack, keys, ids)`. If the response type parameter or `e.details()` doesn't compile, follow the compiler hints — fred 9's error type exposes the message via `details()`; if not, use `e.to_string().contains("BUSYGROUP")`.

- [ ] **Step 6.3: Compile**

Run: `cargo check -p koad-cass`
Expected: clean (iterate on fred generics if needed — semantics above are correct).

- [ ] **Step 6.4: Run the full unit suite**

Run: `cargo test -p koad-cass --lib`
Expected: PASS.

- [ ] **Step 6.5: Commit**

```bash
git add crates/koad-cass/src/services/enrichment_worker.rs crates/koad-cass/src/services/mod.rs
git commit -m "feat(cass): enrichment worker — consumes cass:enrichment stream

LLM metadata (retry x3 then embed-only) -> nomic embedding -> Qdrant
upsert -> L2/L1 metadata refresh -> XACK. Failures leave entries pending."
```

---

### Task 7: Wire the worker in main.rs

**Files:**
- Modify: `crates/koad-cass/src/main.rs`

- [ ] **Step 7.1: Share the SqliteTier Arc and spawn the worker**

In `crates/koad-cass/src/main.rs`:

a) Add import:

```rust
use koad_cass::services::enrichment_worker::EnrichmentWorker;
```

b) The `sqlite` Arc is currently moved into `TieredStorage::new`. Change the storage construction line to clone it:

```rust
    let storage = Arc::new(TieredStorage::new(
        Arc::clone(&redis_tier),
        Arc::clone(&sqlite),
        Arc::clone(&qdrant),
    ));
```

(`qdrant` is already an `Arc<QdrantTier>` from the match block above it; if the current code moves it, wrap with `Arc::clone(&qdrant)` the same way.)

c) After the existing `tokio::spawn` for `eow_pipeline.start_listener()`, add:

```rust
    // Async enrichment worker: LLM metadata + embeddings for committed memories.
    let enrichment_worker = EnrichmentWorker::new(
        redis.pool.clone(),
        Arc::clone(&redis_tier),
        Arc::clone(&sqlite),
        Arc::clone(&qdrant),
        Arc::clone(&intelligence),
    );
    tokio::spawn(async move {
        enrichment_worker.run().await;
    });
```

- [ ] **Step 7.2: Compile and run full test suite**

Run: `cargo check -p koad-cass && cargo test -p koad-cass --lib`
Expected: clean + PASS.

- [ ] **Step 7.3: Commit**

```bash
git add crates/koad-cass/src/main.rs
git commit -m "feat(cass): spawn enrichment worker at boot"
```

---

### Task 8: backfill_embeddings migration binary

**Files:**
- Create: `crates/koad-cass/src/bin/backfill_embeddings.rs`

- [ ] **Step 8.1: Create the binary**

Create `crates/koad-cass/src/bin/backfill_embeddings.rs` (arg-parsing style copied from `backfill_metadata.rs`; async because embedding is async):

```rust
//! Re-embed all memories into Qdrant with the real embedding model, and
//! enqueue enrichment for rows that lack LLM metadata.
//!
//! Usage: backfill_embeddings --db <path> [--qdrant-url <url>] [--apply] [--enqueue]
//! Default is dry-run: reports row counts, writes nothing.
//! --enqueue additionally XADDs unenriched rows to cass:enrichment so the
//! live worker backfills LLM metadata (requires KOADOS_HOME for Redis UDS).
//!
//! DESTRUCTIVE when --apply: drops and recreates both Qdrant collections
//! (fingerprint-vector points are unrecoverable garbage; L2 SQLite is the
//! source of truth and is never modified by this tool except via the worker).

use anyhow::{Context, Result};
use koad_cass::services::enrichment::is_enriched;
use koad_cass::storage::QdrantTier;
use koad_proto::cass::v1::{EpisodicMemory, FactCard};
use koad_intelligence::router::InferenceRouter;
use rusqlite::Connection;
use std::sync::Arc;

const BATCH: usize = 32;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    let args: Vec<String> = std::env::args().collect();
    let db = arg(&args, "--db").ok_or_else(|| anyhow::anyhow!("--db <path> is required"))?;
    let qdrant_url = arg(&args, "--qdrant-url")
        .or_else(|| std::env::var("KOADOS_URL_QDRANT").ok())
        .unwrap_or_else(|| "http://127.0.0.1:6334".to_string());
    let apply = args.iter().any(|a| a == "--apply");
    let enqueue = args.iter().any(|a| a == "--enqueue");

    let conn = Connection::open(&db)?;
    let facts = load_facts(&conn)?;
    let episodes = load_episodes(&conn)?;
    let unenriched = facts.iter().filter(|f| !is_enriched(&f.metadata)).count()
        + episodes.iter().filter(|e| !is_enriched(&e.metadata)).count();

    println!("facts: {}", facts.len());
    println!("episodes: {}", episodes.len());
    println!("rows missing LLM enrichment: {}", unenriched);

    if !apply {
        println!("dry-run: no writes. Re-run with --apply to drop collections and re-embed.");
        return Ok(());
    }

    let intelligence = Arc::new(InferenceRouter::new_default()?);
    let qdrant = QdrantTier::new(&qdrant_url, Some(intelligence))
        .await
        .context("Qdrant unreachable")?;

    println!("recreating collections (drops fingerprint vectors)...");
    qdrant
        .recreate_collections()
        .await
        .context("recreate failed — is Ollama running? (dimension probe requires the embedding model)")?;

    let mut done = 0usize;
    for chunk in facts.chunks(BATCH) {
        qdrant.commit_facts(chunk.to_vec()).await?;
        done += chunk.len();
        println!("facts embedded: {}/{}", done, facts.len());
    }

    use koad_cass::storage::MemoryTier;
    for (i, ep) in episodes.iter().enumerate() {
        qdrant.record_episode(ep.clone()).await?;
        if (i + 1) % BATCH == 0 || i + 1 == episodes.len() {
            println!("episodes embedded: {}/{}", i + 1, episodes.len());
        }
    }

    if enqueue && unenriched > 0 {
        println!("enqueuing {} unenriched rows for the live worker...", unenriched);
        let home = std::env::var("KOADOS_HOME")
            .or_else(|_| std::env::var("KOAD_HOME"))
            .context("--enqueue requires KOADOS_HOME (or KOAD_HOME) for the Redis socket")?;
        let redis = koad_core::utils::redis::RedisClient::new(&home, false).await?;
        let tier = koad_cass::storage::RedisTier::new(redis.pool.clone());
        for f in facts.iter().filter(|f| !is_enriched(&f.metadata)) {
            tier.enqueue_enrichment("fact", &f.id, &f.source_agent).await?;
        }
        for e in episodes.iter().filter(|e| !is_enriched(&e.metadata)) {
            tier.enqueue_enrichment("episode", &e.session_id, &e.session_id)
                .await?;
        }
        println!("enqueued. The live worker will enrich metadata in the background.");
    } else if unenriched > 0 {
        println!(
            "note: {} rows lack LLM metadata. Re-run with --enqueue (services running) \
to have the live worker backfill it; embeddings are present either way.",
            unenriched
        );
    }
    println!("done.");
    Ok(())
}

fn load_facts(conn: &Connection) -> Result<Vec<FactCard>> {
    let mut stmt = conn.prepare(
        "SELECT id, source_agent, session_id, domain, content, confidence, tags, metadata_json
         FROM fact_cards",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(FactCard {
            id: row.get(0)?,
            source_agent: row.get(1)?,
            session_id: row.get(2)?,
            domain: row.get(3)?,
            content: row.get(4)?,
            confidence: row.get::<_, f64>(5)? as f32,
            tags: row
                .get::<_, String>(6)?
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            created_at: None,
            metadata: row
                .get::<_, Option<String>>(7)?
                .and_then(|s| serde_json::from_str(&s).ok()),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn load_episodes(conn: &Connection) -> Result<Vec<EpisodicMemory>> {
    let mut stmt = conn.prepare(
        "SELECT session_id, project_path, summary, turn_count, timestamp, task_ids, metadata_json
         FROM episodic_memories",
    )?;
    let rows = stmt.query_map([], |row| {
        Ok(EpisodicMemory {
            session_id: row.get(0)?,
            project_path: row.get(1)?,
            summary: row.get(2)?,
            turn_count: row.get::<_, i64>(3)? as u32,
            timestamp: None,
            task_ids: row
                .get::<_, String>(5)?
                .split(',')
                .map(|s| s.to_string())
                .collect(),
            metadata: row
                .get::<_, Option<String>>(6)?
                .and_then(|s| serde_json::from_str(&s).ok()),
        })
    })?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}
```

Note: `commit_facts` is an inherent `pub` method on `QdrantTier` (already exists); `record_episode` comes from the `MemoryTier` trait (hence the `use` inside main). If `tracing_subscriber` is not a dependency of koad-cass, delete the `tracing_subscriber::fmt::init();` line — it is cosmetic.

- [ ] **Step 8.2: Compile and dry-run against a scratch DB**

Run:
```bash
cargo build -p koad-cass --bin backfill_embeddings
./target/debug/backfill_embeddings --db /tmp/claude-scratch-cass-test.db 2>/dev/null || true
sqlite3 /tmp/claude-scratch-cass-test.db "CREATE TABLE IF NOT EXISTS fact_cards (id TEXT PRIMARY KEY, source_agent TEXT NOT NULL, session_id TEXT NOT NULL, domain TEXT NOT NULL, content TEXT NOT NULL, confidence REAL NOT NULL, tags TEXT NOT NULL, created_at TEXT NOT NULL, task_ids TEXT, metadata_json TEXT); CREATE TABLE IF NOT EXISTS episodic_memories (session_id TEXT PRIMARY KEY, project_path TEXT NOT NULL, summary TEXT NOT NULL, turn_count INTEGER NOT NULL, timestamp TEXT NOT NULL, task_ids TEXT NOT NULL, metadata_json TEXT);"
./target/debug/backfill_embeddings --db /tmp/claude-scratch-cass-test.db
```
Expected final output: `facts: 0`, `episodes: 0`, `dry-run: no writes...`.

- [ ] **Step 8.3: Commit**

```bash
git add crates/koad-cass/src/bin/backfill_embeddings.rs
git commit -m "feat(cass): backfill_embeddings migration binary

Drops fingerprint-vector collections, re-embeds all L2 rows with the real
embedding model. Dry-run by default."
```

---

### Task 9: Config, integration test, docs

**Files:**
- Modify: `.env.template`
- Modify: `crates/koad-cass/src/storage/tiered.rs` (integration test)
- Modify: `docs/superpowers/specs/2026-07-01-cass-semantic-enrichment-design.md` (status only)

- [ ] **Step 9.1: Document env vars**

In `.env.template`, near the existing Qdrant URL line (line ~9), add:

```bash
# Intelligence models (Ollama)
# Chat/enrichment model — fills memory metadata (salience, summary, privacy flags)
KOADOS_INTEL_MODEL=granite3.3:2b
# Embedding model — semantic vectors for Qdrant recall (dimension must stay
# consistent; changing this requires re-running backfill_embeddings)
KOADOS_EMBED_MODEL=nomic-embed-text
```

- [ ] **Step 9.2: Add end-to-end integration test (ignored by default)**

Append to the `tests` module in `crates/koad-cass/src/storage/tiered.rs`:

```rust
    /// Full pipeline: requires live Redis + Qdrant + Ollama (nomic-embed-text)
    /// AND a running enrichment worker is NOT required — this test drives the
    /// QdrantTier directly to validate semantic (non-substring) recall.
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

        // Paraphrase — zero keyword overlap with "systemd"/"binary" is not
        // required, but the phrasing differs enough that substring/LIKE
        // matching would miss it. Real embeddings must rank it first.
        let results = qdrant
            .search_semantic(
                "why did the service restart not pick up the new build",
                "clyde-semantic-test",
                3,
            )
            .await?;
        assert!(
            results.iter().any(|f| f.id == "semantic-test-001"),
            "semantic search must recall the paraphrased fact; got: {:?}",
            results.iter().map(|f| &f.id).collect::<Vec<_>>()
        );
        Ok(())
    }
```

Also add `koad-intelligence` to the test imports if the compiler asks (it is already a dependency of koad-cass).

- [ ] **Step 9.3: Full workspace verification**

Run: `cargo test --workspace --exclude koad-sandbox 2>&1 | tail -20` (drop the exclude if it errors — it mirrors CI habits)
Expected: all non-ignored tests PASS.

Run: `cargo clippy -p koad-cass -p koad-intelligence 2>&1 | tail -5`
Expected: no new warnings introduced by this work (pre-existing warnings acceptable).

- [ ] **Step 9.4: Mark spec implemented**

In `docs/superpowers/specs/2026-07-01-cass-semantic-enrichment-design.md`, change `**Status:** Approved` to `**Status:** Implemented (see docs/superpowers/plans/2026-07-01-cass-semantic-enrichment.md)`.

- [ ] **Step 9.5: Commit**

```bash
git add .env.template crates/koad-cass/src/storage/tiered.rs docs/superpowers/specs/2026-07-01-cass-semantic-enrichment-design.md
git commit -m "feat(cass): env config, paraphrase recall integration test, spec status"
```

---

## Deploy runbook (human-in-the-loop — NOT for subagents)

Sequenced; do not reorder. Requires a real terminal for sudo (known install.sh gotcha).

1. `ollama pull granite3.3:2b` (already present per `ollama list`, verify) and confirm `nomic-embed-text` present.
2. Add `KOADOS_INTEL_MODEL` / `KOADOS_EMBED_MODEL` to the live `.env`.
3. `./install.sh --update` (builds release, swaps binaries).
4. **Explicit** `sudo systemctl restart koad-cass.service koad-citadel.service` in a real terminal — install.sh's `sudo -n` restart fails silently without passwordless sudo.
5. Verify new PID + binary mtime: `readlink -f /proc/$(pgrep -f koad-cass)/exe`.
6. Run migration: `$KOAD_HOME/bin/backfill_embeddings --db $KOAD_HOME/data/db/cass.db` (dry-run) then `--apply --enqueue`.
7. Run ignored integration test or spot-check `memory.search_semantic` via MCP with a paraphrased query.
8. Watch worker logs: `journalctl -u koad-cass -f | grep EnrichmentWorker`.
9. Ops note: if Redis is down during a commit window, facts land in L2 but are
   never enqueued — after any Redis outage, run `backfill_embeddings --enqueue`
   to close the gap.

## Self-review notes (spec coverage)

- Spec §1 architecture → Tasks 3, 6, 7. Fire-and-forget removal → Task 3.2.
- Spec §2 router split + cloud-fallback removal + env vars → Tasks 1, 9.1.
- Spec §3 worker protocol (stream entry, merge policy, retry, XPENDING recovery) → Tasks 5, 6. Note: recovery uses same-consumer pending re-read ("0" id) instead of XAUTOCLAIM — equivalent for a single named consumer, simpler to implement.
- Spec §4 fingerprint deletion, Result get_vector, deferred boot, backfill (re-embed + --enqueue for unenriched rows), embedding_model payload → Tasks 2, 8.
- Spec §5 error table → Task 2 (dim/offline), Task 3 (XADD non-fatal), Task 6 (retries/pending), tiered fallthrough unchanged.
- Spec §6 testing → unit tests in Tasks 1/2/4/5; paraphrase integration test in Task 9.2; deploy verification in runbook.
- Deviation from spec: worker enriches episodes with `confidence = 1.0` sentinel (episodes lack a confidence field) — effect: episode salience only fills when empty, never overwrites. Consistent with fill-only-empty intent.
