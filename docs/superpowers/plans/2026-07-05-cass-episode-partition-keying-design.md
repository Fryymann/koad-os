# CASS Episode Partition Keying — Design

**Status:** DRAFT — awaiting Dood Gate (Condition Green). No implementation until approved.
**Date:** 2026-07-05
**Author:** Clyde (Officer, Claude Code)
**Problem source:** `docs/cass-semantic-recall-verification.md:72` (residual gap)

## Problem Statement

Semantic search returns zero episodes under production partitions. `QdrantTier::search_semantic_scored` filters episode candidates by `sid.contains(partition)` (`crates/koad-cass/src/storage/qdrant_tier.rs:612`), where `partition` is the full partition key (e.g. `clyde_Jupiter_ideans`). Production session ids are `SID-{agent}-{uuid}` (e.g. `SID-clyde-01d0aefa`) — they contain the *agent name* but never the *partition string*, so the filter rejects everything.

**Correction to the verification report:** the report cited session ids like `20260606_140451_01834a` (no agent string at all). Production data disagrees — all 46 rows carry an agent name (44 × `SID-{agent}-{uuid}`, 2 × legacy `{agent}-{date}`). The gap is real, but its cause is the **partition ≠ agent-name mismatch**, not missing agent identity. A second read path, `query_recent_episodes` (`qdrant_tier.rs:527`, called from `hydration.rs:129`), filters by `session_id.contains(agent_name)` with the *bare agent name* — it works today, but only by substring luck: it is rename-unsafe, collides on prefix-sharing agent names (`clyde` matches a hypothetical `clyde2`), and diverges from the fact-card partition canon.

## Current State (code survey, verified 2026-07-05)

**Creation.** `CitadelSessionService::create_lease` generates `SID-{agent_name}-{uuid_short}` (`crates/koad-citadel/src/services/session.rs:270-274`) — agent name is known at creation. On close, `session.rs:396` broadcasts a `session_closed` event carrying `session_id` + `agent_name`; `EndOfWatchPipeline::process_session_close` (`crates/koad-cass/src/services/eow.rs:61-88`) creates the `EpisodicMemory`. A second creation path exists in `crates/koad-cli/src/handlers/system.rs:1566` (reads `session_history`, creates episode).

**SQLite schema** (`crates/koad-cass/src/storage/sqlite_tier.rs:34-41`): `episodic_memories(session_id TEXT PK, project_path, summary, turn_count, timestamp, task_ids, metadata_json)`. **No agent/partition column.** (`metadata_json` was added later via `ALTER TABLE` at lines 46-48 — precedent for additive migration.)

**Qdrant payload** (`make_episode_payload`, `qdrant_tier.rs:314-365`, upsert at 493-498): `session_id`, `project_path`, `summary`, `turn_count`, `task_ids`, `timestamp`, `metadata_json`. **No partition field.**

**Read paths filtering episodes by identity:**
| Site | Filter | State |
| :--- | :--- | :--- |
| `qdrant_tier.rs:612` (search_semantic_scored) | `sid.contains(partition)` | **BROKEN** — partition never in sid |
| `qdrant_tier.rs:527` (query_recent_episodes) | `sid.contains(agent_name)` | Works by substring luck |
| `sqlite_tier.rs:342-347` | `_agent_name` ignored | No filtering |
| `redis_tier.rs:171-177` | — | Returns empty |
| `mock.rs:114-124` | — | Returns all |

**Facts, for contrast**, are already fixed: partition canon is the **domain prefix**, filtered via `domain_matches_partition` (`qdrant_tier.rs:584`), matching L2 SQLite. Episodes have no equivalent partition-carrying field.

**Production sizing:** 46 rows in `episodic_memories`. Backfill is trivial.

## Options

### Option A — Dedicated partition column + payload field (RECOMMENDED)

Add explicit `partition` to episodes in all three places: struct, SQLite, Qdrant payload.

1. **Struct:** add `partition: String` to `EpisodicMemory`.
2. **SQLite:** `ALTER TABLE episodic_memories ADD COLUMN partition TEXT NOT NULL DEFAULT ''` — same additive idiom as `metadata_json` (sqlite_tier.rs:46-48). Empty string = legacy/unbackfilled.
3. **Qdrant:** add `partition` to `make_episode_payload`; filter server-side with a Qdrant keyword `Filter::must(partition = X)` on the episode search (replaces local `contains` at :612), or locally by exact payload match if the client-filter idiom is preferred for consistency with facts.
4. **Write path:** extend the `session_closed` event payload with `partition` (session lease knows the agent; partition template is `{agent}_{HOSTNAME}_{USER}`, canon per Rook bridge `AGENT_PARTITION`). EOW pipeline writes it through. The koad-cli creation path (`system.rs:1566`) derives it the same way. Fallback when absent: derive from `agent_name` + local hostname/user.
5. **Read paths:** `search_semantic_scored` filters `payload.partition == partition`, with legacy fallback `partition.is_empty() && sid.contains(agent-name-prefix-of-partition)` during transition. `query_recent_episodes` switches from bare `agent_name` contains to the same partition equality — unifying both paths on one canon.
6. **Proto:** add optional `partition` field to the episode message (additive, backward-compatible; old clients ignore it).

*Pros:* matches the fact-card fix's canon (explicit field = partition), enables server-side Qdrant filtering, kills substring fragility in both read paths, mechanical backfill.
*Cons:* touches struct + 2 storage tiers + proto + 2 write paths; needs a backfill pass.

### Option B — Encode partition into session_id

Change id format to `SID-{partition}-{uuid}` at creation (`session.rs:270-274`); `contains(partition)` then works unchanged.

*Pros:* no schema change; one-line generator edit for new episodes.
*Cons:* rewrites id semantics — session ids are consumed by the session-lease system, CLI `session_history`, Redis key refs (`redis_tier.rs:28` uses `kind:id`), and are PKs in SQLite and point ids in Qdrant. Existing 46 rows are unfixable without PK/point-id rewrite (worse than a column backfill). Substring matching stays fragile. Partition changes (host rename, user change) would orphan history. **Rejected.**

### Option C — Join through fact cards

Derive episode ownership at query time: episode belongs to partition P if any fact card with matching `session_id` lives in P's domain.

*Pros:* zero schema change.
*Cons:* extra query per search; wrong for episodes with no facts (EOW episodes routinely have none); keeps ownership implicit — the exact property that caused this gap. **Rejected.**

## Backfill Strategy (Option A)

New binary `backfill_episode_partitions`, following the `backfill_metadata` / `backfill_embeddings` pattern (SQLite-direct, `--dry-run` default):

1. For each row with `partition = ''`: parse agent from `session_id` — regex `^SID-([a-z0-9]+)-` covers 44 rows; the 2 legacy rows (`clyde-2026-05-09`, `clyde-2026-06-30-cass-deploy-f1`) parse as `^([a-z]+)-`.
2. Compute `partition = {agent}_{HOSTNAME}_{USER}` from the local instance (all 46 rows are this instance's data; agents observed: clyde, hermes, tyr, rook).
3. `UPDATE episodic_memories SET partition = ?`; re-upsert the 46 Qdrant episode points with the new payload field (existing point ids unchanged).
4. Verify: count of `partition = ''` rows → 0; live `SearchSemantic` smoke test returns episodes under `clyde_Jupiter_ideans`.

Note: rows for the decommissioned agent `rook` backfill to `rook_{HOSTNAME}_{USER}` — inert but preserved.

## Recommendation

**Option A.** It is the same shape as the fact-card fix Dood already approved (explicit partition-carrying field + filter on it), unifies the two divergent read paths, and the 46-row backfill is mechanical. Options B and C both preserve the implicit-substring coupling that caused the gap.

**Requested decision:** Condition Green on Option A → separate TDD implementation plan (est. scope: `EpisodicMemory` struct, `sqlite_tier`, `qdrant_tier`, `eow.rs`, `session.rs` event, `system.rs` CLI path, proto, backfill binary, integration test extending the paraphrase-recall suite).
