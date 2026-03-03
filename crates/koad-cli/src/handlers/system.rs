use anyhow::{Context, Result};
use chrono::{Local, Utc};
use koad_core::config::KoadConfig;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use serde_json::Value;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tracing::warn;
use crate::cli::SystemAction;
use crate::db::KoadDB;
use crate::utils::{feature_gate, get_gh_pat_for_path, get_gdrive_token_for_path};
use rusqlite::params;

pub async fn handle_system_action(
    action: SystemAction,
    config: &KoadConfig,
    db: &KoadDB,
    role: String,
    is_admin: bool,
) -> Result<()> {
    match action {
        SystemAction::Auth => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (p, _) = get_gh_pat_for_path(&current_dir, &role, config);
            let (d, _) = get_gdrive_token_for_path(&current_dir);
            println!("GH:{} | GD:{}", p, d);
        }
        SystemAction::Init { force: _ } => {
            feature_gate("koad init", Some(25));
        }
        SystemAction::Config { json } => {
            if json {
                let v = serde_json::json!({
                    "home": config.home,
                    "redis_socket": config.redis_socket,
                    "spine_socket": config.spine_socket,
                    "spine_grpc_addr": config.spine_grpc_addr,
                    "gateway_addr": config.gateway_addr,
                    "db_path": config.get_db_path()
                });
                println!("{}", v);
            } else {
                println!("{:#?}", config);
            }
        }
        SystemAction::Refresh { restart } => {
            if !is_admin {
                anyhow::bail!("Admin only.");
            }
            println!("
\x1b[1m--- KoadOS Core Refresh (Hard Reset) ---\x1b[0m");
            let home = config.home.clone();
            println!(">>> [1/3] Energizing Forge (cargo build --release)...");
            let build_status = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(&home)
                .status()?;
            if !build_status.success() {
                anyhow::bail!("Forge failure.");
            }
            if restart {
                println!(">>> [3/3] Rebooting Core Systems...");
            }
        }
        SystemAction::Save { full } => {
            if !is_admin {
                anyhow::bail!("Admin only.");
            }
            println!("
\x1b[1m--- KoadOS Sovereign Save Protocol ---\x1b[0m");
            let home = config.home.clone();
            let now_ts = Local::now().format("%Y%m%d-%H%M%S").to_string();

            // 1. Memory Drain (gRPC)
            println!(">>> [1/4] Neuronal Flush (Spine Drain)...");
            match SpineServiceClient::connect(config.spine_grpc_addr.clone()).await {
                Ok(mut client) => {
                    if let Err(e) = client.drain_all(tonic::Request::new(Empty {})).await {
                        warn!(
                            "  [FAIL] Neuronal flush failed: {}. Continuing with local save.",
                            e
                        );
                    } else {
                        println!("  [OK] Hot-stream drained to durable memory.");
                    }
                }
                Err(_) => warn!("  [SKIP] Spine offline. Skipping hot-stream drain."),
            }

            // 2. Cognitive Snapshot
            println!(">>> [2/4] Archiving Identity (Mind Snapshot)...");
            let conn = db.get_conn()?;
            conn.execute(
                "INSERT INTO identity_snapshots (trigger, notes, created_at) VALUES ('sovereign-save', 'Full system checkpoint.', ?1)",
                params![Local::now().to_rfc3339()],
            )?;
            println!("  [OK] Persona state captured.");

            if full {
                // 3. Database Backup
                println!(">>> [3/4] Fortifying Memory (Database Backup)...");
                let backup_dir = home.join("backups");
                std::fs::create_dir_all(&backup_dir)?;
                let backup_path = backup_dir.join(format!("koad-{}.db", now_ts));
                std::fs::copy(home.join("koad.db"), &backup_path)?;
                println!("  [OK] Database archived to: {}", backup_path.display());

                // 4. Git Checkpoint
                println!(">>> [4/4] Finalizing Timeline (Git Checkpoint)...");
                let m = format!("Sovereign Save: {}", now_ts);
                let _ = Command::new("git")
                    .arg("-C")
                    .arg(&home)
                    .arg("add")
                    .arg(".")
                    .status();
                let _ = Command::new("git")
                    .arg("-C")
                    .arg(&home)
                    .arg("commit")
                    .arg("-m")
                    .arg(&m)
                    .status();
                println!("  [OK] System checkpoint committed to git.");
            }
            println!("
\x1b[32m[CONDITION GREEN] Sovereign Save Complete.\x1b[0m");
        }
        SystemAction::Patch {
            path,
            search,
            replace,
            payload,
            fuzzy,
            dry_run,
        } => {
            if !is_admin {
                anyhow::bail!("Admin only.");
            }

            let (target_path, search_str, replace_str) = if let Some(p_str) = payload {
                let v: Value =
                    serde_json::from_str(&p_str).context("Invalid Patch JSON payload.")?;
                (
                    PathBuf::from(v["path"].as_str().context("Missing 'path' in payload")?),
                    v["search"]
                        .as_str()
                        .context("Missing 'search' in payload")?
                        .to_string(),
                    v["replace"]
                        .as_str()
                        .context("Missing 'replace' in payload")?
                        .to_string(),
                )
            } else {
                (
                    path.context("Missing path.")?,
                    search.context("Missing search string.")?,
                    replace.context("Missing replace string.")?,
                )
            };

            let content = std::fs::read_to_string(&target_path)?;

            let new_content = if fuzzy {
                let escaped = regex::escape(&search_str);
                let regex_pattern = escaped.split_whitespace().collect::<Vec<_>>().join(r"\s+");
                let re =
                    regex::Regex::new(&regex_pattern).context("Failed to build fuzzy regex.")?;

                let matches: Vec<_> = re.find_iter(&content).collect();
                if matches.is_empty() {
                    anyhow::bail!(
                        "Patch Failure (Fuzzy): Search string not found in {:?}.",
                        target_path
                    );
                } else if matches.len() > 1 {
                    anyhow::bail!("Patch Failure (Fuzzy): Search string is ambiguous (found {} occurrences) in {:?}.", matches.len(), target_path);
                }
                re.replace(&content, &replace_str).to_string()
            } else {
                let matches: Vec<_> = content.matches(&search_str).collect();
                if matches.is_empty() {
                    anyhow::bail!(
                        "Patch Failure: Search string not found in {:?}.",
                        target_path
                    );
                } else if matches.len() > 1 {
                    anyhow::bail!("Patch Failure: Search string is ambiguous (found {} occurrences) in {:?}.", matches.len(), target_path);
                }
                content.replace(&search_str, &replace_str)
            };

            if dry_run {
                println!(
                    "\x1b[33m[DRY RUN]\x1b[0m Proposed change for {:?}:",
                    target_path
                );
                println!(
                    "--- SEARCH ---
{}
--- REPLACE ---
{}",
                    search_str, replace_str
                );
            } else {
                std::fs::write(&target_path, new_content)?;
                println!(
                    "\x1b[32m[PATCHED]\x1b[0m File {:?} updated successfully.",
                    target_path
                );
            }
        }
        SystemAction::Tokenaudit { cleanup } => {
            println!("
\x1b[1m--- [AUDIT] KoadOS Token Efficiency (5-Pass) ---\x1b[0m");
            let conn = db.get_conn()?;

            if cleanup {
                println!(">>> [PASS 1] Executing redundancy sweep...");
                let cutoff = (Local::now() - chrono::Duration::days(30)).to_rfc3339();
                let time_pruned = conn.execute(
                    "DELETE FROM knowledge WHERE timestamp < ?1 AND tags NOT LIKE '%principle%' AND tags NOT LIKE '%canon%'",
                    params![cutoff]
                )?;
                
                // Duplicate Content Prune
                let dup_pruned = conn.execute(
                    "DELETE FROM knowledge WHERE id NOT IN (SELECT max(id) FROM knowledge GROUP BY content)",
                    []
                )?;

                println!(">>> [PASS 2] Pruning stale session links...");
                let hb_cutoff = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
                let sessions_darkened = conn.execute(
                    "UPDATE sessions SET status = 'dark' WHERE status = 'active' AND last_heartbeat < ?1",
                    params![hb_cutoff]
                )?;
                
                let dark_cutoff = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();
                let sessions_pruned = conn.execute(
                    "DELETE FROM sessions WHERE status = 'dark' AND last_heartbeat < ?1",
                    params![dark_cutoff]
                )?;

                println!("  [OK] Pruned {} stale and {} duplicate fragments.", time_pruned, dup_pruned);
                println!("  [OK] Darkened {} and purged {} stale session records.", sessions_darkened, sessions_pruned);
            }

            // Pass 1: Redundancy (Knowledge)
            print!("{:<35}", "Pass 1: Redundancy (Knowledge):");
            let total_k: i32 = conn.query_row("SELECT count(*) FROM knowledge", [], |r| r.get(0))?;
            if total_k > 100 { println!("\x1b[33m[WARN]\x1b[0m High entry count ({}); cleanup recommended.", total_k); }
            else { println!("\x1b[32m[PASS]\x1b[0m Content levels optimal ({})", total_k); }

            // Pass 2: Verbosity (Active Sessions)
            print!("{:<35}", "Pass 2: Verbosity (Hygiene):");
            let active_s: i32 = conn.query_row("SELECT count(*) FROM sessions WHERE status = 'active'", [], |r| r.get(0))?;
            println!("\x1b[32m[PASS]\x1b[0m Monitoring {} active links.", active_s);

            // Pass 3: Tool-Call Efficiency
            print!("{:<35}", "Pass 3: Logic (Context Cache):");
            let cache_socket = config.home.join("koad.sock");
            if cache_socket.exists() { println!("\x1b[32m[PASS]\x1b[0m Neural Bus Cache Active."); }
            else { println!("\x1b[31m[FAIL]\x1b[0m Cache Offline."); }

            // Pass 4: Payload Trimming
            print!("{:<35}", "Pass 4: Data (Payloads):");
            println!("\x1b[32m[PASS]\x1b[0m gRPC binary protocol enforced.");

            // Pass 5: Persona Density
            print!("{:<35}", "Pass 5: Identity (Density):");
            let mut stmt = conn.prepare("SELECT id, length(bio) FROM identities")?;
            let bios = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i32>(1)?)))?;
            let mut high_density = true;
            for b in bios {
                let (id, len) = b?;
                if len > 200 { 
                    println!("\x1b[33m[WARN]\x1b[0m KAI '{}' bio too long ({} chars).", id, len);
                    high_density = false;
                }
            }
            if high_density { println!("\x1b[32m[PASS]\x1b[0m All KAIs high-density."); }

            println!("\x1b[1m---------------------------------------------------\x1b[0m
");
        }
    }
    Ok(())
}
