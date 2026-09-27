//! File-based agent inbox: `$KOAD_HOME/agents/inbox/<slug>.<type>.<agent>.md`.
//!
//! This is how KoadOS agents hand work to each other. Files addressed to
//! `all_agents` are broadcasts. Handled items move to `archived/`.

use std::path::{Path, PathBuf};

/// An inbox file addressed to an agent.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InboxItem {
    /// Full path to the file.
    pub path: PathBuf,
    /// File name without the agent suffix, e.g. `zombie_redis_reap_bug.task`.
    pub title: String,
}

/// Inbox directory under a KoadOS home.
pub fn inbox_dir(home: &Path) -> PathBuf {
    home.join("agents").join("inbox")
}

/// Items in `dir` addressed to `agent` (case-insensitive) or to `all_agents`,
/// newest first. Subdirectories such as `archived/` are ignored. A missing
/// directory yields an empty list.
pub fn pending_for(dir: &Path, agent: &str) -> Vec<InboxItem> {
    let suffixes = [
        format!(".{}.md", agent.to_lowercase()),
        ".all_agents.md".to_string(),
    ];
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut items: Vec<(std::time::SystemTime, InboxItem)> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_lowercase();
            let suffix = suffixes.iter().find(|s| name.ends_with(s.as_str()))?;
            let title = name[..name.len() - suffix.len()].to_string();
            let modified = e
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(std::time::UNIX_EPOCH);
            Some((
                modified,
                InboxItem {
                    path: e.path(),
                    title,
                },
            ))
        })
        .collect();
    items.sort_by(|a, b| b.0.cmp(&a.0));
    items.into_iter().map(|(_, item)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_items_for_the_agent_and_broadcasts_only() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path();
        for name in [
            "zombie_bug.task.clyde.md",
            "restart.report.hermes.md",
            "deploy.message.all_agents.md",
            "shared_plan.md",
            "notes.clyde.txt",
        ] {
            std::fs::write(p.join(name), "x").unwrap();
        }
        std::fs::create_dir(p.join("archived")).unwrap();
        std::fs::write(p.join("archived").join("old.task.clyde.md"), "x").unwrap();

        let mut titles: Vec<String> = pending_for(p, "Clyde")
            .into_iter()
            .map(|i| i.title)
            .collect();
        titles.sort();
        assert_eq!(titles, vec!["deploy.message", "zombie_bug.task"]);
    }

    #[test]
    fn missing_directory_is_empty_not_an_error() {
        assert!(pending_for(Path::new("/nonexistent/koad/inbox"), "clyde").is_empty());
    }
}
