# Skill Request — Scribe: Notion Database Sync

<aside>
📌

**Type:** Skill Implementation Request

**Requesting Agent:** Noti (on behalf of Ian)

**Implementing Agent:** Tyr

**Target Agent:** Scribe (gemini-2.5-flash-lite)

**Date:** 2026-03-15

**Status:** PENDING — Awaiting Tyr implementation

**Prerequisite:** Notion API integration token with read access to target databases must be configured in `.koad-os/config/secrets/` (or environment variable)

**Architecture Reference:** [KoadOS Feature Request — Skill System Architecture (Pattern Extraction from Claude Code)](https://www.notion.so/KoadOS-Feature-Request-Skill-System-Architecture-Pattern-Extraction-from-Claude-Code-4c0e75847fe546d3b2050c6197a5ed11?pvs=21)

**Related:** [KoadOS Feature Request: Notion Database index](https://www.notion.so/KoadOS-Feature-Request-Notion-Database-index-324fe8ecae8f80048549e5280e23499a?pvs=21) · [Skill Request — Scribe: Support Knowledge Base Q&A](https://www.notion.so/Skill-Request-Scribe-Support-Knowledge-Base-Q-A-5e23bbb4ecea4d268a353b71713304b3?pvs=21)

</aside>

---

## Purpose

Implement a **notion-sync** skill for Scribe that synchronizes designated Notion databases **bidirectionally** between the local KoadOS datastore (SQLite) and Notion — with each database having a defined sync direction:

- **Notion → KoadOS (pull):** Research, Agent Models and Profiles, Feature and Skill Requests. Content authored by Ian and Noti in Notion is pulled into the local SQLite datastore so all KoadOS agents can access it.
- **KoadOS → Notion (push):** Tyr → Noti Briefings. Tyr authors briefings locally during KoadOS sessions; these are pushed up to Notion so Noti (and Ian) can consume them without needing CLI access.

When Ian (or any agent) says **"Please sync the Notion data sources"**, Scribe should:

1. Connect to the configured Notion databases
2. For **pull** databases: determine which Notion pages are new or updated since the last sync, pull them into the local SQLite datastore
3. For **push** databases: determine which local pages are new or updated since the last sync, push them up to Notion via the API
4. Report what was synced in each direction

This bridges the gap between Notion (where Ian and Noti do research, brainstorming, and knowledge management) and the local KoadOS codebase (where Architect and Developer agents operate) — with data flowing in the correct direction for each use case.

---

## Problem Statement

Ian and Noti continuously add research, architecture notes, standards, and knowledge to Notion. KoadOS agents (Tyr, Scribe, future crew) operate locally and cannot natively access Notion content. Today, knowledge transfer requires manual copy-paste or ad-hoc prompts — creating staleness, context gaps, and friction.

The existing `.notion/snapshots/` approach in the agents-os repo is a step in the right direction, but it lacks:

- **Incremental sync** — currently overwrites everything or requires manual selection
- **Structured queryability** — markdown snapshots aren't indexed or searchable by agents
- **Change detection** — no way for agents to know what's new or updated
- **Self-service** — agents can't trigger a sync; it requires manual script execution
- **Reverse flow** — no mechanism for KoadOS agents to push content *back* to Notion (e.g., Tyr's briefings for Noti)

A SQLite-backed datastore with directional sync solves all five problems: it's file-based (Dark Mode compatible), queryable, supports change tracking, handles both pull and push directions, and can be triggered by Scribe on command.

---

## Skill Anatomy (Per Skill System Architecture)

Implement the following directory structure:

```
.koad-os/skills/scribe/notion-sync/
├── SKILL.md                    # Skill definition + instructions
├── scripts/
│   ├── sync-databases.sh           # Main sync orchestrator (handles both pull and push)
│   ├── fetch-notion-pages.py       # Notion API client — fetches pages from configured DBs (pull)
│   ├── push-to-notion.py           # Notion API client — pushes local pages to Notion (push)
│   ├── diff-and-upsert.py          # Compares fetched pages to local store, upserts changes
│   ├── init-datastore.sh           # Creates/migrates the SQLite schema
│   └── query-datastore.sh          # CLI query interface for agents
├── config/
│   └── sync-manifest.toml          # Declares which Notion databases to sync + direction + field mappings
├── references/
│   └── sync-log.md                 # Human-readable log of last sync (auto-generated)
├── agents/                         # (empty for v1 — no sub-agent delegation)
└── _eval/
    ├── test-prompts.md             # Sample sync commands + query scenarios
    └── grading-schema.md           # Expected behavior criteria
```

---

## [SKILL.md](http://SKILL.md) Specification

### Frontmatter

```yaml
name: notion-sync
description: >
  Synchronize designated Notion databases to the local KoadOS SQLite datastore.
  Provides incremental sync with change detection, so KoadOS agents always have
  access to the latest research, standards, and knowledge authored in Notion.
  Trigger when a user asks to sync Notion, refresh local data from Notion,
  check what's changed in Notion, or query synced Notion content.
trigger_patterns:
  - "Sync the Notion *"
  - "Please sync the Notion data sources"
  - "Refresh Notion data"
  - "Pull latest from Notion"
  - "What's new in Notion?"
  - "Query Notion for *"
  - "What does the * page say?"
requires:
  - python3                         # For Notion API client
  - sqlite3                         # CLI for datastore queries
  - NOTION_API_TOKEN                # Environment variable or secrets config
tier: crew
context_cost: low                   # Skill instructions are lightweight; data stays in SQLite
author: tyr
version: 1.0.0
```

### Instruction Body

The [SKILL.md](http://SKILL.md) body should instruct Scribe to follow this workflow:

**Phase 1 — Intent Classification**

- Parse the user's request to determine the operation:
    - `sync` — full or incremental sync of all configured databases (both directions)
    - `sync <database>` — sync a specific database by name
    - `pull` — sync only Notion → KoadOS databases
    - `push` — sync only KoadOS → Notion databases
    - `query <topic>` — search the local datastore for synced content
    - `status` — report last sync time, record counts, staleness, per direction
- If the intent is ambiguous, default to incremental sync of all configured databases (both directions)

**Phase 2a — Pull Execution** (for `direction = "pull"` databases)

- Read `config/sync-manifest.toml` to identify pull-direction databases and field mappings
- Run `scripts/sync-databases.sh --direction pull` which orchestrates:
    1. `fetch-notion-pages.py` — calls Notion API, retrieves pages modified since last sync timestamp
    2. `diff-and-upsert.py` — compares fetched pages against local SQLite records by page ID, upserts new/changed pages, marks deleted pages
- Report results: pages pulled (added, updated, unchanged, deleted)

**Phase 2b — Push Execution** (for `direction = "push"` databases)

- Read `config/sync-manifest.toml` to identify push-direction databases
- Run `scripts/sync-databases.sh --direction push` which orchestrates:
    1. Query local SQLite for pages in push-direction sources where `updated_at > last_sync_at`
    2. `push-to-notion.py` — calls Notion API to create or update pages in the target Notion database
- Report results: pages pushed (created, updated, unchanged)
- Update `references/sync-log.md` with human-readable summary for both directions

**Phase 3 — Query Execution** (for query operations)

- Run `scripts/query-datastore.sh` with the user's search terms
- Return matching records with title, source database, last synced time, and a content preview
- If no matches, suggest broadening the query or running a fresh sync

**Phase 4 — Status Report**

- Always end with a brief status line: last sync time, total records, any errors
- If data is stale ( > 24 hours since last sync), suggest running a sync

---

## SQLite Schema Design

The local datastore lives at `.koad-os/data/notion-sync.db`

### Core Tables

```sql
-- Tracks which Notion databases are configured for sync
CREATE TABLE sync_sources (
    source_id       TEXT PRIMARY KEY,       -- Notion database UUID
    source_name     TEXT NOT NULL,          -- Human-readable name (e.g., "Research")
    last_sync_at    TEXT,                   -- ISO-8601 timestamp of last successful sync
    page_count      INTEGER DEFAULT 0,
    sync_status     TEXT DEFAULT 'never'    -- never | success | error
);

-- Stores synced page content
CREATE TABLE pages (
    page_id         TEXT PRIMARY KEY,       -- Notion page UUID
    source_id       TEXT NOT NULL,          -- FK to sync_sources
    title           TEXT NOT NULL,
    content_md      TEXT,                   -- Page content as markdown
    properties_json TEXT,                   -- Page properties as JSON
    notion_url      TEXT,                   -- Original Notion URL for back-reference
    created_at      TEXT NOT NULL,          -- Notion created time
    updated_at      TEXT NOT NULL,          -- Notion last edited time
    synced_at       TEXT NOT NULL,          -- When this record was last synced
    is_deleted      INTEGER DEFAULT 0,      -- Soft-delete flag
    FOREIGN KEY (source_id) REFERENCES sync_sources(source_id)
);

-- Full-text search index for fast agent queries
CREATE VIRTUAL TABLE pages_fts USING fts5(
    title,
    content_md,
    content='pages',
    content_rowid='rowid'
);

-- Sync audit log
CREATE TABLE sync_log (
    sync_id         INTEGER PRIMARY KEY AUTOINCREMENT,
    started_at      TEXT NOT NULL,
    completed_at    TEXT,
    source_id       TEXT,                   -- NULL = all sources
    pages_added     INTEGER DEFAULT 0,
    pages_updated   INTEGER DEFAULT 0,
    pages_deleted   INTEGER DEFAULT 0,
    pages_unchanged INTEGER DEFAULT 0,
    status          TEXT DEFAULT 'running', -- running | success | error
    error_message   TEXT
);
```

### Indexes

```sql
CREATE INDEX idx_pages_source ON pages(source_id);
CREATE INDEX idx_pages_updated ON pages(updated_at);
CREATE INDEX idx_pages_synced ON pages(synced_at);
CREATE INDEX idx_pages_deleted ON pages(is_deleted) WHERE is_deleted = 0;
```

---

## Sync Manifest Configuration

`config/sync-manifest.toml` declares which Notion databases to sync:

```toml
[global]
datastore_path = ".koad-os/data/notion-sync.db"
default_sync_mode = "incremental"     # incremental | full
default_direction = "pull"            # pull (Notion→KoadOS) | push (KoadOS→Notion)
max_pages_per_sync = 500              # Safety cap
content_format = "markdown"           # markdown | plain_text

# ─────────────────────────────────────────────
# 1. Research
# ─────────────────────────────────────────────
[[databases]]
name = "Research"
notion_database_id = "collection://02412856-6875-43f5-ab56-c12b1d569287"
direction = "pull"                    # Notion → KoadOS (default)
sync_properties = ["Title", "Domain", "Created"]
include_content = true
priority = "high"

# ─────────────────────────────────────────────
# 2. Agent Models and Profiles (5 sub-databases)
# ─────────────────────────────────────────────
[[databases]]
name = "Gemini API Models"
notion_database_id = "collection://f0f7002a-5d04-4f08-ac7d-ce7c1ab2c0bd"
direction = "pull"
parent_group = "Agent Models and Profiles"
sync_properties = ["Model Name", "Model ID", "Family", "Tier", "Status", "Context Window (M tokens)", "Input $/1M tokens", "Output $/1M tokens", "Best For", "Thinking"]
include_content = true
priority = "high"

[[databases]]
name = "Codex Models"
notion_database_id = "collection://b6ab2bcb-0545-4f9b-b380-53e1f0f8a18c"
direction = "pull"
parent_group = "Agent Models and Profiles"
sync_properties = ["Model Name", "Model ID", "Family", "Tier", "Status", "Context Window (K tokens)", "Input $/1M tokens", "Output $/1M tokens", "Best For", "Thinking"]
include_content = true
priority = "high"

[[databases]]
name = "Claude API Models"
notion_database_id = "collection://58db6794-3e68-44d6-89c9-8e8a3454a6d7"
direction = "pull"
parent_group = "Agent Models and Profiles"
sync_properties = ["Model Name", "Model ID", "Family", "Tier", "Status", "Context Window (K tokens)", "Input $/1M tokens", "Output $/1M tokens", "Best For", "Thinking"]
include_content = true
priority = "high"

[[databases]]
name = "Ollama Models"
notion_database_id = "collection://fd367fa2-9192-4157-807d-6c1a68b6d280"
direction = "pull"
parent_group = "Agent Models and Profiles"
sync_properties = ["Model Name", "Model ID", "Family", "Tier", "Status", "Parameters", "Context Window (K tokens)", "RAM Required", "Best For", "Thinking", "Desktop (5070 Ti)", "Laptop (Io)"]
include_content = true
priority = "high"

[[databases]]
name = "Agent Profile Drafts"
notion_database_id = "collection://323fe8ec-ae8f-80b1-8a33-000bb6646162"
direction = "pull"
parent_group = "Agent Models and Profiles"
sync_properties = ["Name", "Created time", "Last edited time"]
include_content = true
priority = "high"

# ─────────────────────────────────────────────
# 3. Tyr → Noti Briefings  (KoadOS → Notion)
#    Tyr authors briefings locally; Scribe pushes
#    them to Notion so Noti and Ian can read them.
# ─────────────────────────────────────────────
[[databases]]
name = "Tyr → Noti Briefings"
notion_database_id = "collection://20823049-b6a1-421e-ae11-a694775277fe"
direction = "push"                    # KoadOS → Notion (reverse of default)
sync_properties = ["Title", "Status", "Priority", "Created"]
include_content = true
priority = "high"

# ─────────────────────────────────────────────
# 4. Feature and Skill Requests
# ─────────────────────────────────────────────
[[databases]]
name = "Feature and Skill Requests"
notion_database_id = "collection://324fe8ec-ae8f-808c-8d15-000b614223a3"
direction = "pull"                    # Notion → KoadOS
sync_properties = ["Title", "Created time", "Last edited time"]
include_content = true
priority = "high"

# Add more databases as needed — Scribe reads this manifest at sync time
```

---

## Scripts Specification

### `scripts/sync-databases.sh`

**Purpose:** Main orchestrator. Reads the sync manifest, iterates configured databases, calls fetch + diff scripts, writes the sync log.

**Behavior:**

- Accept optional `--database <name>` flag to sync a single source
- Accept `--full` flag to force full re-sync (ignore last_sync_at)
- Default: incremental sync of all configured databases
- For each database: call `fetch-notion-pages.py` → pipe to `diff-and-upsert.py`
- On completion: update `sync_sources.last_sync_at`, write `references/sync-log.md`
- Exit code 0 on success, 1 on partial failure, 2 on total failure

**Estimated complexity:** ~60-100 lines bash.

### `scripts/fetch-notion-pages.py`

**Purpose:** Notion API client. Fetches pages from a single database, filtering by last_edited_time when doing incremental sync.

**Behavior:**

- Accept: `--database-id`, `--since` (ISO timestamp), `--properties` (comma-separated), `--include-content`
- Use Notion API `POST /databases/{id}/query` with `filter` on `last_edited_time` for incremental
- For each page: retrieve properties + optionally retrieve block children (content)
- Convert page content to markdown using a lightweight block-to-markdown converter
- Output: JSON lines to stdout (one JSON object per page)
- Handle pagination (Notion API returns max 100 per request)
- Handle rate limiting with exponential backoff

**Dependencies:** `requests` (Python stdlib-compatible alternative: `urllib`), `json`

**Estimated complexity:** ~150-200 lines Python.

### `scripts/diff-and-upsert.py`

**Purpose:** Reads fetched page JSON from stdin, compares against local SQLite, upserts changes.

**Behavior:**

- Read JSON lines from stdin
- For each page:
    - If page_id not in local DB → INSERT (new page)
    - If page_id exists but updated_at differs → UPDATE (changed page)
    - If page_id exists and updated_at matches → SKIP (unchanged)
- After processing all fetched pages: mark pages not seen in fetch as potentially deleted (soft-delete after 3 consecutive missed syncs to avoid false positives)
- Update FTS index
- Output: summary JSON with counts (added, updated, unchanged, deleted)

**Estimated complexity:** ~100-150 lines Python.

### `scripts/init-datastore.sh`

**Purpose:** Creates the SQLite database and runs schema migrations.

**Behavior:**

- If `.koad-os/data/notion-sync.db` doesn't exist → create it with full schema
- If it exists → check schema version, apply migrations if needed
- Idempotent — safe to re-run
- Create the `data/` directory if it doesn't exist

**Estimated complexity:** ~40-60 lines bash + embedded SQL.

### `scripts/query-datastore.sh`

**Purpose:** CLI query interface for agents to search synced content.

**Behavior:**

- Accept: `--search <terms>` (FTS query), `--source <name>`, `--limit <n>`, `--format <brief|full|json>`
- `--search`: uses FTS5 `MATCH` against `pages_fts`
- `--source`: filters by source database name
- `--format brief`: title + source + last synced (default)
- `--format full`: title + source + full markdown content
- `--format json`: raw JSON output for programmatic consumption
- `--status`: show sync_sources summary (no search)

**Estimated complexity:** ~50-80 lines bash + SQL.

---

## Pattern Implementation Map

| **Pattern** | **Implementation in this Skill** |
| --- | --- |
| **P1: Progressive Disclosure** | Frontmatter (~120 tokens) → [SKILL.md](http://SKILL.md) body (~500 tokens) → sync manifest (loaded on trigger) → actual data stays in SQLite, never bulk-loaded into context. |
| **P2: Phased Convergence** | Four-phase workflow (Intent → Sync/Query → Report → Status). Sync and query are separate code paths selected in Phase 1. |
| **P3: Description as Router** | Trigger patterns cover sync commands, refresh requests, Notion queries, and staleness checks. Broad enough to catch natural phrasing. |
| **P4: Graceful Degradation** | If Notion API is unreachable → Scribe reports the error and offers to query the last-known local data. If SQLite doesn't exist → runs `init-datastore.sh` automatically. If a specific database fails → continues with remaining databases and reports partial success. |
| **P6: Eval-Driven Development** | `_eval/test-prompts.md` covers sync triggers, query scenarios, error handling, and status checks. `_eval/grading-schema.md` verifies correctness of sync behavior and query results. |
| **P7: Anti-Overfitting** | Instructions explain the *why* (bridge Notion knowledge to local agents) and trust Scribe to handle edge cases. No rigid output templates. |

**Patterns deferred to v2:**

- **P5: Subagent Delegation** — v2 could have Scribe delegate content extraction to a sub-agent for complex page structures.
- **P8: Bundled Scripts** — v2 could bundle a richer Notion-to-markdown converter with block type support for databases, callouts, toggles, etc.

---

## Integration with Existing Systems

### Relationship to `.notion/snapshots/`

The existing `.notion/snapshots/pages/` directory in agents-os contains manual markdown exports. This skill **replaces** that approach with a structured, automated, queryable system:

- Existing snapshots can be imported into SQLite as a one-time migration
- Going forward, `notion-sync` is the canonical path for Notion → local data flow
- The `sync-manifest.toml` is the single source of truth for what gets synced

### How Other Agents Consume Synced Data

- **Tyr:** Can query the datastore before making architectural decisions to check if Notion has relevant research or standards updates
- **Scribe (support-kb skill):** Can cross-reference KB articles with the latest Notion research for richer answers
- **Noti (via Ian):** Benefits from knowing that local agents have the latest Notion knowledge — reduces the need for manual copy-paste briefings
- **Future agents:** Any agent with SQLite read access can query `notion-sync.db` directly

**Query pattern for agents:**

```bash
# Search for a topic
.koad-os/skills/scribe/notion-sync/scripts/query-datastore.sh --search "boot hydration" --format brief

# Get full content of a specific research page
.koad-os/skills/scribe/notion-sync/scripts/query-datastore.sh --search "Rust best practices" --format full

# Check sync freshness
.koad-os/skills/scribe/notion-sync/scripts/query-datastore.sh --status
```

### Boot-Time Integration

- Skill metadata loads at boot via `koad boot --agent scribe`
- On boot, Scribe can optionally check sync staleness and suggest a refresh (configurable in manifest)
- The datastore itself is always available — no boot-time loading of content

### Dark Mode Compatibility

- SQLite is file-based — works in Dark Mode, Cold Boot, or Full Citadel
- If Notion API is unreachable (offline / no network), the local datastore remains queryable with its last-known state
- Scribe reports staleness but never blocks on a failed sync

### CASS Integration Path (v2+)

- When CASS comes online, synced page content can be promoted to CASS memory tiers:
    - **Redis (hot):** Frequently queried pages, recently synced content
    - **SQLite (warm):** Full datastore (already built by this skill)
    - **Qdrant (deep):** Embedded page vectors for semantic search across all synced Notion content
- The `notion-sync.db` becomes the warm tier automatically — zero migration needed

### Refresh Hooks

- `koad-refresh` should trigger `sync-databases.sh` when connectivity is available
- Consider a cron-style hook (via `koad schedule`) for automatic daily syncs
- Manual trigger is always available: user says "sync Notion" → Scribe executes

---

## Eval Specification

### `_eval/test-prompts.md`

Contain at least 12 test scenarios:

**Sync Operations (5 scenarios):**

- "Please sync the Notion data sources" → triggers incremental sync of all configured databases
- "Sync the Research database" → targets a single database by name
- "Do a full Notion refresh" → triggers full re-sync (ignore timestamps)
- "Sync Notion" (while API is unreachable) → graceful error + offers local query
- "Sync Notion" (first time, no datastore exists) → auto-initializes, then syncs

**Query Operations (4 scenarios):**

- "What does the Rust best practices page say?" → FTS search, returns matching content
- "Show me all synced Research pages" → source filter query
- "Query Notion for CASS architecture" → FTS search with topic
- "What's in the local Notion store about agents?" → broad FTS query

**Status Operations (3 scenarios):**

- "When was Notion last synced?" → returns last sync timestamps per source
- "Is the Notion data stale?" → checks staleness, recommends sync if needed
- "How many pages are synced?" → returns record counts per source

### `_eval/grading-schema.md`

| **Criterion** | **Weight** | **Pass Threshold** |
| --- | --- | --- |
| **Correct Operation** — Scribe selects the right action (sync/query/status) for the prompt | 30% | Correct operation selected in all cases. |
| **Data Integrity** — Sync produces correct local records matching Notion source | 25% | No data loss, no duplicate records, properties match source. |
| **Incremental Correctness** — Only changed pages are synced on incremental runs | 15% | Unchanged pages show as "unchanged" in report. No unnecessary writes. |
| **Error Handling** — Graceful behavior when API fails, DB missing, or manifest misconfigured | 15% | No crashes. Clear error message. Falls back to local data when possible. |
| **Reporting** — Clear, concise output showing what happened | 10% | Reports include counts (added/updated/unchanged) and timing. |
| **Queryability** — FTS queries return relevant results from synced content | 5% | Search for a known-synced topic returns the correct page(s). |

**Pass/Fail:** A test scenario passes if it scores ≥ 80% weighted. The skill passes eval if ≥ 80% of scenarios pass.

---

## Deliverables for Tyr

1. **Implement the skill directory structure** at `.koad-os/skills/scribe/notion-sync/`
2. **Write [SKILL.md](http://SKILL.md)** with frontmatter + instruction body per the spec above
3. **Design and implement the SQLite schema** via `init-datastore.sh`
4. **Implement `fetch-notion-pages.py`** — Notion API client with incremental filtering and markdown conversion
5. **Implement `diff-and-upsert.py`** — change detection and local upsert logic
6. **Implement `sync-databases.sh`** — orchestrator that ties fetch + diff together
7. **Implement `query-datastore.sh`** — agent-facing query CLI
8. **Create `config/sync-manifest.toml`** — seed with the Research, Admiral Orders, and Brainstorms databases (get UUIDs from Ian or Noti)
9. **Write the eval files** — `test-prompts.md` with 12+ scenarios, `grading-schema.md` with criteria
10. **Run eval** — execute sync against real Notion databases, verify local data, run query tests, grade against schema
11. **KSRP self-review** on all deliverables before presenting to Ian

---

## Execution Rules

1. **This is an implementation task.** Tyr builds the skill. Follow the Canon: View & Assess → Research → Plan → Approval Gate → Implement → KSRP → PSRP → Results Report.
2. **Prerequisite gate:** A Notion API integration token must be available before sync can be tested. The skill structure, schema, and scripts can be built before the token is configured.
3. **Python is acceptable for v1.** The Notion API client and diff logic benefit from Python's `requests` / `json` / `sqlite3` stdlib. Bash orchestrates. Rust port is a v2 consideration.
4. **Skill Writing Guide compliance:** Instructions in [SKILL.md](http://SKILL.md) should follow Pattern 7 (Anti-Overfitting) — explain *why* before *what*, trust Scribe to handle edge cases.
5. **v1 scope only.** No CASS integration, no Qdrant embeddings, no webhook-based real-time sync. Keep it pull-based and script-powered. v2 upgrades are noted but not built.
6. **Security:** The Notion API token must never be logged, committed, or exposed in output. Read from environment variable or `.koad-os/config/secrets/` only.
7. **Test with real data.** The eval is not optional. Scribe must demonstrably sync real Notion databases and serve correct query results before delivery.

---

## Success Criteria

- [ ]  Ian can say "Sync the Notion data sources" and Scribe executes an incremental sync
- [ ]  Only new/changed pages are pulled on incremental sync — unchanged pages are skipped
- [ ]  Any KoadOS agent can query the local datastore for synced Notion content
- [ ]  FTS search returns relevant results for topic-based queries
- [ ]  Sync gracefully handles API errors, missing databases, and first-time initialization
- [ ]  Sync log provides clear human-readable record of what happened
- [ ]  The datastore is Dark Mode compatible — queryable even when Notion is unreachable
- [ ]  Eval passes at ≥ 80% on the test scenario suite
- [ ]  No Notion API tokens are exposed in logs or output

---

<aside>
🌉

**This skill bridges Notion and KoadOS.** It turns Noti's research and Ian's documentation into queryable local knowledge that every agent can access. Build it as infrastructure — reliable, boring, and always available.

</aside>