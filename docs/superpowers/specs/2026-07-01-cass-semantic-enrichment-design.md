# CASS Semantic Enrichment Pipeline — Design

**Date:** 2026-07-01
**Author:** Clyde (Officer, Claude Code) with Dood approval
**Status:** Approved
**Scope:** Real semantic embeddings for Qdrant L3 recall + async LLM metadata enrichment of agent memories

## Problem

Semantic memory recall in CASS is a placeholder. `QdrantTier` stores 32-dim
content-hash fingerprint vectors whenever the embedding path is unavailable, and
the primary embedding path routes through the general chat model (`mistral` via
`KOADOS_INTEL_MODEL`), which is slow and produces weak-quality 4096-dim
embeddings. Existing Qdrant collections are polluted with fingerprint vectors,
so `SearchSemantic` results are effectively content-hash matches, not semantic
matches.

Separately, the rich `MemoryMetadata` schema (proto `cass.proto`:
`RetrievalMetadata`, `PrivacyMetadata`, `summary`) is only populated with
deterministic token estimates at the gRPC commit choke point
(`crates/koad-cass/src/services/memory.rs`). Fields that require judgment —
salience, volatility, audience, privacy flags, summaries, tags — are never
filled. The router primitives for this (`summarize()`, `score_significance()`)
exist in `koad-intelligence` but are unused at commit.

## Goals

1. Real semantic embeddings via a dedicated local model: `nomic-embed-text`
   (768-dim, already pulled, 274 MB).
2. LLM metadata enrichment via a lightweight local model: `granite3.3:2b`
   (promotable to `phi4-mini:3.8b` if quality is insufficient).
3. Async pipeline: `memory.commit` never waits on an LLM (Option A, approved).
4. No silent corruption: fingerprint vectors are never written again.
5. Migration path for existing memories.

## Non-Goals

- Cloud embedding fallback (dropped for dimension consistency; see §2).
- Batch/parallel embedding throughput optimization (serial worker first).
- Re-ranking, hybrid search scoring changes, or recall API changes.
- Cross-agent memory routing decisions (enrichment fills metadata; it does not
  re-partition memories).

## 1. Architecture

```
agent → memory.commit (gRPC)
          ├─ sync: token estimates → L1 Redis + L2 SQLite  (unchanged, fast)
          └─ XADD → Redis stream cass:enrichment
                        │
          enrichment worker (tokio task inside koad-cass)
                        ├─ granite3.3:2b → metadata (summary, salience, tags, privacy)
                        ├─ nomic-embed-text → 768-dim vector
                        ├─ update metadata in L2 SQLite (+ L1 refresh)
                        └─ upsert Qdrant L3 (real embedding + embedding_model payload)
```

- L2 SQLite remains the source of truth for memory content and metadata.
- Qdrant L3 becomes a pure semantic index, written ONLY by the enrichment
  worker and the backfill binary.
- The existing fire-and-forget L3 write in `TieredStorage::commit_fact` /
  `record_episode` (`crates/koad-cass/src/storage/tiered.rs`) is **removed** —
  it is the path that writes fingerprint vectors today. The L3 write moves to
  the worker, post-embedding.

## 2. Router model split (`crates/koad-intelligence`)

- New `InferenceTask::Embedding` variant.
- `InferenceRouter` holds a second `OllamaClient` dedicated to embeddings,
  configured by `KOADOS_EMBED_MODEL` (default `nomic-embed-text`).
- Chat/enrichment tasks stay on `KOADOS_INTEL_MODEL`; its default changes from
  `mistral` to `granite3.3:2b`.
- `InferenceRouter::embed()` routes to the embedding client. The
  Gemini/OpenRouter cloud fallback for embeddings is **removed**: cloud models
  emit different dimensions than the local collection (768), so a fallback
  vector is unusable. Embedding is local-only; failures propagate as errors and
  are retried by the worker.
- `.env.template` documents both variables.

## 3. Enrichment worker (`crates/koad-cass`)

- Tokio task spawned in `main.rs` after tier initialization.
- Redis stream `cass:enrichment`, consumer group `cass-enrichers`, blocking
  `XREADGROUP`, serial processing (one memory at a time — respects local VRAM
  constraints per the ollama delegation matrix).
- Stream entry: `{ kind: "fact" | "episode", id, partition }`.
- Per item:
  1. Load full record from L2 SQLite.
  2. One JSON-output prompt to the enrichment model producing: `summary`,
     `salience` (0.0–1.0), `volatility` (`stable|mutable|ephemeral`),
     `audience` (`self|agent|team|global`), `sensitivity`
     (`public|internal|private|secret-adjacent`), `contains_pii`,
     `contains_secret`, `tags` (string list).
  3. Parse with serde. Merge policy: fill ONLY empty/default metadata fields —
     agent-supplied values are never overwritten. Tags land in
     `metadata_json`.
  4. Embed content via `InferenceTask::Embedding` → upsert Qdrant point with a
     new `embedding_model: "nomic-embed-text"` payload field.
  5. Persist updated metadata to L2; refresh L1 copy.
  6. `XACK` on success.
- Failure handling: retry ×3 with backoff. If enrichment JSON is garbage after
  retries, keep token-only metadata and still attempt the embed + upsert. If
  the embed itself fails (Ollama down), leave the entry pending — it is
  re-claimed on worker restart via `XPENDING`/`XAUTOCLAIM`. Nothing is lost;
  no fake vectors are written.

## 4. Qdrant changes + migration (`crates/koad-cass/src/storage/qdrant_tier.rs`)

- `content_vector()` fingerprint generation is **deleted** from the write path.
- `get_vector()` returns `Result<Vec<f32>>`; errors propagate.
- Query path: `search_semantic` embeds the query via the embedding model. On
  error it returns `Err`, and `TieredStorage::search_semantic` already falls
  through to L2 LIKE search — graceful degradation preserved.
- Dimension is still auto-detected at boot from the embedding client (768 for
  nomic). On mismatch with an existing collection: hard error, refuse writes.
- If detection fails at boot (Ollama down), the 32-dim fallback default is
  removed: `QdrantTier` starts in deferred mode — no writes, queries return
  `Err` (falling through to L2) — and retries detection on first worker use.
- New binary `crates/koad-cass/src/bin/backfill_embeddings.rs` (pattern:
  `backfill_metadata`):
  1. Drops and recreates Qdrant collections at the detected dimension.
  2. Walks all L2 SQLite rows (facts + episodes), re-embeds, re-upserts.
  3. Enqueues `cass:enrichment` entries for rows missing LLM metadata.
  - Run once at deploy, after services restart on the new binary.

## 5. Error handling summary

| Failure | Behavior |
|---|---|
| Ollama down at commit | Commit succeeds; item waits in stream |
| Ollama down at query | Semantic search falls through to L2 LIKE |
| LLM emits garbage JSON | Retry ×3, then token-only metadata; embed still attempted |
| Worker crash mid-item | Pending stream entry re-claimed on restart |
| Dimension mismatch at boot | Hard error, refuse Qdrant writes (no silent poison) |
| Redis down | Commit's XADD fails → log warn; memory safe in L2; backfill binary can re-enqueue |

## 6. Testing

- **Unit:** mock `InferenceClient` for enrichment prompt/parse/merge logic
  (garbage JSON, partial fields, agent-supplied fields preserved). Existing
  Qdrant payload round-trip tests extended with `embedding_model`.
- **Integration** (`#[ignore]`, requires live Redis/Qdrant/Ollama): commit a
  fact → poll until the Qdrant point exists with a 768-dim vector → verify
  `search_semantic` returns it for a paraphrased (non-substring) query.
- **Deploy verification:** run `backfill_embeddings`, then spot-check recall
  quality through the `memory.search_semantic` MCP tool. Follow the known
  install.sh gotcha: explicit `sudo systemctl restart koad-cass.service
  koad-citadel.service` in a real terminal after binary swap.

## Configuration

| Variable | Default | Purpose |
|---|---|---|
| `KOADOS_INTEL_MODEL` | `granite3.3:2b` (was `mistral`) | Chat/enrichment model |
| `KOADOS_EMBED_MODEL` | `nomic-embed-text` | Embedding model |

## Decisions log

- **Async enrichment (Option A)** — approved by Dood 2026-07-01. Commit stays
  fast; worker enriches in background.
- **Approach 1: Redis Stream + in-process worker** — approved by Dood
  2026-07-01. Durable queue without a new deploy surface.
- **Local-only embeddings** — dimension consistency over availability; cloud
  fallback removed for the embed path only (chat fallbacks unchanged).
- **Fingerprint vectors eliminated from writes** — queue-and-retry replaces
  fake-vector fallback.
