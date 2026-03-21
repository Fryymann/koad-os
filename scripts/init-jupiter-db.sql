-- Phase 1A: SQLite Hardening & L4 Procedural Memory
PRAGMA journal_mode=WAL;

-- L4 Procedural Memory (Patterns & Skills)
CREATE TABLE IF NOT EXISTS procedural_memory (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  agent_id TEXT NOT NULL,
  skill_name TEXT NOT NULL,
  pattern TEXT NOT NULL,
  success_count INTEGER DEFAULT 0,
  failure_count INTEGER DEFAULT 0,
  last_used DATETIME,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Optimization Indexes
CREATE INDEX IF NOT EXISTS idx_agent_time ON episodic(agent_id, created_at);
CREATE INDEX IF NOT EXISTS idx_skill_pattern ON procedural_memory(skill_name, pattern);

-- Evolution: Importance Score for Phase 1C (Handles error if already exists)
-- Note: sqlite doesn't support 'IF NOT EXISTS' for columns directly in ALTER
-- This is handled by the application layer or a script check
