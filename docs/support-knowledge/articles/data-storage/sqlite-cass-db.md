# SQLite Storage (cass.db)

> The persistent "cold path" memory store for KoadOS — a SQLite database managed by koad-cass that durably archives all agent knowledge (FactCards) and session retrospectives (EpisodicMemory) behind a clean trait abstraction.

**Complexity:** intermediate
**Related Articles:** [koad-cass](../core-systems/koad-cass.md), [Agent Session Lifecycle](../core-systems/agent-session-lifecycle.md)

---

## Overview

KoadOS separates "hot" and "cold" data storage. Redis handles hot data: active session state, distributed locks, and the event bus. These are ephemeral — fast reads and writes, with TTLs, flushed on restart if needed. SQLite handles cold data: permanent, queryable, archival memory that must persist indefinitely.

The primary SQLite database is `cass.db`, located at `~/.koad-os/cass.db`. It is owned and managed exclusively by the `koad-cass` service. No other component writes to `cass.db` directly — all access goes through the `Storage` trait interface provided by CASS.

`cass.db` is the foundation of KoadOS's **compounding memory**: the mechanism by which the agent swarm accumulates knowledge over time. Every significant insight an agent distills becomes a `FactCard` in this database. Every session that closes (gracefully or via crash) becomes an `EpisodicMemory` record. As the database grows, CASS's Temporal Context Hydration (TCH) can draw from an increasingly rich pool of historical context, making agents progressively smarter across sessions.

SQLite was chosen over a server-based database (like Postgres) deliberately, in alignment with KoadOS's "local-first" principle. SQLite requires no separate process, no network connection, and no configuration server. The database is a single file on disk — portable, durable, and directly inspectable with any SQLite client.

## How It Works

### The `Storage` Trait

All database interaction in CASS is abstracted behind the `Storage` trait in `crates/koad-cass/src/storage/mod.rs`:

```rust
pub trait Storage: Send + Sync {
    fn commit_fact(&self, fact: FactCard) -> Result<()>;
    fn query_facts(&self, domain: &str) -> Result<Vec<FactCard>>;
    fn query_facts_by_tags(&self, tags: &[String]) -> Result<Vec<FactCard>>;
    fn record_episode(&self, episode: EpisodicMemory) -> Result<()>;
    fn query_episodes(&self, agent: &str, limit: usize) -> Result<Vec<EpisodicMemory>>;
}
```

CASS's gRPC services hold `Arc<dyn Storage>` — a thread-safe, reference-counted handle to any type that implements `Storage`. In production, this is `CassStorage`. In tests, it's `MockStorage`. The services themselves contain no SQL and have no direct dependency on `rusqlite`. This is a clean separation of concern: the service layer handles gRPC protocol details, the storage layer handles persistence.

### `CassStorage`: The Real Implementation

`CassStorage` is the production implementation, backed by `rusqlite`. It initializes the database schema on startup using `CREATE TABLE IF NOT EXISTS` statements, so the schema is self-managed — no migration tool required for the current schema.

**Schema: `facts` table**

Stores `FactCard` records — structured knowledge distilled by agents.

| Column | Type | Description |
|--------|------|-------------|
| `id` | `TEXT PRIMARY KEY` | UUID for the fact |
| `source_agent` | `TEXT` | Name of the agent that committed this fact |
| `session_id` | `TEXT` | Session ID during which the fact was created |
| `domain` | `TEXT` | Topic area (e.g., `"redis"`, `"session-lifecycle"`) |
| `content` | `TEXT` | The knowledge content itself |
| `confidence` | `REAL` | 0.0–1.0 significance score (AI-generated if not provided) |
| `tags` | `TEXT` | JSON-serialized list of string tags for filtering |
| `created_at` | `INTEGER` | Unix timestamp |

**Schema: `episodes` table**

Stores `EpisodicMemory` records — AI-generated summaries of completed sessions.

| Column | Type | Description |
|--------|------|-------------|
| `session_id` | `TEXT PRIMARY KEY` | The session that was summarized |
| `agent_name` | `TEXT` | Name of the agent whose session this was |
| `project_path` | `TEXT` | Working directory path during the session |
| `summary` | `TEXT` | AI-generated narrative summary of session activity |
| `turn_count` | `INTEGER` | Number of turns/exchanges in the session |
| `timestamp` | `INTEGER` | Unix timestamp when the session closed |

### `MockStorage`: The Test Double

`MockStorage` in `storage/mock.rs` implements the `Storage` trait using in-memory Vecs:

```rust
pub struct MockStorage {
    facts: Mutex<Vec<FactCard>>,
    episodes: Mutex<Vec<EpisodicMemory>>,
}
```

This allows all CASS gRPC service tests to run without creating any files on disk. Tests are fast, self-contained, and don't require cleanup. The `Mutex` makes it safe to use from async tests. `MockStorage` was the architectural justification for the `Storage` trait — the trait exists specifically to enable this swap.

### How Data Gets In and Out

**Writing a FactCard** (agent commits knowledge):
1. Agent calls `MemoryService::CommitFact` gRPC
2. If no `confidence` score is provided, `MemoryService` calls `koad-intelligence` to score significance
3. `MemoryService` calls `storage.commit_fact(fact)`
4. `CassStorage` executes an `INSERT INTO facts ...` SQL statement

**Writing an EpisodicMemory** (session closed):
1. `EndOfWatchPipeline` detects a `session_closed` event on the Redis stream
2. Calls `koad-intelligence` to summarize the session
3. Calls `storage.record_episode(episode)` with the summary
4. `CassStorage` executes an `INSERT INTO episodes ...` SQL statement

**Reading for hydration** (agent booting):
1. `HydrationService::Hydrate` is called with the agent name and token budget
2. Calls `storage.query_episodes(agent_name, 5)` to get the 5 most recent session summaries
3. Calls `storage.query_facts(domain)` to get high-confidence facts for the agent's current project context
4. Packages these into the context packet, respecting the token budget

### Direct Inspection

`cass.db` is a standard SQLite file and can be queried directly:

```bash
# Open the database
sqlite3 ~/.koad-os/cass.db

# View all FactCards for a domain
SELECT source_agent, domain, content, confidence FROM facts
WHERE domain = 'redis'
ORDER BY confidence DESC;

# View recent session summaries for Tyr
SELECT session_id, summary, timestamp FROM episodes
WHERE agent_name = 'Tyr'
ORDER BY timestamp DESC
LIMIT 5;
```

This is the canonical way to audit memory contents, verify that EndOfWatch summaries are being generated, and debug context hydration issues.

## Configuration

| Key | Default | Description |
|-----|---------|-------------|
| `cass.db` path | `~/.koad-os/cass.db` | Location of the SQLite database file. Created on first CASS startup if it doesn't exist. |

There are no other configuration options for `cass.db`. The schema is managed by `CassStorage::new()` via `CREATE TABLE IF NOT EXISTS` statements. No manual migration is required for the current schema version.

## Failure Modes & Edge Cases

**`cass.db` doesn't exist on first startup.**
Not an error. `CassStorage::new()` uses `CREATE TABLE IF NOT EXISTS`, so the first connection creates the database file and initializes the schema automatically.

**Disk full — writes fail.**
`commit_fact` and `record_episode` calls return errors. CASS logs these with `tracing`. The gRPC service surfaces the error to the caller. Monitor disk space in `~/.koad-os/`. The `cass.db` file grows over time as facts and episodes accumulate; there is no automatic pruning in the current implementation.

**`cass.db` is corrupted.**
SQLite databases can be corrupted by hard shutdown during a write. Run `sqlite3 ~/.koad-os/cass.db "PRAGMA integrity_check;"` to verify integrity. If corrupted, the database may need to be rebuilt from scratch (losing accumulated memory) or restored from a backup. Regular backups of `cass.db` are recommended for production deployments.

**Concurrent writes from multiple CASS instances.**
SQLite's default write locking handles single-writer concurrency correctly. However, if multiple CASS instances are run simultaneously (which should not happen in normal operation), write contention is possible. `koad-cass` is designed to run as a single instance; running multiple instances is not a supported configuration.

**Facts table grows very large.**
There is no current pruning or archival strategy for the `facts` table. Over time, with many agents committing many facts, queries may slow. A future enhancement would add confidence-based pruning (removing low-confidence facts older than N days) or tiered archival. For now, direct SQL deletion is the manual mitigation.

## FAQ

### Q: Where are facts and memories stored permanently?
In `~/.koad-os/cass.db`, a SQLite database managed by the `koad-cass` service. The `facts` table stores `FactCard` records (structured knowledge), and the `episodes` table stores `EpisodicMemory` records (AI-generated session summaries). Both tables persist indefinitely unless manually deleted. You can inspect them directly with any SQLite client: `sqlite3 ~/.koad-os/cass.db`.

### Q: How can I look at the contents of the memory database?
Use the `sqlite3` command-line tool: `sqlite3 ~/.koad-os/cass.db`. From there, use standard SQL queries. To see recent facts: `SELECT * FROM facts ORDER BY created_at DESC LIMIT 20;`. To see session summaries: `SELECT agent_name, summary FROM episodes ORDER BY timestamp DESC LIMIT 5;`. The schema is straightforward — no foreign keys, no views, just two flat tables.

### Q: What is the schema for the `facts` table?
The `facts` table has columns: `id` (UUID, primary key), `source_agent` (agent name), `session_id` (session that created it), `domain` (topic area), `content` (the knowledge text), `confidence` (0.0–1.0 AI-scored significance), `tags` (JSON list of strings), and `created_at` (Unix timestamp). See the full schema definition in `crates/koad-cass/src/storage/mod.rs`.

### Q: Why use SQLite instead of a bigger database like Postgres?
KoadOS follows a "local-first" principle. SQLite requires no separate server, no network port, no authentication configuration, and no background process — the entire database is a single file. For the scale KoadOS operates at (a small number of agents producing hundreds to thousands of records over time), SQLite is more than sufficient and dramatically simpler to operate. If the system scales to the point where SQLite becomes a bottleneck, the `Storage` trait abstraction makes it straightforward to swap in a different backend without touching the service layer.

### Q: What is the purpose of the `Storage` trait?
The `Storage` trait exists to decouple the CASS gRPC services from the database implementation. Services hold `Arc<dyn Storage>` and call methods like `commit_fact()` without any SQL. This means: (1) all SQL is centralized in one file (`storage/mod.rs`), making the schema easy to find and modify; (2) tests can inject `MockStorage` (in-memory Vecs) instead of `CassStorage`, making tests fast and file-system-free; (3) a future storage backend (e.g., a vector database like Qdrant) can be implemented as another `Storage` implementor without changing the service code.

## Source Reference

- `crates/koad-cass/src/storage/mod.rs` — `Storage` trait, `CassStorage` struct; all SQL schema and query logic
- `crates/koad-cass/src/storage/mock.rs` — `MockStorage`; in-memory test double
- `crates/koad-proto/src/cass.proto` — `FactCard` and `EpisodicMemory` protobuf message definitions
- `crates/koad-cass/src/services/hydration.rs` — Primary reader; uses `Storage` to load facts and episodes for TCH
- `crates/koad-cass/src/services/eow.rs` — Primary writer for episodes; the EndOfWatch pipeline
