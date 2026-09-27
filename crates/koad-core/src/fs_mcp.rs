//! Root resolution for `koad-fs-mcp`, the per-agent filesystem MCP launcher.
//!
//! `koad-fs-mcp` runs the official `@modelcontextprotocol/server-filesystem`
//! restricted to the directories resolved here, so harness-less and local
//! models get scoped file access through MCP.

use std::path::{Path, PathBuf};

/// Pinned version of the official filesystem MCP server.
pub const FILESYSTEM_SERVER_PACKAGE: &str = "@modelcontextprotocol/server-filesystem@2026.8.31";

/// Resolve the directories an agent may access.
///
/// Combines the global `[filesystem] allowed_directories`, the agent's own
/// `preferences.allowed_directories`, and the agent's vault. Expands `~`,
/// drops duplicates, and skips directories that do not exist.
///
/// Returns the roots in first-seen order and a warning for each skipped entry.
pub fn resolve_roots(
    home: &Path,
    global: &[String],
    agent: &[String],
    vault: Option<&str>,
) -> (Vec<PathBuf>, Vec<String>) {
    let mut roots: Vec<PathBuf> = Vec::new();
    let mut warnings = Vec::new();
    let entries = global
        .iter()
        .chain(agent.iter())
        .map(String::as_str)
        .chain(vault);
    for entry in entries {
        let path = expand_tilde(home, entry);
        if roots.contains(&path) {
            continue;
        }
        if path.is_dir() {
            roots.push(path);
        } else {
            warnings.push(format!("skipping {} (not a directory)", path.display()));
        }
    }
    (roots, warnings)
}

fn expand_tilde(home: &Path, entry: &str) -> PathBuf {
    match entry.strip_prefix('~') {
        Some("") => home.to_path_buf(),
        Some(rest) if rest.starts_with('/') => home.join(rest.trim_start_matches('/')),
        _ => PathBuf::from(entry),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_tilde_dedupes_and_includes_the_vault() {
        let home = tempfile::tempdir().unwrap();
        for d in ["projects", "vault", "shared"] {
            std::fs::create_dir(home.path().join(d)).unwrap();
        }
        let (roots, warnings) = resolve_roots(
            home.path(),
            &["~/projects".into(), "~/shared".into()],
            &["~/projects".into()],
            Some("~/vault"),
        );
        assert_eq!(
            roots,
            vec![
                home.path().join("projects"),
                home.path().join("shared"),
                home.path().join("vault"),
            ]
        );
        assert!(warnings.is_empty(), "{warnings:?}");
    }

    #[test]
    fn skips_missing_directories_with_a_warning() {
        let home = tempfile::tempdir().unwrap();
        std::fs::create_dir(home.path().join("real")).unwrap();
        let (roots, warnings) =
            resolve_roots(home.path(), &["~/real".into(), "~/gone".into()], &[], None);
        assert_eq!(roots, vec![home.path().join("real")]);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("gone"), "{warnings:?}");
    }

    #[test]
    fn bare_tilde_and_absolute_paths_resolve() {
        let home = tempfile::tempdir().unwrap();
        let abs = tempfile::tempdir().unwrap();
        let (roots, _) = resolve_roots(
            home.path(),
            &["~".into(), abs.path().to_string_lossy().into_owned()],
            &[],
            None,
        );
        assert_eq!(
            roots,
            vec![home.path().to_path_buf(), abs.path().to_path_buf()]
        );
    }
}
