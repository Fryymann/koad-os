#!/bin/bash
DB_PATH="$HOME/.koad-os/data/notion-sync.db"
mkdir -p "$(dirname "$DB_PATH")"

sqlite3 "$DB_PATH" <<SQL
-- Tracks which Notion databases are configured for sync
CREATE TABLE IF NOT EXISTS sync_sources (
    source_id       TEXT PRIMARY KEY,       -- Notion database UUID
    source_name     TEXT NOT NULL,          -- Human-readable name (e.g., "Research")
    last_sync_at    TEXT,                   -- ISO-8601 timestamp of last successful sync
    page_count      INTEGER DEFAULT 0,
    sync_status     TEXT DEFAULT 'never'    -- never | success | error
);

-- Stores synced page content
CREATE TABLE IF NOT EXISTS pages (
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
-- Note: VIRTUAL TABLE FTS5 requires a specific sqlite3 build, but most modern versions have it.
CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
    title,
    content_md,
    content='pages',
    content_rowid='rowid'
);

-- Sync audit log
CREATE TABLE IF NOT EXISTS sync_log (
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

CREATE INDEX IF NOT EXISTS idx_pages_source ON pages(source_id);
CREATE INDEX IF NOT EXISTS idx_pages_updated ON pages(updated_at);
CREATE INDEX IF NOT EXISTS idx_pages_synced ON pages(synced_at);
CREATE INDEX IF NOT EXISTS idx_pages_deleted ON pages(is_deleted) WHERE is_deleted = 0;
SQL

echo "Notion Sync Datastore initialized at $DB_PATH"
