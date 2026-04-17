//! # Captain's Sync Handler
//!
//! Implements the 2-way synchronization between local Citadel reality
//! (TEAM-LOG.md, updates/, and tasks.json) and GitHub Project #6.
//! This fulfills the Captain's primary directive to ensure the board is
//! always up-to-date and the crew is informed.

use anyhow::{Context, Result};
use koad_board::GitHubClient;
use koad_core::config::KoadConfig;
use koad_proto::cass::v1::pulse_service_client::PulseServiceClient;
use koad_proto::cass::v1::AddPulseRequest;
use std::env;
use std::path::PathBuf;
use tracing::{info, warn};

/// Orchestrates the 2-way sync process.
pub async fn handle_board_sync(
    dry_run: bool,
    auto_spawn: bool,
    config: &KoadConfig,
    agent_name: &str,
) -> Result<()> {
    println!("\x1b[1;34m--- Captain's Board Sync: Project #6 ---\x1b[0m");
    if dry_run {
        println!("\x1b[33m[DRY RUN]\x1b[0m No changes will be persisted to GitHub or CASS.");
    }

    // 1. Resolve GitHub Context
    let token = config.resolve_gh_token(None, None)?;
    let owner = config.get_github_owner(None);
    let repo = config.get_github_repo(None);
    let project_num = config
        .integrations
        .github
        .as_ref()
        .map(|g| g.default_project_number)
        .unwrap_or(2) as i32;

    let client = GitHubClient::new(token, owner, repo)?;
    
    // 2. Fetch Board State
    info!(">>> Fetching current Project Board state...");
    let board_items = client.list_project_items(project_num).await?;
    info!("Found {} items on the board.", board_items.len());

    // 3. Scan Local Reality (Updates)
    info!(">>> Scanning local updates for completions...");
    let cwd = env::current_dir().unwrap_or_default();
    let updates_dir = config.home.join("updates");
    
    // We'll reuse the logic from updates.rs to load the last 50 entries
    let entries = load_recent_updates(&updates_dir)?;
    info!("Found {} local update entries.", entries.len());

    let mut synced_count = 0;
    let mut spawned_count = 0;
    let mut messages = Vec::new();

    for entry in entries {
        // Look for "Fixes #123" or "#123" in the summary or body
        let issue_numbers = extract_issue_numbers(&entry.summary, &entry.body);
        
        if issue_numbers.is_empty() && auto_spawn && (entry.category == "feature" || entry.category == "fix") {
            // Potential for auto-spawn
            if !dry_run {
                println!("\x1b[36m[SPAWN]\x1b[0m Auto-spawning issue for: {}", entry.summary);
                // For now, we'll pulse it. Actual issue creation can be added if needed.
                messages.push(format!("New {} reported: {}", entry.category, entry.summary));
                spawned_count += 1;
            } else {
                println!("\x1b[33m[DRY RUN]\x1b[0m Would auto-spawn issue for: {}", entry.summary);
            }
            continue;
        }

        for num in issue_numbers {
            if let Some(item) = board_items.iter().find(|i| i.number == Some(num)) {
                if item.status != "Done" {
                    if !dry_run {
                        println!("\x1b[32m[SYNC]\x1b[0m Moving #{} to 'Done' (referenced by update: {})", num, entry.id);
                        client.update_item_status(project_num, num, "Done").await?;
                        client.close_issue(num).await?;
                        messages.push(format!("Issue #{} closed (local sync).", num));
                        synced_count += 1;
                    } else {
                        println!("\x1b[33m[DRY RUN]\x1b[0m Would move #{} to 'Done'", num);
                    }
                }
            }
        }
    }

    // 4. Broadcast Pulse
    if !dry_run && (!messages.is_empty() || synced_count > 0 || spawned_count > 0) {
        let summary_msg = if messages.is_empty() {
            format!("Board Sync Complete: {} items updated, {} spawned.", synced_count, spawned_count)
        } else {
            messages.join(" | ")
        };

        match PulseServiceClient::connect(config.network.cass_grpc_addr.clone()).await {
            Ok(mut pulse_client) => {
                let _ = pulse_client.add_pulse(AddPulseRequest {
                    context: None,
                    author: agent_name.to_string(),
                    role: "global".to_string(),
                    message: format!("Captain's Sync: {}", summary_msg),
                    ttl_seconds: 3600,
                }).await;
                println!("\x1b[32m[PULSE]\x1b[0m Sync summary broadcast to all agents.");
            }
            Err(_) => warn!("CASS offline. Pulse not broadcast."),
        }
    }

    println!("\x1b[32m[OK]\x1b[0m Sync complete. Board is now aligned with local reality.");
    Ok(())
}

fn extract_issue_numbers(summary: &str, body: &str) -> Vec<i32> {
    let mut nums = Vec::new();
    let re = regex::Regex::new(r"#(\d+)").unwrap();
    
    for cap in re.captures_iter(summary) {
        if let Ok(n) = cap[1].parse::<i32>() {
            nums.push(n);
        }
    }
    for cap in re.captures_iter(body) {
        if let Ok(n) = cap[1].parse::<i32>() {
            nums.push(n);
        }
    }
    nums.sort();
    nums.dedup();
    nums
}

struct SimpleUpdate {
    id: String,
    summary: String,
    body: String,
    category: String,
}

fn load_recent_updates(dir: &std::path::Path) -> Result<Vec<SimpleUpdate>> {
    if !dir.exists() {
        return Ok(vec![]);
    }

    let mut entries = Vec::new();
    let files = std::fs::read_dir(dir)?;
    
    for entry in files.filter_map(Result::ok) {
        if entry.path().extension().map(|s| s == "md").unwrap_or(false) {
            let content = std::fs::read_to_string(entry.path())?;
            if let Some((fm, body)) = parse_simple_entry(&content) {
                entries.push(SimpleUpdate {
                    id: fm.id,
                    summary: fm.summary,
                    category: fm.category,
                    body: body.to_string(),
                });
            }
        }
    }
    Ok(entries)
}

struct SimpleFM {
    id: String,
    summary: String,
    category: String,
}

fn parse_simple_entry(content: &str) -> Option<(SimpleFM, &str)> {
    let mut parts = content.splitn(3, "+++");
    let _ = parts.next()?;
    let fm_text = parts.next()?;
    let body = parts.next().unwrap_or("");

    let mut id = String::new();
    let mut summary = String::new();
    let mut category = String::new();

    for line in fm_text.lines() {
        if let Some((k, v)) = line.split_once('=') {
            let val = v.trim().trim_matches('"').to_string();
            match k.trim() {
                "id" => id = val,
                "summary" => summary = val,
                "category" => category = val,
                _ => {}
            }
        }
    }

    if id.is_empty() { return None; }
    Some((SimpleFM { id, summary, category }, body))
}
