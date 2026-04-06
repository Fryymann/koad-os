//! Updates board handler — chronological devlog for Citadel, Station, and Outpost levels.
//!
//! Manages a file-based feed at `<workspace>/updates/`. Entries use TOML frontmatter
//! (`+++...+++`) and are named `YYYYMMDD_HHMMSS_<slug>.md` for deterministic sort order.
//! The `digest` subcommand outputs compact markdown designed as the CASS TCH contract
//! (Phase 2+ context hydration feed-in format).

use crate::cli::UpdatesAction;
use anyhow::Result;
use chrono::Utc;
use koad_core::config::KoadConfig;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::instrument;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[allow(dead_code)]
struct UpdateEntry {
    id: String,
    timestamp: String,
    author: String,
    level: String,
    category: String,
    summary: String,
    body: String,
    filename: String,
}

// ---------------------------------------------------------------------------
// Public entrypoint
// ---------------------------------------------------------------------------

/// Dispatch an [`UpdatesAction`] to the appropriate handler.
///
/// # Errors
/// Returns an error if the underlying file I/O or display operation fails.
#[instrument(skip(config))]
pub async fn handle_updates_action(action: UpdatesAction, config: &KoadConfig) -> Result<()> {
    let cwd = env::current_dir().unwrap_or_default();
    match action {
        UpdatesAction::Post { summary, category, body, level, author } => {
            let pulse_summary = summary.clone();
            let pulse_author = author.clone();
            post_update(config, &cwd, summary, category, body, level, author)?;
            // Trigger a best-effort pulse signal — silent on CASS offline.
            let pulse_msg = format!("Update posted: {}", pulse_summary);
            let pulse_author_str = pulse_author.unwrap_or_else(|| {
                env::var("KOAD_AGENT_NAME").unwrap_or_else(|_| "unknown".to_string())
            });
            if let Ok(mut client) = koad_proto::cass::v1::pulse_service_client::PulseServiceClient::connect(
                config.network.cass_grpc_addr.clone(),
            )
            .await
            {
                let _ = client
                    .add_pulse(koad_proto::cass::v1::AddPulseRequest {
                        context: None,
                        author: pulse_author_str,
                        role: "global".to_string(),
                        message: pulse_msg,
                        ttl_seconds: 3600,
                    })
                    .await;
            }
        }
        UpdatesAction::List { limit, author, category, level } => {
            list_updates(config, &cwd, limit, author, category, level)?;
        }
        UpdatesAction::Show { id } => {
            show_update(config, &cwd, &id)?;
        }
        UpdatesAction::Digest { limit, level } => {
            digest_updates(config, &cwd, limit, level)?;
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Level + directory resolution
// ---------------------------------------------------------------------------

fn resolve_level(config: &KoadConfig, cwd: &Path, override_level: Option<String>) -> String {
    if let Some(l) = override_level {
        return l.to_lowercase();
    }
    if cwd.starts_with(&config.home) {
        "citadel".to_string()
    } else if config.resolve_project_context(cwd).is_some() {
        "station".to_string()
    } else {
        "outpost".to_string()
    }
}

fn updates_dir(config: &KoadConfig, cwd: &Path, level: &str) -> PathBuf {
    match level {
        "citadel" => config.home.join("updates"),
        _ => cwd.join("updates"),
    }
}

// ---------------------------------------------------------------------------
// Slug helper
// ---------------------------------------------------------------------------

fn slugify(s: &str) -> String {
    let raw: String = s
        .chars()
        .map(|c| if c.is_alphanumeric() { c.to_ascii_lowercase() } else { '-' })
        .collect();
    raw.split('-')
        .filter(|p| !p.is_empty())
        .take(5)
        .collect::<Vec<_>>()
        .join("-")
}

// ---------------------------------------------------------------------------
// Post
// ---------------------------------------------------------------------------

fn post_update(
    config: &KoadConfig,
    cwd: &Path,
    summary: String,
    category: String,
    body: Option<String>,
    level: Option<String>,
    author: Option<String>,
) -> Result<()> {
    let level = resolve_level(config, cwd, level);
    let dir = updates_dir(config, cwd, &level);
    fs::create_dir_all(&dir)?;

    let now = Utc::now();
    let timestamp = now.to_rfc3339();
    let slug = slugify(&summary);
    let id = format!("upd_{}_{}", now.format("%Y%m%d_%H%M%S"), slug);
    let filename = format!("{}_{}.md", now.format("%Y%m%d_%H%M%S"), slug);

    let agent_name = author.unwrap_or_else(|| {
        env::var("KOAD_AGENT_NAME").unwrap_or_else(|_| "unknown".to_string())
    });

    let body_text = body.unwrap_or_default();
    let content = format!(
        "+++\nid        = \"{id}\"\ntimestamp = \"{timestamp}\"\nauthor    = \"{agent_name}\"\nlevel     = \"{level}\"\ncategory  = \"{category}\"\nsummary   = \"{summary}\"\n+++\n\n{body_text}\n",
    );

    let path = dir.join(&filename);
    fs::write(&path, &content)?;

    println!("\x1b[32m[OK]\x1b[0m Update posted.");
    println!("     ID:   {}", id);
    println!("     File: {}", path.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Frontmatter parser
// ---------------------------------------------------------------------------

fn parse_entry(filename: &str, content: &str) -> Option<UpdateEntry> {
    // File format: +++\n<frontmatter>\n+++\n\n<body>
    let mut parts = content.splitn(3, "+++");
    let _ = parts.next()?; // empty string before first +++
    let frontmatter = parts.next()?;
    let body = parts.next().unwrap_or("").trim().to_string();

    let mut id = String::new();
    let mut timestamp = String::new();
    let mut author = String::new();
    let mut level = String::new();
    let mut category = String::new();
    let mut summary = String::new();

    for line in frontmatter.lines() {
        if let Some((k, v)) = line.split_once('=') {
            let val = v.trim().trim_matches('"').to_string();
            match k.trim() {
                "id"        => id = val,
                "timestamp" => timestamp = val,
                "author"    => author = val,
                "level"     => level = val,
                "category"  => category = val,
                "summary"   => summary = val,
                _ => {}
            }
        }
    }

    if id.is_empty() {
        return None;
    }

    Some(UpdateEntry {
        id,
        timestamp,
        author,
        level,
        category,
        summary,
        body,
        filename: filename.to_string(),
    })
}

// ---------------------------------------------------------------------------
// Load entries from a directory (sorted newest-first)
// ---------------------------------------------------------------------------

fn load_entries(
    dir: &Path,
    author_filter: Option<&str>,
    category_filter: Option<&str>,
    limit: usize,
) -> Result<Vec<UpdateEntry>> {
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut files: Vec<_> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        .collect();

    // Descending by filename (YYYYMMDD_HHMMSS prefix ensures correct order)
    files.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

    let mut entries = Vec::new();
    for file in files {
        if entries.len() >= limit {
            break;
        }
        let fname = file.file_name().to_string_lossy().to_string();
        let content = fs::read_to_string(file.path()).unwrap_or_default();
        if let Some(entry) = parse_entry(&fname, &content) {
            if let Some(a) = author_filter {
                if !entry.author.eq_ignore_ascii_case(a) {
                    continue;
                }
            }
            if let Some(c) = category_filter {
                if !entry.category.eq_ignore_ascii_case(c) {
                    continue;
                }
            }
            entries.push(entry);
        }
    }
    Ok(entries)
}

// ---------------------------------------------------------------------------
// List
// ---------------------------------------------------------------------------

fn list_updates(
    config: &KoadConfig,
    cwd: &Path,
    limit: usize,
    author: Option<String>,
    category: Option<String>,
    level: Option<String>,
) -> Result<()> {
    let level = resolve_level(config, cwd, level);
    let dir = updates_dir(config, cwd, &level);
    let entries = load_entries(&dir, author.as_deref(), category.as_deref(), limit)?;

    if entries.is_empty() {
        println!(
            "\x1b[33m[EMPTY]\x1b[0m No updates found on the {} board.",
            level.to_uppercase()
        );
        return Ok(());
    }

    println!(
        "\x1b[1;34m--- Updates Board: {} ({} entries) ---\x1b[0m",
        level.to_uppercase(),
        entries.len()
    );
    for e in &entries {
        let date = e.timestamp.get(..10).unwrap_or(&e.timestamp);
        println!(
            "  \x1b[36m{}\x1b[0m [\x1b[33m{}\x1b[0m] {} \x1b[2m— {}\x1b[0m",
            date, e.category, e.summary, e.author
        );
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Show
// ---------------------------------------------------------------------------

fn show_update(config: &KoadConfig, cwd: &Path, id: &str) -> Result<()> {
    for level in &["citadel", "station", "outpost"] {
        let dir = updates_dir(config, cwd, level);
        if !dir.exists() {
            continue;
        }
        for file in fs::read_dir(&dir)?.filter_map(|e| e.ok()) {
            let fname = file.file_name().to_string_lossy().to_string();
            let content = fs::read_to_string(file.path()).unwrap_or_default();
            if let Some(entry) = parse_entry(&fname, &content) {
                if entry.id == id || fname.contains(id) {
                    println!("\x1b[1m{}\x1b[0m", entry.summary);
                    println!(
                        "ID: {}  |  Author: {}  |  Category: {}  |  Level: {}",
                        entry.id, entry.author, entry.category, entry.level
                    );
                    println!("Timestamp: {}", entry.timestamp);
                    if !entry.body.is_empty() {
                        println!("\n{}", entry.body);
                    }
                    return Ok(());
                }
            }
        }
    }
    anyhow::bail!("No update found matching '{}'", id)
}

// ---------------------------------------------------------------------------
// Digest — compact markdown output for CASS context hydration
// ---------------------------------------------------------------------------

fn digest_updates(
    config: &KoadConfig,
    cwd: &Path,
    limit: usize,
    level: Option<String>,
) -> Result<()> {
    let level = resolve_level(config, cwd, level);
    let dir = updates_dir(config, cwd, &level);
    let entries = load_entries(&dir, None, None, limit)?;

    if entries.is_empty() {
        return Ok(());
    }

    let label = {
        let mut s = level.clone();
        if let Some(c) = s.get_mut(0..1) {
            c.make_ascii_uppercase();
        }
        s
    };

    println!("## Recent Updates ({})", label);
    for e in &entries {
        let date = e.timestamp.get(..10).unwrap_or(&e.timestamp);
        println!("- {} [{}] {} — {}", date, e.category, e.summary, e.author);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
    }

    #[test]
    fn test_slugify_truncates_to_five_words() {
        assert_eq!(
            slugify("one two three four five six seven"),
            "one-two-three-four-five"
        );
    }

    #[test]
    fn test_slugify_strips_special_chars() {
        assert_eq!(slugify("fix: boot (KOAD_BIN)"), "fix-boot-koad-bin");
    }

    #[test]
    fn test_parse_entry_valid() {
        let content = "+++\nid        = \"upd_001\"\ntimestamp = \"2026-03-22T00:00:00Z\"\nauthor    = \"clyde\"\nlevel     = \"citadel\"\ncategory  = \"ops\"\nsummary   = \"Test update\"\n+++\n\nBody text here.";
        let entry = parse_entry("upd_001.md", content).expect("should parse valid entry");
        assert_eq!(entry.id, "upd_001");
        assert_eq!(entry.author, "clyde");
        assert_eq!(entry.summary, "Test update");
        assert_eq!(entry.body, "Body text here.");
    }

    #[test]
    fn test_parse_entry_missing_id_returns_none() {
        let content = "+++\nauthor = \"clyde\"\n+++\n\nBody.";
        assert!(parse_entry("bad.md", content).is_none());
    }

    #[test]
    fn test_load_entries_missing_dir_returns_empty() {
        // Non-existent dir should return empty vec, not error
        let dir = std::path::Path::new("/tmp/koad_test_nonexistent_updates_dir_xyz");
        let result = load_entries(dir, None, None, 10).expect("should not error on missing dir");
        assert!(result.is_empty());
    }
}
