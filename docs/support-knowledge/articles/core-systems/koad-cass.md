# koad-cass: The Agent Support System

> The persistent gRPC service that forms the "Brain" of KoadOS — managing long-term memory, intelligent context hydration, code graph queries, and end-of-session summarization for the agent swarm.

**Complexity:** advanced
**Related Articles:** [The Tri-Tier Model](../architecture/tri-tier-model.md), [koad-citadel](./koad-citadel.md), [SQLite Storage (cass.db)](../data-storage/sqlite-cass-db.md), [Agent Session Lifecycle](./agent-session-lifecycle.md)

---

## Overview

`koad-cass` — the Citadel Agent Support System — is the cognitive layer of KoadOS. While the [Citadel](./koad-citadel.md) manages whether an agent *exists* (sessions, security, infrastructure), CASS manages what an agent *knows* and *remembers* (memory, context, code understanding).

CASS is a separate `tonic`-based gRPC server running on its own port (e.g., `127.0.0.1:50052`). Agents generally do not call CASS directly — the Citadel orchestrates CASS on their behalf. CASS is purpose-built for slower, more cognitively expensive operations: it calls AI inference models, performs filesystem walks, runs code graph queries, and writes to SQLite. The Citadel remains fast and low-latency by delegating all of this to CASS.

The core services CASS provides are:
- **MemoryService** — commit and query structured long-term knowledge (`FactCard`, `EpisodicMemory`)
- **HydrationService** — build the initial context packet an agent receives at boot (Temporal Context Hydration, TCH)
- **SymbolService** — query and re-index the project's code graph
- **EndOfWatchPipeline** — automatically summarize sessions as they close (background task)

Together, these services form the "cognitive loop" that allows agents to accumulate knowledge over time rather than starting cold with each new session.

## How It Works

### Architecture and Storage

CASS starts in `main.rs`, initializing:
- `CassStorage` — the SQLite abstraction layer (`cass.db`)
- `InferenceRouter` — routes AI inference calls to the configured provider (via `koad-intelligence`)
- `CodeGraph` — the tree-sitter-backed code graph (via `koad-codegraph`)
- All gRPC service handlers

All services share these components via `Arc<dyn Storage>` and `Arc<InferenceRouter>` — safe, reference-counted handles for concurrent access across async tasks.

The `Storage` trait is central to CASS's architecture:

```rust
// crates/koad-cass/src/storage/mod.rs
pub trait Storage: Send + Sync {
    fn commit_fact(&self, fact: FactCard) -> Result<()>;
    fn query_facts(&self, domain: &str) -> Result<Vec<FactCard>>;
    fn record_episode(&self, episode: EpisodicMemory) -> Result<()>;
    // ...
}
```

Services call methods on `Arc<dyn Storage>`. In production, this is `CassStorage` backed by `rusqlite`. In unit tests, it's `MockStorage` backed by in-memory Vecs — no file I/O, fast tests. This trait boundary cleanly separates service logic from persistence logic.

### Service: `MemoryService`

MemoryService is the interface to CASS's long-term memory, which lives in the `facts` and `episodes` tables of `cass.db`.

**`CommitFact`**: Saves a `FactCard` to the `facts` table. A `FactCard` is a distilled piece of structured knowledge — something an agent has learned and wants to remember across sessions. Fields include `domain` (topic area), `content` (the knowledge itself), `tags`, and `confidence` (a 0.0–1.0 significance score). If no confidence score is provided, CASS automatically calls `koad-intelligence` to score the fact's significance before storing it.

**`QueryFacts`**: Retrieves facts from `cass.db` filtered by domain and/or tags. This is how an agent asks "what do I know about Redis?" or "what have I learned about the session lifecycle?".

**`RecordEpisode`**: Saves an `EpisodicMemory` record (the EndOfWatch summary for a session). Typically called by the `EndOfWatchPipeline` automatically, not by agents directly.

### Service: `HydrationService` — Temporal Context Hydration (TCH)

TCH is CASS's primary value-add at boot time. The problem it solves: an agent starting a new session has no in-memory context. Loading the entire codebase would exhaust its token budget and include enormous amounts of irrelevant material. TCH builds a *curated, budget-aware context packet* by intelligently selecting the most relevant information.

The `Hydrate` RPC accepts a `HydrationRequest` containing:
- `agent_name` — whose memory to load
- `workspace_level` — from the [Workspace Hierarchy](../architecture/workspace-hierarchy.md), determines the scope of the file walk
- `token_budget` — maximum tokens to use for the context packet

The hydration process performs a **"Hierarchy Walk"**:
1. Load recent `EpisodicMemory` records (the agent's last few session summaries)
2. Load high-confidence `FactCard`s (sorted by `confidence` descending) for the agent
3. Walk the filesystem at the appropriate hierarchy level — for Outpost level, this means the current repo's `AGENTS.md`, local `.agents/` folder, and recent git activity
4. Add material until the `token_budget` is exhausted, prioritizing recency and confidence

The result is a `HydrationResponse` containing the packed context. This is what allows an agent to boot and immediately "remember" where it left off — without re-reading the entire codebase.

### Service: `SymbolService`

The SymbolService provides code intelligence backed by `koad-codegraph` (which uses tree-sitter for parsing):

**`Query`**: Given a symbol name (function, struct, trait), returns its definition location and signature. Useful for agents that need to find where a specific piece of code lives without a full codebase search.

**`IndexProject`**: Triggers a full re-indexing of the project's source code. This is CPU-intensive (tree-sitter parses every file), so it's executed via `tokio::spawn_blocking` to avoid blocking the async runtime — a direct application of the [RUST_CANON](../protocols/rust-canon.md) Non-Blocking Rule.

> **Note:** The SymbolService depends on `koad-codegraph`, which is listed as a Phase 4 component. If the code graph has not been indexed (i.e., `IndexProject` has not been called since the last significant codebase change), `Query` results may be stale or empty.

### Background Task: `EndOfWatchPipeline`

The EndOfWatch (EOW) pipeline is not a gRPC service — it's a long-running background task spawned when CASS starts. It:

1. Subscribes to the `koad:stream:system` Redis stream (via the Citadel's Signal bus)
2. Listens for `session_closed` events
3. When a session closes, retrieves the session's activity log
4. Calls `koad-intelligence` to generate an AI-powered summary of the session: what was accomplished, what was learned, what problems were encountered
5. Saves the summary as an `EpisodicMemory` via `MemoryService::RecordEpisode`

This pipeline is what closes the cognitive loop. Without it, agents would have sessions disappear into the void. With it, every session — including crashed ones (which fire a `session_closed` event via the Citadel's reaper) — becomes a permanent memory entry that future sessions can learn from.

The pipeline was implemented in Phase 3 to bridge the "Distillation Gap": the period where sessions closed but were never summarized.

## Configuration

| Key | Location | Description |
|-----|----------|-------------|
| `cass_grpc_addr` | `config/kernel.toml` | Address CASS listens on (e.g., `127.0.0.1:50052`) |
| `cass.db` | `~/.koad-os/cass.db` | The SQLite database file managed by `CassStorage` |

> **Note:** `cass_grpc_addr` is defined in `config/kernel.toml` but may not yet be referenced by the Citadel for routing in the current implementation. Cross-service routing between the Citadel and CASS is actively being wired up.

## Failure Modes & Edge Cases

**CASS is down at boot time.**
If CASS is unavailable when an agent boots, the Citadel can still create the session lease — session management does not depend on CASS. The agent loses its hydration context packet, which means it starts without its accumulated memory. Operations that require memory queries or code graph access will fail. CASS can be restarted without disrupting active sessions.

**`EndOfWatchPipeline` misses a session closure.**
If CASS restarts after a `session_closed` event was emitted to the Redis stream but before EOW consumed it, the event may be missed depending on the Redis stream consumer configuration. This would result in a session with no `EpisodicMemory` record. The session's raw log might still exist and could be manually re-processed. Consumer group configuration for the `koad:stream:system` stream is the long-term fix.

**Code graph is stale.**
`SymbolService::Query` returns results from the last indexed state. If significant changes have been made to the codebase without re-running `IndexProject`, symbol lookups may return incorrect locations or miss new symbols entirely. Re-indexing is the fix; for large codebases it can take tens of seconds.

**`CassStorage` fails to write (disk full, permissions).**
`CommitFact` and `RecordEpisode` calls will return errors. CASS logs these with the `tracing` crate. The calling service should surface the error to the agent. The `cass.db` file lives at `~/.koad-os/cass.db` by default — ensure the KoadOS home directory has adequate disk space.

## FAQ

### Q: What is CASS and how is it different from the Citadel?
The Citadel manages infrastructure: sessions, security, locks, and the event bus. CASS manages cognition: memory storage, context building, and code understanding. The Citadel is fast and stateless-ish (backed by Redis). CASS is slower and stateful (backed by SQLite and AI inference). The Citadel answers "can this agent act?" — CASS answers "what does this agent know?". In the [Tri-Tier Model](../architecture/tri-tier-model.md), the Citadel is the Body and CASS is the Brain.

### Q: How does the system decide what information to give an agent when it starts up?
CASS's `HydrationService` builds the context packet using Temporal Context Hydration (TCH). It takes the agent's name, workspace level, and a token budget, then performs a Hierarchy Walk: loading recent session summaries (`EpisodicMemory`), high-confidence facts (`FactCard`), and relevant local files (in priority order, from the `AGENTS.md` outward), stopping when the token budget is exhausted. The result is a curated snapshot of the agent's relevant history and local context — enough to resume work without re-reading the entire codebase.

### Q: Where are agent memories actually stored?
In `cass.db`, a SQLite database at `~/.koad-os/cass.db`. The `facts` table holds `FactCard` records (structured knowledge); the `episodes` table holds `EpisodicMemory` records (session summaries). Both tables are managed exclusively by the `CassStorage` struct in `crates/koad-cass/src/storage/mod.rs`. The database can be queried directly with any SQLite client for inspection.

### Q: What happens at the "EndOfWatch"?
When a session closes (either via `koad logout` or via the Citadel's reaper purging a crashed session), a `session_closed` event fires on the `koad:stream:system` Redis stream. CASS's `EndOfWatchPipeline` background task detects this event, retrieves the session's activity log, calls `koad-intelligence` to generate an AI summary of what the session accomplished, and saves the result as an `EpisodicMemory` record in `cass.db`. On the agent's next boot, TCH includes this summary in the hydration packet, so the agent knows what it did last time.

### Q: How can I query the code graph to find where a function is?
Use the `SymbolService::Query` RPC with the symbol name. This returns the definition location and signature. Note that the code graph must be indexed first — call `SymbolService::IndexProject` to build or refresh the index. The index is backed by tree-sitter parsing, managed by the `koad-codegraph` crate. The index persists across CASS restarts but must be refreshed after significant codebase changes.

## Source Reference

- `crates/koad-cass/src/main.rs` — Binary entry point; initializes all components and starts the gRPC server
- `crates/koad-cass/src/storage/mod.rs` — `Storage` trait and `CassStorage` struct; all SQLite logic and schema
- `crates/koad-cass/src/storage/mock.rs` — `MockStorage`; in-memory test double for the `Storage` trait
- `crates/koad-cass/src/services/hydration.rs` — `CassHydrationService`; TCH implementation and Hierarchy Walk
- `crates/koad-cass/src/services/eow.rs` — `EndOfWatchPipeline`; background session summarization
- `proto/cass.proto` — gRPC service and message definitions (MemoryService, HydrationService, SymbolService)
