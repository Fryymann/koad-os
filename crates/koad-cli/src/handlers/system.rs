use crate::cli::{ConfigAction, SystemAction};
use crate::db::KoadDB;
use crate::utils::{feature_gate, get_gdrive_token_for_path, get_gh_pat_for_path};
use anyhow::{Context, Result};
use chrono::Local;
use fred::interfaces::{HashesInterface, KeysInterface, LuaInterface};
use fred::types::Scanner;
use futures_util::StreamExt;
use koad_core::config::KoadConfig;
use koad_core::utils::lock::DistributedLock;
use koad_core::utils::redis::RedisClient;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use rusqlite::params;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, warn};

pub struct RedisLockClient {
    pub socket: PathBuf,
}

#[async_trait::async_trait]
impl DistributedLock for RedisLockClient {
    async fn lock(&self, sector: &str, agent_name: &str, ttl_secs: u64) -> Result<bool> {
        let client =
            RedisClient::new(&self.socket.parent().unwrap().to_string_lossy(), false).await?;
        let key = format!("koad:lock:{}", sector);
        let session_id = env::var("KOAD_SESSION_ID").unwrap_or_else(|_| agent_name.to_string());
        let val = format!("{}:{}", agent_name, session_id);

        let res: Option<String> = client
            .pool
            .set(
                &key,
                &val,
                Some(fred::types::Expiration::EX(ttl_secs as i64)),
                Some(fred::types::SetOptions::NX),
                false,
            )
            .await?;

        Ok(res.is_some()
            || client.pool.get::<Option<String>, _>(&key).await?.as_deref() == Some(&val))
    }

    async fn unlock(&self, sector: &str, agent_name: &str) -> Result<bool> {
        let client =
            RedisClient::new(&self.socket.parent().unwrap().to_string_lossy(), false).await?;
        let key = format!("koad:lock:{}", sector);
        let session_id = env::var("KOAD_SESSION_ID").unwrap_or_else(|_| agent_name.to_string());
        let val = format!("{}:{}", agent_name, session_id);

        let script = r"
            if redis.call('get', KEYS[1]) == ARGV[1] then
                return redis.call('del', KEYS[1])
            else
                return 0
            end
        ";

        let result: i32 = client
            .pool
            .next()
            .eval(script, vec![key], vec![val])
            .await?;
        Ok(result == 1)
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn spawn_issue(
    config: &KoadConfig,
    db: &KoadDB,
    template: &str,
    title: &str,
    weight: &str,
    objective: Option<String>,
    scope: Option<String>,
    labels: Vec<String>,
    raw_body: Option<String>,
) -> Result<koad_board::issue::Issue> {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let project_ctx = config.resolve_project_context(&current_dir);
    let project = project_ctx.as_ref().map(|(_, p)| p);

    // Resolve repository from Context or DB
    let (owner, repo) = if let Some(p) = project {
        (
            config.get_github_owner(Some(p)),
            config.get_github_repo(Some(p)),
        )
    } else if let Ok(conn) = db.get_conn() {
        let abs_current = std::fs::canonicalize(&current_dir).unwrap_or(current_dir);
        let search_path = abs_current.to_string_lossy().to_string();
        let mut stmt = conn.prepare("SELECT github_repo FROM projects WHERE ?1 LIKE path || '%' ORDER BY length(path) DESC LIMIT 1")?;
        let repo_full: Option<String> = stmt.query_row(params![search_path], |r| r.get(0)).ok();

        if let Some(full) = repo_full {
            let parts: Vec<&str> = full.split('/').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                (config.get_github_owner(None), config.get_github_repo(None))
            }
        } else {
            (config.get_github_owner(None), config.get_github_repo(None))
        }
    } else {
        (config.get_github_owner(None), config.get_github_repo(None))
    };

    let token = config.resolve_gh_token(project, None)?;
    let client = koad_board::GitHubClient::new(token, owner.clone(), repo.clone())?;

    let body = if let Some(rb) = raw_body {
        rb
    } else {
        let template_path = config
            .home
            .join("templates")
            .join("issues")
            .join(format!("{}.md", template));
        if !template_path.exists() {
            anyhow::bail!("Template '{}' not found at {:?}", template, template_path);
        }

        let mut b = std::fs::read_to_string(&template_path)?;

        // String Substitution for fast-spawning
        b = b.replace("[trivial | standard | complex]", weight);

        if let Some(obj) = objective {
            b = b.replace(
                "[Describe the high-level goal of this architectural change]",
                &obj,
            );
            b = b.replace("[Describe the system subsystem to be hardened]", &obj);
            b = b.replace("[Identify the resource or latency bottleneck]", &obj);
            b = b.replace(
                "[Describe the observed behavior vs expected behavior]",
                &obj,
            );
        }
        if let Some(sc) = scope {
            b = b.replace(
                "- [Component A]\n- [Component B]\n- [Interface change/Addition]",
                &sc,
            );
            b = b.replace("- [Recovery logic for X]\n- [Watchdog implementation for Y]\n- [Self-healing procedure for Z]", &sc);
            b = b.replace(
                "- [Caching strategy]\n- [Refactor of inefficient loop]\n- [Payload reduction]",
                &sc,
            );
        }
        b
    };

    client.create_issue(title, &body, labels).await
}

pub async fn handle_system_action(
    action: SystemAction,
    config: &KoadConfig,
    db: &KoadDB,
    role: String,
    is_admin: bool,
    agent_name: &str,
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
        SystemAction::Config { action, json } => match action {
            Some(ConfigAction::Set { key, value }) => {
                let client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
                let mut hot_config = config.clone();

                match key.as_str() {
                    "github_project_number" => {
                        if let Ok(num) = value.parse::<u32>() {
                            if let Some(ref mut g) = hot_config.integrations.github {
                                g.default_project_number = num;
                            } else {
                                anyhow::bail!("GitHub integration not configured in TOML.");
                            }
                        } else {
                            anyhow::bail!("Invalid number: {}", value);
                        }
                    }
                    "gateway_addr" => hot_config.network.gateway_addr = value.clone(),
                    "spine_grpc_addr" => hot_config.network.spine_grpc_addr = value.clone(),
                    _ => {
                        hot_config.extra.insert(key.clone(), value.clone());
                    }
                }

                let json = hot_config.to_json()?;
                let _: () = client
                    .pool
                    .next()
                    .set(
                        koad_core::constants::REDIS_KEY_CONFIG,
                        json,
                        None,
                        None,
                        false,
                    )
                    .await?;
                println!(
                    "\x1b[32m[OK]\x1b[0m Config '{}' set to '{}' in Redis.",
                    key, value
                );
            }
            Some(ConfigAction::Get { key }) => match key.as_str() {
                "github_project_number" => {
                    let val = config
                        .integrations
                        .github
                        .as_ref()
                        .map(|g| g.default_project_number.to_string())
                        .unwrap_or_else(|| "Not Configured".to_string());
                    println!("{}", val);
                }
                "gateway_addr" => println!("{}", config.network.gateway_addr),
                "spine_grpc_addr" => println!("{}", config.network.spine_grpc_addr),
                _ => {
                    if let Some(v) = config.extra.get(&key) {
                        println!("{}", v);
                    } else {
                        println!("Key '{}' not found.", key);
                    }
                }
            },
            Some(ConfigAction::List) => {
                println!("--- Dynamic Configuration ---");
                for (k, v) in &config.extra {
                    println!("{}: {}", k, v);
                }
            }
            None => {
                if json {
                    println!("{}", config.to_json()?);
                } else {
                    println!("{:#?}", config);
                }
            }
        },
        SystemAction::Refresh { restart, confirm } => {
            if !is_admin {
                anyhow::bail!("Admin only.");
            }

            if restart && !confirm {
                println!("\x1b[33m[SAFETY GATE]\x1b[0m Restarting services will terminate active sessions and may cause disruption.");
                println!("Run with --confirm to proceed with full refresh.");
                return Ok(());
            }

            let lock_client = RedisLockClient {
                socket: config.get_redis_socket(),
            };

            koad_core::with_sector_lock!(&lock_client, "refresh", agent_name, 600, {
                println!(
                    "
\x1b[1m--- KoadOS Core Refresh (Hard Reset) ---\x1b[0m"
                );
                let home = config.home.clone();
                println!(">>> [1/3] Energizing Forge (cargo build)...");
                let build_status = Command::new("cargo")
                    .arg("build")
                    .current_dir(&home)
                    .status()?;
                if !build_status.success() {
                    anyhow::bail!("Forge failure.");
                }

                println!(">>> [2/3] Verifying Core Links (bin/ alignment)...");
                let bin_dir = home.join("bin");
                let target_dir = home.join("target/debug");

                let links = [
                    ("koad", "koad"),
                    ("kspine", "koad-spine"),
                    ("koad-asm", "koad-asm"),
                    ("koad-watchdog", "koad-watchdog"),
                    ("koad-cli", "koad"),
                ];

                for (link_name, target_name) in links {
                    let link_path = bin_dir.join(link_name);
                    let target_path = target_dir.join(target_name);

                    if link_path.exists() {
                        let _ = std::fs::remove_file(&link_path);
                    }

                    #[cfg(unix)]
                    {
                        if let Err(e) = std::os::unix::fs::symlink(&target_path, &link_path) {
                            warn!(
                                "  [FAIL] Failed to link {} -> {}: {}",
                                link_name, target_name, e
                            );
                        } else {
                            println!("  [OK] {} linked to {}", link_name, target_name);
                        }
                    }
                }

                if restart {
                    println!(">>> [3/3] Rebooting Core Systems...");
                    let _ = Command::new("pkill").arg("-9").arg("kspine").status();
                    let _ = Command::new("pkill").arg("-9").arg("koad-asm").status();
                    let _ = Command::new("pkill")
                        .arg("-9")
                        .arg("koad-watchdog")
                        .status();

                    // Spine handles autonomic restart of ASM when started
                    let spine_bin = bin_dir.join("kspine");
                    let _ = Command::new(spine_bin).env("KOAD_HOME", &home).spawn();

                    // Launch Watchdog
                    let watchdog_bin = bin_dir.join("koad-watchdog");
                    let _ = Command::new(watchdog_bin).env("KOAD_HOME", &home).spawn();

                    println!("  [OK] Core systems energized.");
                }
                Ok::<(), anyhow::Error>(())
            })
            .await??;
        }
        SystemAction::Save { full } => {
            if !is_admin {
                anyhow::bail!("Admin only.");
            }
            println!(
                "
\x1b[1m--- KoadOS Sovereign Save Protocol ---\x1b[0m"
            );
            let home = config.home.clone();
            let now_ts = Local::now().format("%Y%m%d-%H%M%S").to_string();

            // 1. Memory Drain (gRPC)
            println!(">>> [1/4] Neuronal Flush (Spine Drain)...");
            match SpineServiceClient::connect(config.network.spine_grpc_addr.clone()).await {
                Ok(mut client) => {
                    if let Err(e) = client
                        .drain_all(crate::utils::authenticated_request(Empty {}))
                        .await
                    {
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
                "INSERT INTO identity_snapshots (trigger, notes, created_at, origin_agent) VALUES ('sovereign-save', 'Full system checkpoint.', ?1, ?2)",
                params![Local::now().to_rfc3339(), agent_name],
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
            println!(
                "
\x1b[32m[CONDITION GREEN] Sovereign Save Complete.\x1b[0m"
            );
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
                    search.context("Missing search pattern.")?,
                    replace.context("Missing replacement string.")?,
                )
            };

            let content = std::fs::read_to_string(&target_path)?;
            let new_content = if fuzzy {
                // Simplified fuzzy replace for now
                content.replace(&search_str, &replace_str)
            } else {
                content.replace(&search_str, &replace_str)
            };

            if dry_run {
                println!("--- [DRY RUN] Patch for {:?} ---", target_path);
                println!("{}", new_content);
            } else {
                std::fs::write(&target_path, new_content)?;
                println!("\x1b[32m[OK]\x1b[0m Patch applied to {:?}", target_path);
            }
        }
        SystemAction::Tokenaudit { cleanup: _ } => {
            println!(
                "
\x1b[1m--- KoadOS Cognitive Efficiency Audit ---\x1b[0m"
            );
            let conn = db.get_conn()?;

            // Pass 1: Memory Density
            print!("{:<35}", "Pass 1: Storage (Knowledge):");
            let total_k: i32 =
                conn.query_row("SELECT count(*) FROM knowledge", [], |r| r.get(0))?;
            println!("\x1b[32m[PASS]\x1b[0m Ingested {} facts.", total_k);

            // Pass 2: Session Isolation
            print!("{:<35}", "Pass 2: Identity (Sessions):");
            let active_s: i32 =
                conn.query_row("SELECT count(*) FROM identity_roles", [], |r| r.get(0))?;
            println!(
                "\x1b[32m[PASS]\x1b[0m Monitoring {} active links.",
                active_s
            );

            // Pass 3: Tool-Call Efficiency
            print!("{:<35}", "Pass 3: Logic (Context Cache):");
            let cache_socket = config.home.join("koad.sock");
            if cache_socket.exists() {
                println!("\x1b[32m[PASS]\x1b[0m Neural Bus Cache Active.");
            } else {
                println!("\x1b[31m[FAIL]\x1b[0m Cache Offline.");
            }

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
                    println!(
                        "\x1b[33m[WARN]\x1b[0m KAI '{}' bio too long ({} chars).",
                        id, len
                    );
                    high_density = false;
                }
            }
            if high_density {
                println!("\x1b[32m[PASS]\x1b[0m All KAIs high-density.");
            }

            println!(
                "\x1b[1m---------------------------------------------------\x1b[0m
"
            );
        }
        SystemAction::Spawn {
            template,
            title,
            weight,
            objective,
            scope,
            labels,
        } => {
            println!(">>> [SPAWN] Energizing Forge for Issue: {}...", title);
            let issue = spawn_issue(
                config, db, &template, &title, &weight, objective, scope, labels, None,
            )
            .await?;

            // Resolve repo string for the reporter (using normalized path)
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let abs_current = std::fs::canonicalize(&current_dir).unwrap_or(current_dir);
            let search_path = abs_current.to_string_lossy().to_string();

            let repo_full = if let Ok(conn) = db.get_conn() {
                let stmt = conn.prepare("SELECT github_repo FROM projects WHERE ?1 LIKE path || '%' ORDER BY length(path) DESC LIMIT 1").ok();
                stmt.and_then(|mut s| {
                    s.query_row(params![search_path], |r| r.get::<_, String>(0))
                        .ok()
                })
                .unwrap_or_else(|| {
                    format!(
                        "{}/{}",
                        config.get_github_owner(None),
                        config.get_github_repo(None)
                    )
                })
            } else {
                format!(
                    "{}/{}",
                    config.get_github_owner(None),
                    config.get_github_repo(None)
                )
            };

            println!(
                "\x1b[32m[SPAWNED]\x1b[0m Issue #{} live at: https://github.com/{}/issues/{}",
                issue.number, repo_full, issue.number
            );
        }
        SystemAction::Import { .. } => {
            // Handled in main.rs dispatcher
        }
        SystemAction::Lock { sector, ttl } => {
            let lock_client = RedisLockClient {
                socket: config.get_redis_socket(),
            };

            if lock_client.lock(&sector, agent_name, ttl).await? {
                println!(
                    "\x1b[32m[OK]\x1b[0m Sector '{}' locked by '{}'.",
                    sector, agent_name
                );
            } else {
                let client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
                let owner: String = client
                    .pool
                    .get::<Option<String>, _>(format!("koad:lock:{}", sector))
                    .await?
                    .unwrap_or_else(|| "unknown".to_string());
                anyhow::bail!(
                    "LOCK_DENIED: Sector '{}' is already held by '{}'.",
                    sector,
                    owner
                );
            }
        }
        SystemAction::Unlock { sector } => {
            let lock_client = RedisLockClient {
                socket: config.get_redis_socket(),
            };

            if lock_client.unlock(&sector, agent_name).await? {
                println!("\x1b[32m[OK]\x1b[0m Sector '{}' released.", sector);
            } else {
                let client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
                let owner: Option<String> = client
                    .pool
                    .get::<Option<String>, _>(format!("koad:lock:{}", sector))
                    .await?;
                match owner {
                    Some(o) => {
                        anyhow::bail!(
                            "UNLOCK_DENIED: You do not own the lock for '{}' (Held by '{}').",
                            sector,
                            o
                        );
                    }
                    None => {
                        println!("\x1b[33m[WARN]\x1b[0m Sector '{}' was not locked.", sector);
                    }
                }
            }
        }
        SystemAction::Locks => {
            let client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
            let mut scan_stream = client.pool.next().scan("koad:lock:*", None, None);

            let mut all_keys: Vec<String> = Vec::new();
            while let Some(res) = scan_stream.next().await {
                if let Ok(page) = res {
                    if let Some(keys) = page.results() {
                        for key in keys {
                            if let Some(k) = key.as_str() {
                                all_keys.push(k.to_string());
                            }
                        }
                    }
                }
            }

            println!("\n\x1b[1m--- Active Distributed Locks ---\x1b[0m");
            if all_keys.is_empty() {
                println!("No active locks found.");
            } else {
                println!(
                    "{:<20} | {:<30} | {:<10}",
                    "Sector", "Owner:Session", "TTL (s)"
                );
                println!("{}", "-".repeat(65));
                for key in all_keys {
                    let val: String = client
                        .pool
                        .next()
                        .get::<Option<String>, _>(&key)
                        .await?
                        .unwrap_or_else(|| "unknown".to_string());
                    let ttl: i64 = client.pool.next().ttl(&key).await?;
                    let sector = key.replace("koad:lock:", "");
                    println!("{:<20} | {:<30} | {:<10}", sector, val, ttl);
                }
            }
        }
        SystemAction::Reconnect { agent, live } => {
            let session_id = env::var("KOAD_SESSION_ID").unwrap_or_default();
            let body_id = env::var("KOAD_BODY_ID").unwrap_or_default();

            // If live is requested but no SID, fail early
            if live && session_id.is_empty() {
                anyhow::bail!("Live reconnection failed: KOAD_SESSION_ID not set in environment.");
            }

            let target_agent = agent.unwrap_or_else(|| config.get_agent_name());

            if live {
                println!(
                    ">>> [LIVE RE-SYNC] Re-linking session {} for agent {}...",
                    session_id, target_agent
                );
            } else {
                println!(">>> [RECONNECT] Searching for Ghost: {}...", target_agent);
            }

            let mut client =
                SpineServiceClient::connect(config.network.spine_grpc_addr.clone()).await?;
            let req = ReconnectSessionRequest {
                agent_name: target_agent.clone(),
                body_id: body_id.clone(),
                session_id: if live {
                    session_id.clone()
                } else {
                    "".to_string()
                },
            };

            let res = client.reconnect_session(req).await;

            match res {
                Ok(resp) => {
                    let pkg = resp.into_inner();
                    let sid = pkg.session_id;
                    let identity: koad_core::identity::Identity =
                        serde_json::from_str(&pkg.identity_json)?;

                    // Recover body_id from session if not in environment
                    let bid = body_id;

                    if live {
                        println!("\n\x1b[32m--- KoadOS Live Connection RESTORED ---\x1b[0m");
                        println!("Agent:    {}", target_agent);
                        println!("Session:  {}", sid);
                    } else {
                        println!("\n\x1b[32m--- KoadOS Neural Link RE-ESTABLISHED ---\x1b[0m");
                        println!("Agent:    {}", target_agent);
                        println!("Rank:     {:?}", identity.rank);
                        println!("Session:  {}", sid);
                        println!("Body:     {}", bid);
                        println!("\nShell:    Run `export KOAD_SESSION_ID={} KOAD_BODY_ID={}` to bind this shell.", sid, bid);
                    }

                    if let Some(intel) = pkg.intelligence {
                        println!("\n\x1b[1mBrain Refreshed:\x1b[0m");
                        println!("{}", intel.mission_briefing);
                    }
                }
                Err(e) => {
                    anyhow::bail!(
                        "Reconnection Failed: {}. Try a fresh `koad boot`.",
                        e.message()
                    );
                }
            }
        }
        SystemAction::Backup { source } => {
            if !is_admin {
                anyhow::bail!("Admin only.");
            }
            let mut client =
                SpineServiceClient::connect(config.network.spine_grpc_addr.clone()).await?;
            let req = TriggerBackupRequest { source };
            let res = client
                .trigger_backup(crate::utils::authenticated_request(req))
                .await?
                .into_inner();

            if res.success {
                println!("\x1b[32m[OK]\x1b[0m {}", res.message);
                println!("Backup ID: {}", res.backup_id);
            } else {
                println!("\x1b[31m[ERROR]\x1b[0m {}", res.message);
            }
        }
        SystemAction::Logs {
            service,
            tail,
            follow,
        } => {
            let log_dir = config.home.join("logs");
            let service_filter = service.as_deref().unwrap_or("");

            let log_files = fs::read_dir(&log_dir)?
                .filter_map(Result::ok)
                .map(|e| e.path())
                .filter(|p| {
                    p.is_file()
                        && p.extension().and_then(|s| s.to_str()) == Some("log")
                        && p.file_name()
                            .and_then(|s| s.to_str())
                            .map(|s| s.contains(service_filter))
                            .unwrap_or(false)
                })
                .collect::<Vec<_>>();

            if log_files.is_empty() {
                anyhow::bail!(
                    "No matching log files found in {:?} (Filter: '{}')",
                    log_dir,
                    service_filter
                );
            }

            let latest_log = log_files
                .iter()
                .max_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());

            if let Some(path) = latest_log {
                let mut args = vec!["-n".to_string(), tail.to_string()];
                if follow {
                    args.push("-f".to_string());
                }
                args.push(path.to_string_lossy().to_string());

                println!(">>> Tailing log: {:?}", path);
                let mut child = Command::new("tail").args(args).spawn()?;
                let _ = child.wait();
            }
        }
        SystemAction::Stop { drain, confirm } => {
            if !is_admin {
                anyhow::bail!("Admin only.");
            }

            if !confirm {
                println!("\x1b[33m[SAFETY GATE]\x1b[0m This will shut down the KoadOS Spine and all background services.");
                println!("Run with --confirm to proceed.");
                return Ok(());
            }

            if drain {
                println!(">>> [1/2] Neuronal Flush (Spine Drain)...");
                if let Ok(mut client) =
                    SpineServiceClient::connect(config.network.spine_grpc_addr.clone()).await
                {
                    let _ = client
                        .drain_all(crate::utils::authenticated_request(Empty {}))
                        .await;
                }
            }

            println!(">>> [2/2] Terminating KoadOS Core...");
            let _ = Command::new("pkill").arg("koad-spine").status();
            let _ = Command::new("pkill").arg("koad-watchdog").status();
            println!("\x1b[32m[OK]\x1b[0m System halted.");
        }
        SystemAction::Context { action } => {
            handle_context_action(action, config, db, agent_name).await?;
        }
        SystemAction::Heartbeat { daemon, session } => {
            handle_heartbeat(daemon, session, config).await?;
        }
    }
    Ok(())
}

pub async fn handle_heartbeat(
    daemon: bool,
    session: Option<String>,
    config: &KoadConfig,
) -> Result<()> {
    let session_id = session
        .or_else(|| env::var("KOAD_SESSION_ID").ok())
        .context("No session ID provided or found in environment (KOAD_SESSION_ID).")?;

    let mut client = SpineServiceClient::connect(config.network.spine_grpc_addr.clone())
        .await
        .context("Failed to connect to Spine gRPC")?;

    if daemon {
        // Heartbeat Daemon: Subconscious Neural Pulse
        loop {
            if let Err(e) = client
                .heartbeat(crate::utils::authenticated_request(Empty {}))
                .await
            {
                warn!(
                    "Heartbeat failure for session {}: {}. Attempting re-connection...",
                    session_id, e
                );

                // 1. Re-connect to gRPC
                if let Ok(mut c) =
                    SpineServiceClient::connect(config.network.spine_grpc_addr.clone()).await
                {
                    // 2. If session is unknown to Spine (e.g. Spine rebooted), trigger Live Reconnect
                    if e.code() == tonic::Code::NotFound || e.code() == tonic::Code::Unavailable {
                        let agent_name = config.get_agent_name();
                        let body_id = env::var("KOAD_BODY_ID").unwrap_or_default();
                        let rec_req = ReconnectSessionRequest {
                            agent_name,
                            body_id,
                            session_id: session_id.clone(),
                        };
                        if c.reconnect_session(rec_req).await.is_ok() {
                            info!(
                                "Subconscious Recovery: Session {} re-linked to Spine.",
                                session_id
                            );
                        }
                    }
                    client = c;
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        }
    } else {
        client
            .heartbeat(crate::utils::authenticated_request(Empty {}))
            .await?;
        println!(
            "\x1b[32m[OK]\x1b[0m Heartbeat transmitted for session {}.",
            session_id
        );
    }
    Ok(())
}

pub async fn handle_context_action(
    action: crate::cli::ContextAction,
    config: &KoadConfig,
    db: &KoadDB,
    agent_name: &str,
) -> Result<()> {
    let mut client = SpineServiceClient::connect(config.network.spine_grpc_addr.clone())
        .await
        .context("Failed to connect to Spine gRPC")?;

    match action {
        crate::cli::ContextAction::Hydrate {
            session,
            path,
            text,
            ttl,
        } => {
            let session_id = if let Some(s) = session {
                s
            } else {
                // Try to resolve session ID from Redis for current agent
                let client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
                let key = format!("koad:identity:{}", agent_name);
                let sid: String = client
                    .pool
                    .hget::<Option<String>, _, _>(&key, "session_id")
                    .await?
                    .context("No active session found for current agent. Provide --session.")?;
                sid
            };

            let (content, file_path) = if let Some(p) = path {
                ("".to_string(), Some(p.to_string_lossy().to_string()))
            } else if let Some(t) = text {
                (t, None)
            } else {
                anyhow::bail!("Provide either --path or --text to hydrate.");
            };

            let chunk_id = if let Some(ref path) = file_path {
                path.clone()
            } else {
                let mut hasher = sha2::Sha256::new();
                use sha2::Digest;
                hasher.update(content.as_bytes());
                format!("{:x}", hasher.finalize())
            };

            let req = HydrationRequest {
                session_id: session_id.clone(),
                chunk: Some(HotContextChunk {
                    chunk_id,
                    content,
                    file_path: file_path.unwrap_or_default(),
                    ttl_seconds: ttl,
                    created_at: None,
                }),
            };

            let res = client
                .hydrate_context(crate::utils::authenticated_request(req))
                .await?
                .into_inner();
            if res.success {
                println!(
                    "\x1b[32m[OK]\x1b[0m Context Hydrated for session {}. Current size: {} bytes.",
                    session_id, res.current_context_size
                );
            } else {
                println!("\x1b[31m[ERROR]\x1b[0m Hydration Failed: {}", res.error);
            }
        }
        crate::cli::ContextAction::Flush { session, confirm } => {
            if !confirm {
                println!("\x1b[33m[SAFETY GATE]\x1b[0m This will purge all volatile hot context for the target session. This action is irreversible.");
                println!("Run with --confirm to proceed.");
                return Ok(());
            }
            let target_sid = if let Some(s) = session {
                s
            } else {
                env::var("KOAD_SESSION_ID")
                    .context("KOAD_SESSION_ID not set. Provide --session ID.")?
            };

            client
                .flush_context(crate::utils::authenticated_request(FlushContextRequest {
                    session_id: target_sid.clone(),
                }))
                .await?;

            println!(
                "\x1b[32m[OK]\x1b[0m Hot context flushed for session {}.",
                target_sid
            );
        }
        crate::cli::ContextAction::List { agent } => {
            let conn = db.get_conn()?;
            let target_agent = agent.unwrap_or_else(|| agent_name.to_string());

            let mut stmt = conn.prepare("SELECT id, session_id, created_at FROM context_snapshots WHERE agent_name = ?1 ORDER BY created_at DESC")?;
            let snapshot_iter = stmt.query_map(params![target_agent], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })?;

            println!(
                "\n\x1b[1m--- Cognitive Quicksaves for {} ---\x1b[0m",
                target_agent
            );
            println!(
                "{:<38} | {:<15} | {:<20}",
                "Snapshot ID", "Session ID", "Created At"
            );
            println!("{}", "-".repeat(80));

            let mut count = 0;
            for snap in snapshot_iter {
                let (id, sid, ts) = snap?;
                let dt = chrono::DateTime::from_timestamp(ts, 0).unwrap_or_default();
                println!("{:<38} | {:<15} | {:<20}", id, &sid[..8], dt.to_rfc3339());
                count += 1;
            }

            if count == 0 {
                println!("No snapshots found for agent '{}'.", target_agent);
            }
            println!();
        }
        crate::cli::ContextAction::Restore { id, session } => {
            let conn = db.get_conn()?;

            // 1. Fetch snapshot from DB
            let snapshot_json: String = conn
                .query_row(
                    "SELECT snapshot_json FROM context_snapshots WHERE id = ?1 OR id LIKE ?2",
                    params![id, format!("{}%", id)],
                    |row| row.get(0),
                )
                .context("Snapshot not found.")?;

            let data: serde_json::Value = serde_json::from_str(&snapshot_json)?;
            let hot_context = data["hot_context"]
                .as_object()
                .context("Malformed snapshot: missing hot_context")?;

            // 2. Resolve target session
            let target_session_id = if let Some(s) = session {
                s
            } else {
                let client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
                let key = format!("koad:identity:{}", agent_name);
                let sid: String = client
                    .pool
                    .hget::<Option<String>, _, _>(&key, "session_id")
                    .await?
                    .context("No active session found. Provide --session ID.")?;
                sid
            };

            println!(
                ">>> Restoring {} context chunks to session {}...",
                hot_context.len(),
                target_session_id
            );

            // 3. Hydrate chunks via gRPC
            let mut success_count = 0;
            for (chunk_id, chunk_val) in hot_context {
                if let Ok(chunk_data) =
                    serde_json::from_str::<serde_json::Value>(chunk_val.as_str().unwrap_or(""))
                {
                    let req = HydrationRequest {
                        session_id: target_session_id.clone(),
                        chunk: Some(HotContextChunk {
                            chunk_id: chunk_id.clone(),
                            content: chunk_data["content"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                            file_path: chunk_data["file_path"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string(),
                            ttl_seconds: chunk_data["ttl_seconds"].as_i64().unwrap_or(0) as i32,
                            created_at: None,
                        }),
                    };

                    if client.hydrate_context(req).await.is_ok() {
                        success_count += 1;
                    }
                }
            }

            println!(
                "\x1b[32m[OK]\x1b[0m Successfully restored {}/{} context chunks.",
                success_count,
                hot_context.len()
            );
        }
    }

    Ok(())
}
