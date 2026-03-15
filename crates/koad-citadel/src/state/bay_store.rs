//! Bay Store
//!
//! Manages the persistent storage for Personal Bays using per-agent SQLite databases.
//! This module handles directory provisioning, health logging, and workspace mapping.

use anyhow::{Context, Result};
use rusqlite::params;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, warn};

/// Manages per-agent Personal Bay SQLite databases.
///
/// The `BayStore` maintains a collection of open SQLite connections to ensure
/// high-performance logging and querying of agent-specific state.
pub struct BayStore {
    /// Base path for bays: e.g., `~/.koad-os/bays/`
    base_path: PathBuf,
    /// Open connections keyed by agent name.
    connections: Arc<Mutex<HashMap<String, rusqlite::Connection>>>,
}

impl BayStore {
    /// Creates a new `BayStore` instance.
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Provision a Personal Bay for an agent: create directory + SQLite schema.
    ///
    /// # Errors
    /// Returns an error if directory creation or SQLite initialization fails.
    pub async fn provision(&self, agent_name: &str) -> Result<()> {
        let bay_dir = self.base_path.join(agent_name);
        std::fs::create_dir_all(&bay_dir)
            .with_context(|| format!("Failed to create bay directory for {}", agent_name))?;

        let db_path = bay_dir.join("state.db");
        let conn = rusqlite::Connection::open(&db_path)
            .with_context(|| format!("Failed to open bay DB for {}", agent_name))?;

        // Enable WAL mode for concurrency
        let _: String = conn
            .query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))
            .context("Failed to set WAL mode on bay DB")?;

        // Health log: tracks docking state transitions
        conn.execute(
            "CREATE TABLE IF NOT EXISTS health_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                state TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                message TEXT
            )",
            [],
        )
        .context("Failed to create health_log table")?;

        // FS Map: tracks worktree paths assigned to this agent
        conn.execute(
            "CREATE TABLE IF NOT EXISTS fs_map (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_id TEXT NOT NULL UNIQUE,
                worktree_path TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                active INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )
        .context("Failed to create fs_map table")?;

        // Session history: completed sessions
        conn.execute(
            "CREATE TABLE IF NOT EXISTS session_history (
                session_id TEXT PRIMARY KEY,
                agent_name TEXT NOT NULL,
                start_time INTEGER NOT NULL,
                end_time INTEGER,
                eow_path TEXT,
                final_state TEXT NOT NULL
            )",
            [],
        )
        .context("Failed to create session_history table")?;

        // Agent Metadata: Tracks XP, Level, and other permanent stats
        conn.execute(
            "CREATE TABLE IF NOT EXISTS agent_metadata (
                agent_name TEXT PRIMARY KEY,
                xp INTEGER NOT NULL DEFAULT 0,
                level INTEGER NOT NULL DEFAULT 1
            )",
            [],
        )
        .context("Failed to create agent_metadata table")?;

        // Initial metadata seed if not exists
        conn.execute(
            "INSERT OR IGNORE INTO agent_metadata (agent_name, xp, level) VALUES (?1, 0, 1)",
            params![agent_name],
        )?;

        let mut conns = self.connections.lock().await;
        conns.insert(agent_name.to_string(), conn);

        info!("BayStore: Provisioned bay for '{}'", agent_name);
        Ok(())
    }

    /// Auto-provision bays for all agents found in the identities directory.
    ///
    /// # Errors
    /// Returns an error if the identities directory cannot be read.
    pub async fn auto_provision_all(&self, identities_dir: &Path) -> Result<()> {
        if !identities_dir.exists() {
            warn!(
                "BayStore: Identities directory not found: {:?}",
                identities_dir
            );
            return Ok(());
        }

        for entry in std::fs::read_dir(identities_dir).context("Failed to read identities dir")? {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "toml") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let bay_dir = self.base_path.join(stem);
                    if !bay_dir.exists() {
                        info!("BayStore: Auto-provisioning bay for '{}'", stem);
                        self.provision(stem).await?;
                    } else {
                        // Ensure we have a connection even if directory already exists
                        let db_path = bay_dir.join("state.db");
                        if db_path.exists() {
                            let conn = rusqlite::Connection::open(&db_path)?;
                            let mut conns = self.connections.lock().await;
                            conns.insert(stem.to_string(), conn);
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Record a worktree path in the agent's FS Map.
    ///
    /// # Errors
    /// Returns an error if the bay is not provisioned or DB write fails.
    pub async fn record_worktree(
        &self,
        agent_name: &str,
        task_id: &str,
        worktree_path: &Path,
    ) -> Result<()> {
        let conns = self.connections.lock().await;
        let conn = conns
            .get(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Bay not provisioned for '{}'", agent_name))?;

        let now = chrono::Utc::now().timestamp();
        let path_str = worktree_path.to_string_lossy().to_string();

        conn.execute(
            "INSERT OR REPLACE INTO fs_map (task_id, worktree_path, created_at, active)
             VALUES (?1, ?2, ?3, 1)",
            params![task_id, path_str, now],
        )
        .context("Failed to record worktree in bay")?;

        info!(
            "BayStore: Recorded worktree for '{}' task '{}': {}",
            agent_name, task_id, path_str
        );
        Ok(())
    }

    /// Log a state transition in the health log.
    ///
    /// # Errors
    /// Returns an error if the database operation fails.
    pub async fn log_state_transition(
        &self,
        agent_name: &str,
        session_id: &str,
        state: &str,
        message: Option<&str>,
    ) -> Result<()> {
        let conns = self.connections.lock().await;
        let conn = match conns.get(agent_name) {
            Some(c) => c,
            None => return Ok(()), // Bay not provisioned yet, skip silently
        };

        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO health_log (session_id, state, timestamp, message)
             VALUES (?1, ?2, ?3, ?4)",
            params![session_id, state, now, message],
        )
        .context("Failed to log state transition")?;

        Ok(())
    }

    /// Record a completed session in session history.
    ///
    /// # Errors
    /// Returns an error if the database operation fails.
    pub async fn record_session_end(
        &self,
        agent_name: &str,
        session_id: &str,
        start_time: i64,
        eow_path: Option<&str>,
        final_state: &str,
    ) -> Result<()> {
        let conns = self.connections.lock().await;
        let conn = match conns.get(agent_name) {
            Some(c) => c,
            None => return Ok(()),
        };

        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO session_history
             (session_id, agent_name, start_time, end_time, eow_path, final_state)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                session_id,
                agent_name,
                start_time,
                now,
                eow_path,
                final_state
            ],
        )
        .context("Failed to record session end")?;

        Ok(())
    }

    /// Get the health status of a bay by checking the last health_log entry.
    ///
    /// # Errors
    /// Returns an error if the bay is not provisioned or query fails.
    pub async fn get_health(&self, agent_name: &str) -> Result<String> {
        let conns = self.connections.lock().await;
        let conn = conns
            .get(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Bay not provisioned for '{}'", agent_name))?;

        let state: Option<String> = conn
            .query_row(
                "SELECT state FROM health_log ORDER BY timestamp DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .ok();

        match state.as_deref() {
            Some("ACTIVE" | "WORKING") => Ok("GREEN".to_string()),
            Some("DARK") => Ok("YELLOW".to_string()),
            Some("TEARDOWN") => Ok("RED".to_string()),
            _ => Ok("GREEN".to_string()), // No logs yet = fresh bay
        }
    }

    /// Get the XP and Level for an agent.
    pub async fn get_xp_and_level(&self, agent_name: &str) -> Result<(u32, u32)> {
        let conns = self.connections.lock().await;
        let conn = conns
            .get(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Bay not provisioned for '{}'", agent_name))?;

        let (xp, level): (u32, u32) = conn.query_row(
            "SELECT xp, level FROM agent_metadata WHERE agent_name = ?1",
            params![agent_name],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        Ok((xp, level))
    }

    /// Update the XP and Level for an agent.
    pub async fn update_xp_and_level(&self, agent_name: &str, xp: u32, level: u32) -> Result<()> {
        let conns = self.connections.lock().await;
        let conn = conns
            .get(agent_name)
            .ok_or_else(|| anyhow::anyhow!("Bay not provisioned for '{}'", agent_name))?;

        conn.execute(
            "UPDATE agent_metadata SET xp = ?1, level = ?2 WHERE agent_name = ?3",
            params![xp, level, agent_name],
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_bay_provisioning() -> Result<()> {
        let dir = tempdir()?;
        let store = BayStore::new(dir.path().to_path_buf());

        store.provision("test-agent").await?;
        assert!(dir.path().join("test-agent/state.db").exists());

        let health = store.get_health("test-agent").await?;
        assert_eq!(health, "GREEN");

        Ok(())
    }
}
