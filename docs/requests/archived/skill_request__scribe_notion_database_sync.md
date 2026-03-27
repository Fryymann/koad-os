# Skill Request — Scribe: Notion Database Sync
---
## Purpose
Implement a notion-sync skill for Scribe that synchronizes designated Notion databases bidirectionally between the local KoadOS datastore (SQLite) and Notion — with each database having a defined sync direction:
- Notion → KoadOS (pull): Research, Agent Models and Profiles, Feature and Skill Requests. Content authored by Ian and Noti in Notion is pulled into the local SQLite datastore so all KoadOS agents can access it.
- KoadOS → Notion (push): Tyr → Noti Briefings. Tyr authors briefings locally during KoadOS sessions; these are pushed up to Notion so Noti (and Ian) can consume them without needing CLI access.
When Ian (or any agent) says "Please sync the Notion data sources", Scribe should:
1. Connect to the configured Notion databases
1. For pull databases: determine which Notion pages are new or updated since the last sync, pull them into the local SQLite datastore
1. For push databases: determine which local pages are new or updated since the last sync, push them up to Notion via the API
1. Report what was synced in each direction
This bridges the gap between Notion (where Ian and Noti do research, brainstorming, and knowledge management) and the local KoadOS codebase (where Architect and Developer agents operate) — with data flowing in the correct direction for each use case.
---
## Problem Statement
Ian and Noti continuously add research, architecture notes, standards, and knowledge to Notion. KoadOS agents (Tyr, Scribe, future crew) operate locally and cannot natively access Notion content. Today, knowledge transfer requires manual copy-paste or ad-hoc prompts — creating staleness, context gaps, and friction.
The existing .notion/snapshots/ approach in the agents-os repo is a step in the right direction, but it lacks:
- Incremental sync — currently overwrites everything or requires manual selection
- Structured queryability — markdown snapshots aren't indexed or searchable by agents
- Change detection — no way for agents to know what's new or updated
- Self-service — agents can't trigger a sync; it requires manual script execution
- Reverse flow — no mechanism for KoadOS agents to push content back to Notion (e.g., Tyr's briefings for Noti)
A SQLite-backed datastore with directional sync solves all five problems: it's file-based (Dark Mode compatible), queryable, supports change tracking, handles both pull and push directions, and can be triggered by Scribe on command.
---
## Skill Anatomy (Per Skill System Architecture)
Implement the following directory structure:
```plain text
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
## SKILL.md Specification
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
The SKILL.md body should instruct Scribe to follow this workflow:
Phase 1 — Intent Classification
- Parse the user's request to determine the operation:
- If the intent is ambiguous, default to incremental sync of all configured databases (both directions)
Phase 2a — Pull Execution (for direction = "pull" databases)
- Read config/sync-manifest.toml to identify pull-direction databases and field mappings
- Run scripts/sync-databases.sh --direction pull which orchestrates:
- Report results: pages pulled (added, updated, unchanged, deleted)
Phase 2b — Push Execution (for direction = "push" databases)
- Read config/sync-manifest.toml to identify push-direction databases
- Run scripts/sync-databases.sh --direction push which orchestrates:
- Report results: pages pushed (created, updated, unchanged)
- Update references/sync-log.md with human-readable summary for both directions
Phase 3 — Query Execution (for query operations)
- Run scripts/query-datastore.sh with the user's search terms
- Return matching records with title, source database, last synced time, and a content preview
- If no matches, suggest broadening the query or running a fresh sync
Phase 4 — Status Report
- Always end with a brief status line: last sync time, total records, any errors
- If data is stale ( > 24 hours since last sync), suggest running a sync
---
## SQLite Schema Design
The local datastore lives at .koad-os/data/notion-sync.db
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
config/sync-manifest.toml declares which Notion databases to sync:
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
### scripts/sync-databases.sh
Purpose: Main orchestrator. Reads the sync manifest, iterates configured databases, calls fetch + diff scripts, writes the sync log.
Behavior:
- Accept optional --database <name> flag to sync a single source
- Accept --full flag to force full re-sync (ignore last_sync_at)
- Default: incremental sync of all configured databases
- For each database: call fetch-notion-pages.py → pipe to diff-and-upsert.py
- On completion: update sync_sources.last_sync_at, write references/sync-log.md
- Exit code 0 on success, 1 on partial failure, 2 on total failure
Estimated complexity: ~60-100 lines bash.
### scripts/fetch-notion-pages.py
Purpose: Notion API client. Fetches pages from a single database, filtering by last_edited_time when doing incremental sync.
Behavior:
- Accept: --database-id, --since (ISO timestamp), --properties (comma-separated), --include-content
- Use Notion API POST /databases/{id}/query with filter on last_edited_time for incremental
- For each page: retrieve properties + optionally retrieve block children (content)
- Convert page content to markdown using a lightweight block-to-markdown converter
- Output: JSON lines to stdout (one JSON object per page)
- Handle pagination (Notion API returns max 100 per request)
- Handle rate limiting with exponential backoff
Dependencies: requests (Python stdlib-compatible alternative: urllib), json
Estimated complexity: ~150-200 lines Python.
### scripts/diff-and-upsert.py
Purpose: Reads fetched page JSON from stdin, compares against local SQLite, upserts changes.
Behavior:
- Read JSON lines from stdin
- For each page:
- After processing all fetched pages: mark pages not seen in fetch as potentially deleted (soft-delete after 3 consecutive missed syncs to avoid false positives)
- Update FTS index
- Output: summary JSON with counts (added, updated, unchanged, deleted)
Estimated complexity: ~100-150 lines Python.
### scripts/init-datastore.sh
Purpose: Creates the SQLite database and runs schema migrations.
Behavior:
- If .koad-os/data/notion-sync.db doesn't exist → create it with full schema
