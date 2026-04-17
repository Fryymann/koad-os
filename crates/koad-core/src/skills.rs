//! Skill Blueprint discovery and capability validation.
//!
//! [`SkillScanner`] walks `$KOAD_HOME/skills/**/*.skill.toml` and
//! deserializes each file into a [`crate::config::SkillBlueprint`].

use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::config::SkillBlueprint;

/// Capability tokens recognized by the KoadOS sandbox policy.
/// A blueprint listing an unknown capability will be rejected by [`validate_capabilities`].
pub const KNOWN_CAPABILITIES: &[&str] = &[
    "fs_read",
    "fs_write",
    "network_in",
    "network_out",
    "exec",
    "secrets",
    "ipc",
];

/// Discovers Skill Blueprints stored in `$KOAD_HOME/skills/`.
pub struct SkillScanner {
    home: PathBuf,
}

impl SkillScanner {
    /// Create a scanner rooted at `home` (typically `$KOAD_HOME`).
    pub fn new(home: impl Into<PathBuf>) -> Self {
        Self { home: home.into() }
    }

    /// Walk `<home>/skills/**/*.skill.toml` and deserialize every blueprint found.
    ///
    /// Errors on individual files are logged as warnings and skipped so one
    /// malformed blueprint does not block the entire scan.
    pub fn scan(&self) -> Result<Vec<SkillBlueprint>> {
        let skills_dir = self.home.join("skills");
        if !skills_dir.exists() {
            return Ok(vec![]);
        }

        let mut blueprints = Vec::new();
        Self::walk(&skills_dir, &mut blueprints)?;
        Ok(blueprints)
    }

    fn walk(dir: &std::path::Path, out: &mut Vec<SkillBlueprint>) -> Result<()> {
        for entry in std::fs::read_dir(dir).with_context(|| format!("reading {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                Self::walk(&path, out)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("toml")
                && path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.ends_with(".skill.toml"))
                    .unwrap_or(false)
            {
                match Self::load_blueprint(&path) {
                    Ok(bp) => out.push(bp),
                    Err(e) => {
                        tracing::warn!("skipping malformed blueprint {}: {e}", path.display());
                    }
                }
            }
        }
        Ok(())
    }

    fn load_blueprint(path: &std::path::Path) -> Result<SkillBlueprint> {
        let raw =
            std::fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        let mut bp: SkillBlueprint =
            toml::from_str(&raw).with_context(|| format!("parsing {}", path.display()))?;
        // Auto-populate `id` from filename if not set in TOML.
        if bp.id.is_empty() {
            bp.id = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .trim_end_matches(".skill.toml")
                .to_string();
        }
        Ok(bp)
    }
}

/// Validate that every capability token in `caps` is in [`KNOWN_CAPABILITIES`].
///
/// Returns `Err` listing any unknown tokens so the caller can surface a clear error.
pub fn validate_capabilities(caps: &[String]) -> Result<()> {
    let unknown: Vec<&str> = caps
        .iter()
        .map(String::as_str)
        .filter(|c| !KNOWN_CAPABILITIES.contains(c))
        .collect();
    if unknown.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "unknown capability tokens: {}. Allowed: {}",
            unknown.join(", "),
            KNOWN_CAPABILITIES.join(", ")
        )
    }
}
