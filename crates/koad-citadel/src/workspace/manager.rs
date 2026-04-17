//! Workspace Manager
//!
//! Manages isolated Git worktrees for agent tasks.

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing::{info, warn};

use crate::state::bay_store::BayStore;

/// Manages isolated Git worktrees for agent tasks.
#[derive(Clone)]
pub struct WorkspaceManager {
    /// Base path for all worktrees: e.g., `~/.koad-os/workspaces/`
    base_path: PathBuf,
    /// Repository root for git worktree operations.
    repo_root: PathBuf,
}

impl WorkspaceManager {
    /// Creates a new `WorkspaceManager`.
    pub fn new(base_path: PathBuf, repo_root: PathBuf) -> Self {
        Self {
            base_path,
            repo_root,
        }
    }

    /// Create a git worktree for an agent's task.
    ///
    /// # Errors
    /// Returns an error if the parent directories cannot be created or if the
    /// `git worktree add` command fails.
    pub async fn create_worktree(
        &self,
        agent_name: &str,
        task_id: &str,
        bay_store: &BayStore,
    ) -> Result<PathBuf> {
        let worktree_path = self.base_path.join(agent_name).join(task_id);

        // Create parent directories
        if let Some(parent) = worktree_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create workspace parent dir for {}", agent_name)
            })?;
        }

        // Branch name derived from task_id
        let branch_name = format!("{}/{}", agent_name.to_lowercase(), task_id);

        // Run git worktree add
        let status = tokio::process::Command::new("git")
            .args([
                "worktree",
                "add",
                "-b",
                &branch_name,
                &worktree_path.to_string_lossy(),
                "HEAD",
            ])
            .current_dir(&self.repo_root)
            .output()
            .await
            .context("Failed to execute 'git worktree add' command")?;

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            anyhow::bail!(
                "git worktree add failed for task '{}': {}",
                task_id,
                stderr.trim()
            );
        }

        // Record in Personal Bay FS Map
        bay_store
            .record_worktree(agent_name, task_id, &worktree_path)
            .await?;

        info!(
            "WorkspaceManager: Created worktree for '{}' task '{}' at {:?}",
            agent_name, task_id, worktree_path
        );

        Ok(worktree_path)
    }

    /// Remove a worktree when a task is complete.
    ///
    /// This function is best-effort and will only log a warning if removal fails,
    /// as stale worktrees can be manually cleaned up.
    pub async fn remove_worktree(&self, agent_name: &str, task_id: &str) -> Result<()> {
        let worktree_path = self.base_path.join(agent_name).join(task_id);

        if !worktree_path.exists() {
            warn!(
                "WorkspaceManager: Worktree not found for removal: {:?}",
                worktree_path
            );
            return Ok(());
        }

        let status = tokio::process::Command::new("git")
            .args(["worktree", "remove", &worktree_path.to_string_lossy()])
            .current_dir(&self.repo_root)
            .output()
            .await
            .context("Failed to execute 'git worktree remove' command")?;

        if !status.status.success() {
            let stderr = String::from_utf8_lossy(&status.stderr);
            warn!(
                "WorkspaceManager: git worktree remove failed: {}",
                stderr.trim()
            );
        }

        Ok(())
    }
}
