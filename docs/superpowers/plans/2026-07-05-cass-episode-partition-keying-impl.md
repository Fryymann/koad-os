# CASS Episode Partition Keying — Implementation Plan (Option A)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **Methodology: SPEC-DRIVEN (per Dood, 2026-07-05).** Each task states its spec first, then implementation, then verification. Tests are written to verify spec compliance *after* the implementation step — do not reorder into test-first.

**Goal:** Give episodes an explicit `partition` key end-to-end (proto → SQLite → Qdrant payload → both read filters → backfill), closing the semantic-recall gap where `sid.contains(partition)` matches nothing.

**Architecture:** `EpisodicMemory` is proto-generated (`proto/cass.proto:105`), so the field lands once in the proto and flows into every crate. Partition canon is `{agent}_{host}_{user}` (e.g. `clyde_Jupiter_ideans`), produced by a new `koad_core::utils::partition::partition_key()` helper. Write paths (Citadel session-close event, EOW pipeline, koad-cli session sync) stamp it; storage tiers persist it; the two read filters switch to partition equality with a legacy substring fallback for unbackfilled rows; a backfill binary migrates the 46 production rows.

**Design authority:** `docs/superpowers/plans/2026-07-05-cass-episode-partition-keying-design.md` (Condition Green 2026-07-05).

**Tech Stack:** Rust workspace, prost/tonic proto codegen, rusqlite, qdrant-client, Ollama embeddings (nomic-embed-text).

---

### Task 1: Proto field + partition helper + compile-fix all constructors

**Files:**
- Modify: `proto/cass.proto:105-113` (EpisodicMemory message)
- Create: `crates/koad-core/src/utils/partition.rs`
- Modify: `crates/koad-core/src/utils/mod.rs` (register module)
- Modify: every `EpisodicMemory {` struct literal (known: `crates/koad-cass/src/services/eow.rs:87`, `crates/koad-cli/src/handlers/system.rs:~1565`, `crates/koad-cass/src/storage/qdrant_tier.rs:390`; find the rest by grep)

**Spec:**
1. `EpisodicMemory` gains `string partition = 8;` — additive, backward-compatible (old clients ignore it; missing value decodes as `""`).
2. Partition canon: `partition_key(agent) = "{agent}_{host}_{user}"`. Host resolution order: `KOAD_HOST` env → `/etc/hostname` (trimmed) → `HOSTNAME` env → `"unknown-host"`. User: `USER` env → `"unknown-user"`.
3. After this task the workspace compiles; construction sites that don't yet know the real partition pass `String::new()` (wired in Task 2). Empty string is the canonical "legacy/unknown" marker everywhere.

- [ ] **Step 1: Add proto field**

In `proto/cass.proto`, message `EpisodicMemory`, after `MemoryMetadata metadata = 7;`:

```proto
  string partition = 8; // partition key canon: "{agent}_{host}_{user}"; "" = legacy/unbackfilled
```

- [ ] **Step 2: Create the helper**

`crates/koad-core/src/utils/partition.rs`:

```rust
//! Partition key canon: `{agent}_{host}_{user}` (e.g. `clyde_Jupiter_ideans`).
//! Matches the `AGENT_PARTITION` convention used by koad-os-mcp.

fn host() -> String {
    if let Ok(h) = std::env::var("KOAD_HOST") {
        if !h.is_empty() {
            return h;
        }
    }
    if let Ok(h) = std::fs::read_to_string("/etc/hostname") {
        let h = h.trim();
        if !h.is_empty() {
            return h.to_string();
        }
    }
    std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown-host".into())
}

fn user() -> String {
    std::env::var("USER").unwrap_or_else(|_| "unknown-user".into())
}

/// Canonical partition key for an agent on this instance.
pub fn partition_key(agent: &str) -> String {
    format!("{}_{}_{}", agent, host(), user())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_key_has_three_segments_prefixed_by_agent() {
        let key = partition_key("clyde");
        assert!(key.starts_with("clyde_"));
        assert!(key.split('_').count() >= 3);
    }
}
```

Register in `crates/koad-core/src/utils/mod.rs`: add `pub mod partition;` alongside the existing modules.

- [ ] **Step 3: Compile-fix every EpisodicMemory literal**

Run: `grep -rn 'EpisodicMemory {' /home/ideans/koados-citadel/crates/ --include='*.rs'`

For each site add a `partition` field:
- `crates/koad-cass/src/storage/qdrant_tier.rs:390` (`payload_to_episode`): `partition: get_str("partition").unwrap_or_default(),` (legacy Qdrant points lack the key — must not fail the decode).
- `crates/koad-cass/src/services/eow.rs:87` and `crates/koad-cli/src/handlers/system.rs:~1565`: `partition: String::new(),` (real value wired in Task 2).
- Test fixtures / mocks: `partition: String::new(),` unless the test is partition-specific.

- [ ] **Step 4: Verify workspace compiles + helper test passes**

Run: `~/.cargo/bin/cargo check --workspace 2>&1 | tail -3`
Expected: `Finished`, zero errors.
Run: `~/.cargo/bin/cargo test -p koad-core partition 2>&1 | grep 'test result'`
Expected: `test result: ok` with ≥1 passed.

- [ ] **Step 5: Commit**

```bash
git add proto/cass.proto crates/koad-core/src/utils/partition.rs crates/koad-core/src/utils/mod.rs crates/koad-cass crates/koad-cli
git commit -m "feat(cass): EpisodicMemory partition field + partition_key canon helper

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 2: Write paths stamp the partition

**Files:**
- Modify: `crates/koad-citadel/src/services/session.rs:394-399` (session_closed broadcast)
- Modify: `crates/koad-cass/src/services/eow.rs:61-98` (process_session_close)
- Modify: `crates/koad-cli/src/handlers/system.rs:~1565` (session sync episode literal)

**Spec:**
1. The `session_closed` broadcast JSON gains a `partition` field computed by `partition_key(agent_name)` at the source of truth (Citadel session service).
2. EOW pipeline prefers the event's `partition`; when absent/empty (older Citadel binary), derives it locally via `partition_key(agent_name)`. Episodes are never recorded with an empty partition by this path.
3. The koad-cli session-sync path derives via `partition_key(agent_name)` directly.

- [ ] **Step 1: Extend the broadcast**

In `crates/koad-citadel/src/services/session.rs` (inside the `if let Some(record) = record_opt` block, currently ~line 393), compute the partition and add it to the JSON:

```rust
        if let Some(record) = record_opt {
            let partition = koad_core::utils::partition::partition_key(&record.agent_name);
            let _ = self.signal_corps.broadcast(
                "system",
                &format!("{{\"event_type\": \"session_closed\", \"session_id\": \"{}\", \"agent_name\": \"{}\", \"partition\": \"{}\"}}", sid, record.agent_name, partition),
                "EOW-TRIGGER",
                "citadel",
            ).await;
```

- [ ] **Step 2: EOW consumes it (with fallback)**

In `crates/koad-cass/src/services/eow.rs`, `process_session_close`, after the `agent_name` extraction (line 63):

```rust
        let partition = match event["partition"].as_str() {
            Some(p) if !p.is_empty() => p.to_string(),
            _ => koad_core::utils::partition::partition_key(agent_name),
        };
```

and in the `EpisodicMemory` literal replace `partition: String::new(),` with `partition,`.

- [ ] **Step 3: CLI sync path**

In `crates/koad-cli/src/handlers/system.rs` episode literal, replace `partition: String::new(),` with:

```rust
                                    partition: koad_core::utils::partition::partition_key(&agent_name),
```

(`agent_name` is already in scope — it feeds the summary `format!` just above.)

- [ ] **Step 4: Verify**

Run: `~/.cargo/bin/cargo check --workspace 2>&1 | tail -3`
Expected: `Finished`, zero errors, no new warnings.

- [ ] **Step 5: Commit**

```bash
git add crates/koad-citadel/src/services/session.rs crates/koad-cass/src/services/eow.rs crates/koad-cli/src/handlers/system.rs
git commit -m "feat(cass): stamp partition on episode write paths

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Storage tiers persist the partition

**Files:**
- Modify: `crates/koad-cass/src/storage/sqlite_tier.rs` (CREATE TABLE ~:34, ALTER block ~:45, `record_episode` :279, every `episodic_memories` SELECT + row mapping)
- Modify: `crates/koad-cass/src/storage/qdrant_tier.rs` (`make_episode_payload` :314)
- Modify: `crates/koad-cass/src/storage/redis_tier.rs` (episode JSON serializer/deserializer, if episodes are cached — verify by grep)

**Spec:**
1. SQLite column `partition TEXT NOT NULL DEFAULT ''` — present in fresh CREATE TABLE and added to existing DBs via the same idempotent-ALTER idiom as `metadata_json`.
2. Every episode INSERT writes it; every episode SELECT reads it into the struct.
3. Qdrant episode payload always carries `partition` (empty string allowed — marks legacy until backfilled).

- [ ] **Step 1: SQLite schema**

In the `CREATE TABLE IF NOT EXISTS episodic_memories` statement add after `task_ids TEXT NOT NULL`:

```sql
                task_ids TEXT NOT NULL,
                partition TEXT NOT NULL DEFAULT ''
```

In the idempotent-migrations block (after the existing two ALTERs, ~line 48):

```rust
        let _ = conn.execute(
            "ALTER TABLE episodic_memories ADD COLUMN partition TEXT NOT NULL DEFAULT ''",
            [],
        );
```

- [ ] **Step 2: SQLite writes and reads**

`record_episode` (:279) — extend column list and params:

```rust
            "INSERT OR REPLACE INTO episodic_memories
             (session_id, project_path, summary, turn_count, timestamp, task_ids, metadata_json, partition)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                episode.session_id,
                episode.project_path,
                episode.summary,
                episode.turn_count,
                timestamp,
                task_ids,
                metadata_json,
                episode.partition
            ],
```

Then: `grep -n 'FROM episodic_memories' crates/koad-cass/src/storage/sqlite_tier.rs` — for EVERY select (known: `get_episode_by_session` :89, the episode listing in `search_semantic`/`query_recent_episodes` region ~:342, and any enrichment-worker accessor), append `partition` to the column list and map it into the struct (`partition: row.get(N)?` where N is its 0-based position). Keep column order consistent: always append `partition` last.

- [ ] **Step 3: Qdrant payload**

In `make_episode_payload` (:314), after the `task_ids` insert:

```rust
        p.insert(
            "partition".into(),
            Value {
                kind: Some(Kind::StringValue(episode.partition.clone())),
            },
        );
```

(`payload_to_episode` already reads it via Task 1 Step 3.)

- [ ] **Step 4: Redis tier**

Run: `grep -n 'episode' crates/koad-cass/src/storage/redis_tier.rs`
If episodes are JSON-serialized there (the `kind`/`id` cache comment at :28 suggests they may be), add `"partition": episode.partition` to the serializer and `partition: v["partition"].as_str().unwrap_or_default().to_string()` to the deserializer. If only facts are cached, this step is a no-op — note that in the commit body.

- [ ] **Step 5: Verify — spec-compliance round-trip test**

Add to the existing test module in `crates/koad-cass/src/storage/sqlite_tier.rs` (or the tier test module where episode tests live — grep `record_episode` in `#[cfg(test)]` blocks):

```rust
    #[tokio::test]
    async fn episode_partition_roundtrip() -> anyhow::Result<()> {
        let tier = SqliteTier::new(":memory:")?;
        let episode = EpisodicMemory {
            session_id: "SID-eptest-0001".into(),
            project_path: "/tmp/eptest".into(),
            summary: "test".into(),
            turn_count: 1,
            timestamp: None,
            task_ids: vec![],
            metadata: None,
            partition: "eptest_Host_user".into(),
        };
        tier.record_episode(episode).await?;
        let loaded = tier.get_episode_by_session("SID-eptest-0001").await?.unwrap();
        assert_eq!(loaded.partition, "eptest_Host_user");
        Ok(())
    }
```

(Adjust the `SqliteTier::new` construction to match the existing test idiom in that file.)

Run: `~/.cargo/bin/cargo test -p koad-cass episode_partition_roundtrip 2>&1 | grep 'test result'`
Expected: `ok. 1 passed`.

- [ ] **Step 6: Full crate check + commit**

Run: `~/.cargo/bin/cargo test -p koad-cass 2>&1 | grep 'test result' | grep -v ' 0 failed' | wc -l` → `0`

```bash
git add crates/koad-cass/src/storage/
git commit -m "feat(cass): persist episode partition in SQLite, Qdrant payload, Redis cache

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: Read paths filter on partition (with legacy fallback)

**Files:**
- Modify: `crates/koad-cass/src/storage/qdrant_tier.rs:608-616` (search_semantic_scored episode filter)
- Modify: `crates/koad-cass/src/storage/qdrant_tier.rs:527` (query_recent_episodes filter)

**Spec:**
1. `search_semantic_scored`: an episode point matches when its `partition` payload equals the query partition exactly. Points with empty/missing `partition` (pre-backfill) fall back to the old behavior against the *agent-name prefix* of the partition (`clyde` from `clyde_Jupiter_ideans`) — so recall degrades gracefully instead of to zero during rollout.
2. `query_recent_episodes(agent_name)` receives a bare agent name (hydration path). An episode matches when its partition starts with `{agent_name}_`; empty-partition episodes fall back to the old `session_id.contains(agent_name)`.
3. After the Task 6 backfill both fallbacks become dead paths but stay in place (harmless, self-documenting).

- [ ] **Step 1: search_semantic_scored episode filter**

Replace the filter closure at :608-616:

```rust
                    .filter(|p| {
                        let ep_partition = match p.payload.get("partition").and_then(|v| v.kind.as_ref()) {
                            Some(Kind::StringValue(s)) => s.as_str(),
                            _ => "",
                        };
                        if !ep_partition.is_empty() {
                            return ep_partition == partition;
                        }
                        // Legacy point (pre-backfill): fall back to matching the
                        // agent-name prefix of the partition against the session id.
                        if let Some(Kind::StringValue(sid)) =
                            p.payload.get("session_id").and_then(|v| v.kind.as_ref())
                        {
                            let agent = partition.split('_').next().unwrap_or(partition);
                            sid.contains(agent)
                        } else {
                            false
                        }
                    })
```

Update the comment above it (`// Filter locally by agent partition since EpisodicMemory does not store partition natively` at :605) to:

```rust
                // Filter by the partition payload field; legacy points without it
                // fall back to agent-prefix substring matching until backfilled.
```

- [ ] **Step 2: query_recent_episodes filter**

Replace `.filter(|ep| ep.session_id.contains(agent_name))` at :527:

```rust
            .filter(|ep| {
                if !ep.partition.is_empty() {
                    ep.partition.starts_with(&format!("{}_", agent_name))
                } else {
                    ep.session_id.contains(agent_name)
                }
            })
```

- [ ] **Step 3: Verify + commit**

Run: `~/.cargo/bin/cargo test -p koad-cass 2>&1 | grep 'test result' | grep -v ' 0 failed' | wc -l` → `0`

```bash
git add crates/koad-cass/src/storage/qdrant_tier.rs
git commit -m "fix(cass): episode recall filters on partition key, legacy substring fallback

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 5: Partition-aware live integration test

**Files:**
- Modify: `crates/koad-cass/src/storage/tiered.rs` (test module, after `test_semantic_recall_paraphrase` ~:296)

**Spec:** With live Qdrant + Ollama, an episode recorded under partition P is recalled by a *paraphrased* semantic query with partition P, and is NOT returned for partition Q. (Extends the existing ignored live-services suite; same run conditions.)

- [ ] **Step 1: Add the test**

```rust
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
```

**Check during implementation:** episodes surface from `search_semantic` mapped into `FactCard`s (see the merge section after `qdrant_tier.rs:625`). Confirm the episode→FactCard mapping preserves `session_id`; if it maps into a different field (e.g. `id`), adjust both assertions to that field.

- [ ] **Step 2: Compile-verify (test is ignored by default)**

Run: `~/.cargo/bin/cargo test -p koad-cass --no-run 2>&1 | tail -2`
Expected: compiles clean.

- [ ] **Step 3: Commit**

```bash
git add crates/koad-cass/src/storage/tiered.rs
git commit -m "test(cass): episode partition recall + cross-partition isolation (live suite)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 6: Backfill binary

**Files:**
- Create: `crates/koad-cass/src/bin/backfill_episode_partitions.rs`

**Spec:**
1. Follows the house backfill pattern (`backfill_metadata.rs`): `--db <path>` required, dry-run by default, `--apply` to write.
2. SQLite phase: for every row with `partition = ''`, derive the agent from `session_id` — `SID-{agent}-{uuid}` → regex `^SID-([A-Za-z0-9]+)-`; legacy `{agent}-{date}` → `^([A-Za-z]+)-`; underivable rows are reported and skipped. Partition = `koad_core::utils::partition::partition_key(&agent)` (overridable host/user via `KOAD_HOST`/`USER` env, already supported by the helper).
3. Qdrant phase (`--qdrant <url>`, only with `--apply`): re-upserts every now-partitioned episode through `QdrantTier::record_episode` so the payload gains the field (point ids unchanged; summaries re-embed — 46 rows, negligible).
4. Idempotent: rerunning finds zero empty-partition rows.

- [ ] **Step 1: Write the binary**

```rust
//! Backfill partition keys for episodic_memories rows lacking one, then
//! optionally re-upsert episode points to Qdrant with the new payload field.
//! Usage: backfill_episode_partitions --db <path> [--qdrant <url>] [--apply]
//! Default is dry-run: reports what would change, writes nothing.

use anyhow::Result;
use rusqlite::Connection;

fn arg(args: &[String], key: &str) -> Option<String> {
    args.iter()
        .position(|a| a == key)
        .and_then(|i| args.get(i + 1))
        .cloned()
}

fn derive_agent(session_id: &str) -> Option<String> {
    // SID-{agent}-{uuid}
    if let Some(rest) = session_id.strip_prefix("SID-") {
        if let Some((agent, _)) = rest.split_once('-') {
            if !agent.is_empty() {
                return Some(agent.to_string());
            }
        }
    }
    // legacy: {agent}-{date...}
    if let Some((agent, _)) = session_id.split_once('-') {
        if !agent.is_empty() && agent.chars().all(|c| c.is_ascii_alphanumeric()) {
            return Some(agent.to_string());
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let db = arg(&args, "--db").ok_or_else(|| anyhow::anyhow!("--db <path> is required"))?;
    let qdrant_url = arg(&args, "--qdrant");
    let apply = args.iter().any(|a| a == "--apply");

    let conn = Connection::open(&db)?;
    // Idempotent: ensure the column exists even on an older DB.
    let _ = conn.execute(
        "ALTER TABLE episodic_memories ADD COLUMN partition TEXT NOT NULL DEFAULT ''",
        [],
    );

    let rows: Vec<String> = {
        let mut stmt =
            conn.prepare("SELECT session_id FROM episodic_memories WHERE partition = ''")?;
        let mapped = stmt.query_map([], |r| r.get::<_, String>(0))?;
        mapped.collect::<rusqlite::Result<Vec<_>>>()?
    };

    let mut updated = 0usize;
    let mut skipped: Vec<String> = vec![];
    for sid in &rows {
        match derive_agent(sid) {
            Some(agent) => {
                let partition = koad_core::utils::partition::partition_key(&agent);
                if apply {
                    conn.execute(
                        "UPDATE episodic_memories SET partition = ?1 WHERE session_id = ?2",
                        rusqlite::params![partition, sid],
                    )?;
                }
                println!("{} {} -> {}", if apply { "SET " } else { "PLAN" }, sid, partition);
                updated += 1;
            }
            None => skipped.push(sid.clone()),
        }
    }
    println!(
        "{}: {} rows partitioned, {} underivable{}",
        if apply { "APPLIED" } else { "DRY-RUN" },
        updated,
        skipped.len(),
        if skipped.is_empty() { String::new() } else { format!(": {:?}", skipped) }
    );

    // Qdrant re-upsert phase: push the partitioned rows back so payloads gain the field.
    if let (Some(url), true) = (qdrant_url, apply) {
        use koad_cass::storage::{MemoryTier, QdrantTier};
        use std::sync::Arc;
        let intelligence = Arc::new(koad_intelligence::router::InferenceRouter::new_default()?);
        let qdrant = QdrantTier::new(&url, Some(intelligence)).await?;

        let episodes: Vec<koad_proto::cass::v1::EpisodicMemory> = {
            let mut stmt = conn.prepare(
                "SELECT session_id, project_path, summary, turn_count, timestamp, task_ids, metadata_json, partition
                 FROM episodic_memories WHERE partition != ''",
            )?;
            let mapped = stmt.query_map([], |r| {
                Ok(koad_proto::cass::v1::EpisodicMemory {
                    session_id: r.get(0)?,
                    project_path: r.get(1)?,
                    summary: r.get(2)?,
                    turn_count: r.get::<_, i64>(3)? as u32,
                    timestamp: None, // record_episode restamps; original stays in SQLite
                    task_ids: r
                        .get::<_, String>(5)?
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(String::from)
                        .collect(),
                    metadata: None,
                    partition: r.get(7)?,
                })
            })?;
            mapped.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let n = episodes.len();
        for ep in episodes {
            qdrant.record_episode(ep).await?;
        }
        println!("QDRANT: re-upserted {} episode points to {}", n, url);
    }

    Ok(())
}
```

**Adjust during implementation:** match `QdrantTier::new` signature, `MemoryTier` trait import path, and metadata handling (`metadata_json` → `MemoryMetadata` re-parse if `record_episode` expects it; check how `backfill_embeddings.rs` handles the same concern and copy its idiom). If dropping `timestamp`/`metadata` on the Qdrant re-upsert loses payload data that `make_episode_payload` writes, parse them from columns 4/6 instead of `None` — the SQLite row is the source of truth.

- [ ] **Step 2: Unit-verify derive_agent**

Append to the binary:

```rust
#[cfg(test)]
mod tests {
    use super::derive_agent;

    #[test]
    fn derives_sid_and_legacy_formats() {
        assert_eq!(derive_agent("SID-clyde-01d0aefa").as_deref(), Some("clyde"));
        assert_eq!(derive_agent("SID-hermes-025daf6d").as_deref(), Some("hermes"));
        assert_eq!(derive_agent("clyde-2026-05-09").as_deref(), Some("clyde"));
        assert_eq!(derive_agent("20260606_140451_01834a"), None);
    }
}
```

Run: `~/.cargo/bin/cargo test -p koad-cass --bin backfill_episode_partitions 2>&1 | grep 'test result'`
Expected: `ok. 1 passed`.

- [ ] **Step 3: Dry-run against a copy of the production DB**

```bash
cp ~/.citadel-jupiter/data/db/cass.db /tmp/claude-cass-backfill-test.db
~/.cargo/bin/cargo run -p koad-cass --bin backfill_episode_partitions -- --db /tmp/claude-cass-backfill-test.db
```
Expected: `DRY-RUN: 46 rows partitioned, 0 underivable` (all ids derive: 44 SID-form + 2 legacy).
(Exact DB path: verify with `ls ~/.citadel-jupiter/data/db/` — adjust if the file is named differently.)

- [ ] **Step 4: Commit**

```bash
git add crates/koad-cass/src/bin/backfill_episode_partitions.rs
git commit -m "feat(cass): backfill_episode_partitions binary (dry-run default)

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 7: Live deploy, backfill, smoke test, docs — 🚦 needs Ian's terminal

**Files:**
- Modify: `docs/cass-semantic-recall-verification.md:72-75` (close the residual gap section)
- Modify: `SITREP.md` (mission checkbox)

**Spec:** Production episodes recallable via `SearchSemantic` under `clyde_Jupiter_ideans`. Docs reflect reality.

- [ ] **Step 1: Push + deploy binaries**

```bash
git push origin nightly
./install.sh --update
```

- [ ] **Step 2: Restart services — REAL TERMINAL (sudo needs a TTY; install.sh's `sudo -n` fails silently)**

Ian runs:
```bash
sudo systemctl restart koad-cass.service koad-citadel.service
```
Verify new binary is live: `readlink -f /proc/$(pgrep -f koad-cass | head -1)/exe` + mtime check.

- [ ] **Step 3: Backfill production**

```bash
~/.cargo/bin/cargo run -p koad-cass --release --bin backfill_episode_partitions -- --db ~/.citadel-jupiter/data/db/cass.db
# review dry-run output, then:
~/.cargo/bin/cargo run -p koad-cass --release --bin backfill_episode_partitions -- --db ~/.citadel-jupiter/data/db/cass.db --qdrant http://127.0.0.1:6334 --apply
```
Expected: `APPLIED: 46 rows partitioned, 0 underivable` + `QDRANT: re-upserted 46 episode points`.
Idempotency check: rerun dry-run → `DRY-RUN: 0 rows partitioned`.

- [ ] **Step 4: Live smoke test**

Run the ignored suite against live services:
```bash
~/.cargo/bin/cargo test -p koad-cass test_episode_recall_by_partition -- --ignored 2>&1 | grep 'test result'
```
Expected: `ok. 1 passed`.
Then a production-partition probe via the MCP shim or koad CLI: `SearchSemantic` with partition `clyde_Jupiter_ideans`, a query paraphrasing a known session summary (e.g. "deploying the metadata feature and qdrant phase one") — expect ≥1 episode hit.

- [ ] **Step 5: Docs + commit**

`docs/cass-semantic-recall-verification.md` — replace the residual-gap paragraph (:72-75) with a dated note: episodes carry explicit `partition` (proto field 8), both read paths filter on it, 46 rows backfilled + re-upserted, live test passing.
`SITREP.md` — tick the Episode Partition Keying Design mission, note "implemented + verified live".

```bash
git add docs/cass-semantic-recall-verification.md SITREP.md
git commit -m "docs(cass): episode partition keying implemented, backfilled, verified live

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
git push origin nightly
```

---

## Execution Order & Dependencies

Tasks 1 → 2 → 3 → 4 are strictly sequential (each builds on the previous compile state). Task 5 needs 4; Task 6 needs 3; Task 7 needs everything and **stops for Ian's terminal at Step 2** (sudo restart).

## Rollback

Every task is one commit — `git revert` individually. The SQLite ALTER is additive and harmless if code is reverted (extra column ignored). Qdrant re-upserted points carry a superset payload — old code ignores `partition`. The backfill is idempotent and non-destructive (only fills empty values).
