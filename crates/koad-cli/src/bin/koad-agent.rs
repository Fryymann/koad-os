//! # KoadOS Agent Bootstrap Tool
//!
//! Provides the foundational "Ghost-Body Hydration" flow for KoadOS agents.
//! This tool coordinates with the Citadel for session leasing and CASS for 
//! context hydration, ensuring a secure and informed boot process.

use anyhow::{Context, Result};
use koad::utils::errors::{map_connect_err, map_status_err};
use clap::{Parser, Subcommand};
use koad_core::config::KoadConfig;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;

use koad_proto::cass::v1::hydration_service_client::HydrationServiceClient;
use koad_proto::cass::v1::HydrationRequest;
use koad_proto::citadel::v5::citadel_session_client::CitadelSessionClient;
use koad_proto::citadel::v5::{LeaseRequest, TraceContext, WorkspaceLevel};
use std::time::Duration;
use tonic::transport::Endpoint;

#[derive(Parser)]
#[command(name = "koad-agent")]
#[command(about = "KoadOS Agent Bootstrap Tool: Neural link hydration and pre-flight.")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bootstrap an agent session with identity and environment hydration.
    Boot {
        /// The name of the agent to boot.
        #[arg(short, long)]
        agent: Option<String>,
        /// Position name of the agent to boot (legacy compatibility).
        name: Option<String>,
        /// Generate output suitable for shell evaluation.
        #[arg(short, long, default_value_t = true)]
        shell: bool,
    },
    /// Verify the integrity of an agent's personal vault (KAPV).
    Verify {
        /// The name of the agent vault to verify.
        agent: String,
    },
    /// Display summary information for an agent identity.
    Info {
        /// The name of the agent to inspect.
        agent: String,
    },
    /// Generate a high-density context packet for a crate.
    Context {
        /// Name of the crate (e.g. "koad-core", "koad-cass").
        crate_name: String,
        /// Output path (defaults to ./<crate_name>.context.md).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Validate and register a task manifest for the current agent.
    Task {
        /// Path to the task manifest (.md or .toml file).
        manifest: PathBuf,
        /// Release the current active task (mark as complete).
        #[arg(long)]
        done: bool,
    },
}

/// Timeout for boot-path gRPC connections to local Citadel/CASS services.
/// Loopback services respond in <50ms when live — 3s is generous for live mode
/// and fast enough for dark-mode degradation (~6s total vs 60s+ without timeout).
const BOOT_SERVICE_TIMEOUT: Duration = Duration::from_secs(3);

/// The main entry point for the agent bootstrap process.
/// 
/// # Errors
/// Returns an error if the configuration cannot be loaded or if any hydration step fails.
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = KoadConfig::load()?;

    match cli.command {
        Commands::Boot { agent, name, shell } => {
            let boot_start = std::time::Instant::now();
            let agent_name = agent.or(name).context("No agent name provided. Use 'koad-agent boot <name>' or 'koad-agent boot --agent <name>'.")?;
            
        let agent_key = agent_name.to_lowercase();
        let identity_config = config.identities.get(&agent_key);

        // --- [Pre-flight: Body Check] ---
        // If the agent declares a required runtime, abort before any hydration
        // unless KOAD_RUNTIME env var matches. No body = no ghost.
        if let Some(id) = identity_config {
            if let Some(required_runtime) = &id.runtime {
                let active_runtime = std::env::var("KOAD_RUNTIME").unwrap_or_default();
                if active_runtime.to_lowercase() != required_runtime.to_lowercase() {
                    eprintln!(
                        "\x1b[31m[BOOT DENIED]\x1b[0m No agent body detected for '{}'.",
                        agent_name
                    );
                    eprintln!(
                        "  Required runtime: \x1b[33m{}\x1b[0m — set KOAD_RUNTIME={} to authorize.",
                        required_runtime, required_runtime
                    );
                    std::process::exit(1);
                }
            }
        }

        let vault_uri = config.resolve_vault_uri(&agent_name)
            .context("Could not resolve vault URI for current agent.")?;
        let vault_path = match config.resolve_vault_path(&vault_uri) {
            Ok(p) => p,
            Err(e) => {
                if shell {
                    println!("echo \"\x1b[31m[ERROR]\x1b[0m {}\";", e);
                    return Ok(());
                } else {
                    return Err(e);
                }
            }
        };

            if let Err(e) = verify_kapv(&vault_path).await {
                if shell {
                    println!(
                        "echo \"\x1b[31m[ERROR]\x1b[0m Vault verification failed for '{}': {}\";",
                        agent_name, e
                    );
                    return Ok(());
                } else {
                    return Err(e);
                }
            }

            if shell {
                let mut cass_packet_size = 0;
                let mut boot_status = "OK";
                let mut cass_packet = String::new();
                let mut git_status = String::new();
                let now = chrono::Utc::now();
                let timestamp = now.to_rfc3339();

                let mut hasher = DefaultHasher::new();
                timestamp.hash(&mut hasher);
                let cache_hash = hasher.finish();

                println!("export KOADOS_HOME=\"{}\";", config.home.display());
                println!("export KOAD_AGENT_NAME=\"{}\";", agent_name);
                println!("export KOAD_VAULT_URI=\"{}\";", vault_uri);
                println!("export KOAD_VAULT_PATH=\"{}\";", vault_path.display());
                println!("export KOAD_BANK_PATH=\"{}/bank\";", vault_path.display());
                println!(
                    "export HISTFILE=\"{}/sessions/bash_history\";",
                    vault_path.display()
                );
                println!("export TMPDIR=\"{}/bank/tmp\";", vault_path.display());
                println!("export KOAD_PROMPT_CACHE_HASH=\"{}\";", cache_hash);
                println!("export KOAD_BOOT_MODE=\"dark\";");

                let agent_key = agent_name.to_lowercase();
                let home = dirs::home_dir().unwrap_or_default();

                if let Some(identity_config) = identity_config {
                    println!("export KOAD_AGENT_ROLE=\"{}\";", identity_config.role);
                    println!("export KOAD_AGENT_RANK=\"{}\";", identity_config.rank);
                    if let Some(rt) = &identity_config.runtime {
                        println!("export KOAD_RUNTIME=\"{}\";", rt);
                    }

                    if let Some(prefs) = &identity_config.preferences {
                        for key in &prefs.access_keys {
                            let resolved = config.resolve_secret(key, None);
                            if !resolved.is_empty() {
                                println!("export {}=\"{}\";", key, resolved);
                            }
                        }
                        // Export GitHub context alongside PAT for agents with GitHub access
                        if prefs.access_keys.iter().any(|k| k == "GITHUB_PAT" || k == "KOADOS_PAT_GITHUB_ADMIN") {
                            let github_owner = config.get_github_owner(None);
                            if !github_owner.is_empty() {
                                println!("export GITHUB_OWNER=\"{}\";", github_owner);
                            }
                            let github_repo = config.get_github_repo(None);
                            if !github_repo.is_empty() {
                                println!("export GITHUB_REPO=\"{}\";", github_repo);
                            }
                            println!("export GITHUB_PROJECT_NUMBER=2;");
                        }
                    }

                // --- [Parallel Phase 1: Handshakes & Data] ---
                let project_root = std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let citadel_addr = config.network.citadel_grpc_addr.clone();
                let cass_addr = config.network.cass_grpc_addr.clone();
                
                let agent_name_lease = agent_name.clone();
                let project_root_lease = project_root.clone();
                let agent_name_hydra = agent_name.clone();
                let project_root_hydra = project_root.clone();

                let lease_task = tokio::spawn(async move {
                    match Endpoint::from_shared(citadel_addr.clone())
                        .unwrap()
                        .connect_timeout(BOOT_SERVICE_TIMEOUT)
                        .timeout(BOOT_SERVICE_TIMEOUT)
                        .connect()
                        .await
                    {
                        Ok(channel) => {
                            let mut client = CitadelSessionClient::new(channel);
                            let mut request = tonic::Request::new(LeaseRequest {
                                context: Some(TraceContext {
                                    trace_id: format!("BOOT-{}", cache_hash),
                                    origin: "Bridge".to_string(),
                                    actor: agent_name_lease.clone(),
                                    timestamp: Some(prost_types::Timestamp {
                                        seconds: now.timestamp(),
                                        nanos: 0,
                                    }),
                                    level: WorkspaceLevel::LevelUnspecified as i32,
                                }),
                                agent_name: agent_name_lease.clone(),
                                project_root: project_root_lease,
                                force: true,
                                body_id: cache_hash.to_string(),
                                driver_id: "cli".to_string(),
                                metrics: None,
                            });

                            // Add mandatory Zero-Trust headers
                            request.metadata_mut().insert("x-actor", agent_name_lease.parse().unwrap());
                            request.metadata_mut().insert("x-session-id", "BOOT".parse().unwrap());
                            request.metadata_mut().insert("x-session-token", "NONE".parse().unwrap());

                            match client.create_lease(request).await {
                                Ok(resp) => Some(resp),
                                Err(e) => {
                                    eprintln!("{}", map_status_err("KoadOS Citadel", e));
                                    None
                                }
                            }
                        },
                        Err(e) => {
                            eprintln!("{}", map_connect_err("KoadOS Citadel", &citadel_addr, e));
                            None
                        }
                    }
                });

                let hydration_task = tokio::spawn(async move {
                    let cass_addr_display = cass_addr.clone();
                    match Endpoint::from_shared(cass_addr)
                        .unwrap()
                        .connect_timeout(BOOT_SERVICE_TIMEOUT)
                        .timeout(BOOT_SERVICE_TIMEOUT)
                        .connect()
                        .await
                    {
                        Ok(channel) => {
                            let mut cass_client = HydrationServiceClient::new(channel);
                            let hydration_req = tonic::Request::new(HydrationRequest {
                                agent_name: agent_name_hydra,
                                project_root: project_root_hydra,
                                level: WorkspaceLevel::LevelUnspecified as i32,
                                token_budget: 4000,
                                task_id: String::new(),
                            });
                            cass_client.hydrate(hydration_req).await.ok()
                        }
                        Err(e) => {
                            eprintln!("{}", map_connect_err("KoadOS CASS", &cass_addr_display, e));
                            None
                        }
                    }
                });

                let git_task = tokio::spawn(async move {
                    Command::new("git")
                        .arg("status")
                        .arg("-s")
                        .output()
                        .await
                        .ok()
                        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                        .unwrap_or_default()
                });

                let (lease_res, hydration_res, git_res) = tokio::join!(lease_task, hydration_task, git_task);
                
                if let Ok(Some(lease_response)) = lease_res {
                    let res = lease_response.into_inner();
                    println!("export KOAD_SESSION_ID=\"{}\";", res.session_id);
                    println!("export KOAD_SESSION_TOKEN=\"{}\";", res.token);
                }

                if let Ok(Some(h_res)) = hydration_res {
                    cass_packet = h_res.into_inner().markdown_packet;
                    cass_packet_size = cass_packet.len();
                } else {
                    boot_status = "FAIL (CASS/Hydration)";
                }

                git_status = git_res.unwrap_or_default();

                // Telemetry (Phase 0)
                println!(
                    "{}/scripts/koad-telemetry.sh boot {} {};",
                    config.home.display(), agent_name, cache_hash
                );
                println!(
                    "trap \"{}/scripts/koad-telemetry.sh shutdown {} {}\" EXIT;",
                    config.home.display(), agent_name, cache_hash
                );

                // --- AI Anchor Generation ---
                let mut anchor_content = format!(
                    "# KoadOS Agent Identity Anchor\nGenerated At: {}\n\n## Identity\nName: {}\nRole: {}\nRank: {}\n\n## Bio\n{}\n\n## MANDATORY: Session Hydration\nIf you have not done so, or if you need to refresh your context, run:\n`source ~/.koad-os/bin/koad-functions.sh && agent-boot {}`\n\n## 📂 Filesystem Protocol: Scoped MCP\nAll filesystem operations MUST be performed via the `koadFsMcp` toolset (read_text_file, write_file, list_directory, etc.). Raw shell commands for file manipulation are strictly prohibited to ensure Sanctuary compliance.\n\n## 🧭 Navigation Protocol: Game Map HUD\nUse `koad map` for instant situational awareness. \n- `koad map look` → Describe surroundings & POIs.\n- `koad map exits` → Show available paths.\n- `koad map goto <alias>` → Fast-travel to pinned locations.\n- `koad map nearby` → Scan for related configs/tasks.\n\n## ⚡ Efficiency Policy: The 'No-Read' Rule\nTo minimize token burn, you are STRICTLY FORBIDDEN from reading entire source files unless they are under 50 lines. \n1. **Use your Context Packet:** Structural maps of relevant crates are provided in the CASS section below. Use them first.\n2. **Discovery:** Use `grep_search` to locate specific logic or patterns.\n3. **Targeted Reading:** Use `read_file` ONLY with `start_line` and `end_line` parameters for surgical extraction.\n",
                    timestamp, identity_config.name, identity_config.role, identity_config.rank, identity_config.bio, agent_key
                );

                if !cass_packet.is_empty() {
                    anchor_content.push_str("\n## 🧠 Temporal Context Packet (CASS)\n");
                    anchor_content.push_str(&cass_packet);
                }

                // Batch anchor writes (Global and Local Entry Point)
                let _ = tokio::join!(
                    fs::write(home.join(".gemini/GEMINI.md"), &anchor_content),
                    fs::write(home.join(".claude/CLAUDE.md"), &anchor_content),
                    fs::write(home.join(".codex/AGENTS.md"), &anchor_content),
                    fs::write("GEMINI.md", &anchor_content),
                    fs::write("CLAUDE.md", &anchor_content),
                    fs::write("AGENTS.md", &anchor_content)
                );
            } else {
                // If NO identity config, we still need a git_status for the brief below
                let git_task = tokio::spawn(async move {
                    Command::new("git")
                        .arg("status")
                        .arg("-s")
                        .output()
                        .await
                        .ok()
                        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                        .unwrap_or_default()
                });
                git_status = git_task.await.unwrap_or_default();
            }

            // PATH Hydration
            let home = dirs::home_dir().unwrap_or_default();
            let cargo_bin = home.join(".cargo/bin");
            let koad_bin = config.home.join("bin");
            println!(
                "export PATH=\"{}:{}:$PATH\";",
                koad_bin.display(),
                cargo_bin.display()
            );

            // Session Brief
            let cache_dir = config.home.join("cache");
            let _ = fs::create_dir_all(&cache_dir).await;

            let mut brief_content = format!(
                "# Session Brief: {}\nGenerated At: {}\n\n## Git Status\n```\n{}\n```\n",
                agent_name,
                timestamp,
                git_status.trim()
            );

            // --- [ABC: Automated Boot Cognition] ---
            // Officers (3) and Captains (4) receive a tactical brief.
            // WE NOW BACKGROUND THIS to keep boot under 500ms.
            if let Some(id) = identity_config {
                if id.tier >= 3 {
                    let config_clone = config.clone();
                    let vault_clone = vault_path.clone();
                    let agent_clone = agent_name.clone();
                    let agent_key_clone = agent_key.clone();
                    let cache_dir_clone = cache_dir.clone();
                    let brief_content_clone = brief_content.clone();

                    tokio::spawn(async move {
                        if let Ok(tactical_brief) = koad::handlers::abc::run_abc(&config_clone, &vault_clone, &agent_clone).await {
                            let mut final_brief = brief_content_clone;
                            final_brief.push_str("\n## Tactical Brief (Citadel ABC)\n");
                            final_brief.push_str(&tactical_brief);
                            final_brief.push_str("\n");
                            
                            let wm_path = vault_clone.join("memory/WORKING_MEMORY.md");
                            if let Ok(mem) = fs::read_to_string(&wm_path).await {
                                final_brief.push_str("\n## Working Memory\n");
                                final_brief.push_str(&mem);
                            }
                            let _ = fs::write(
                                cache_dir_clone.join(format!("session-brief-{}.md", agent_key_clone)),
                                &final_brief,
                            ).await;
                        }
                    });
                }
            }

                let working_memory_path = vault_path.join("memory/WORKING_MEMORY.md");
                if let Ok(mem) = fs::read_to_string(&working_memory_path).await {
                    brief_content.push_str("\n## Working Memory\n");
                    brief_content.push_str(&mem);
                }
                let _ = fs::write(
                    cache_dir.join(format!("session-brief-{}.md", agent_key)),
                    &brief_content,
                )
                .await;

                let boot_duration = boot_start.elapsed();
                let metrics_content = format!(
                    "- **Hydration Time:** {:.2}ms\n- **CASS Packet Size:** {} bytes\n- **Status:** {}\n",
                    boot_duration.as_secs_f64() * 1000.0,
                    cass_packet_size,
                    boot_status
                );
                let _ = fs::write(
                    cache_dir.join(format!("boot-metrics-{}.md", agent_key)),
                    &metrics_content,
                )
                .await;

                println!("function koad-refresh() {{ echo \"[REFRESH] Regenerating session brief...\"; eval $(koad-agent boot $KOAD_AGENT_NAME); }};");
                println!(
                    "echo \"\x1b[1;34m--- KoadOS Session: {} ---\x1b[0m\";",
                    agent_name
                );
                println!(
                    "echo \"\x1b[32m[BOOT]\x1b[0m Neural link hydrated for agent '{}'.\";",
                    agent_name
                );
            }
        },
        Commands::Verify { agent } => {
            let vault_uri = config.resolve_vault_uri(&agent)
                .context("Could not resolve vault URI for current agent.")?;
            let vault_path = config.resolve_vault_path(&vault_uri)?;
            verify_kapv(&vault_path).await?;
            println!("\x1b[32m[OK]\x1b[0m Vault for '{}' is valid.", agent);
        },
        Commands::Info { agent } => {
            println!("Agent Identity: {}", agent);
            // ... add more info here if needed
        }
        Commands::Context { crate_name, output } => {
            handle_context(&config, &crate_name, output).await?;
        }
        Commands::Task { manifest, done } => {
            handle_task(&config, manifest, done).await?;
        }
    }
    Ok(())
}

async fn handle_context(config: &KoadConfig, crate_name: &str, output: Option<PathBuf>) -> Result<()> {
    use std::process::Command as StdCommand;

    let out_path = output.unwrap_or_else(|| PathBuf::from(format!("{}.context.md", crate_name)));

    // Locate crate directory
    let crate_dir = config.home.join("crates").join(crate_name);
    let crate_exists = crate_dir.exists();

    let mut packet = format!("# Context Packet: {}\nGenerated: {}\n\n", crate_name, chrono::Utc::now().format("%Y-%m-%d"));

    // 1. Crate purpose from lib.rs or main.rs doc comment
    if crate_exists {
        let candidates = ["src/lib.rs", "src/main.rs"];
        for candidate in &candidates {
            let candidate_path = crate_dir.join(candidate);
            if candidate_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&candidate_path) {
                    let doc_lines: String = content.lines()
                        .take_while(|l| l.starts_with("//!") || l.trim().is_empty())
                        .map(|l| l.trim_start_matches("//!").trim())
                        .filter(|l| !l.is_empty())
                        .take(10)
                        .collect::<Vec<_>>()
                        .join("\n");
                    if !doc_lines.is_empty() {
                        packet.push_str(&format!("## Purpose\n{}\n\n", doc_lines));
                    }
                }
                break;
            }
        }
    }

    // 2. Public symbols via koad-codegraph
    if crate_exists {
        match koad_codegraph::CodeGraph::new_with_memory() {
            Ok(graph) => {
                if let Ok(()) = graph.index_project(&crate_dir) {
                    if let Ok(summary) = graph.get_crate_summary(&crate_dir.to_string_lossy()) {
                        if !summary.is_empty() {
                            packet.push_str("## Public API\n");
                            packet.push_str(&summary);
                            packet.push_str("\n");
                        }
                    }
                }
            }
            Err(e) => {
                packet.push_str(&format!("## Public API\n_Symbol extraction unavailable: {}_\n\n", e));
            }
        }
    } else {
        packet.push_str(&format!("## Note\nCrate directory not found at `{}`. Generating from git log only.\n\n", crate_dir.display()));
    }

    // 3. Recent git history for this crate
    let git_log = StdCommand::new("git")
        .args([
            "log", "--oneline", "-10",
            "--", &format!("crates/{}/", crate_name),
        ])
        .current_dir(&config.home)
        .output();

    match git_log {
        Ok(out) if out.status.success() => {
            let log_str = String::from_utf8_lossy(&out.stdout);
            if !log_str.trim().is_empty() {
                packet.push_str("## Recent Git Activity\n```\n");
                packet.push_str(log_str.trim());
                packet.push_str("\n```\n\n");
            }
        }
        _ => {
            packet.push_str("## Recent Git Activity\n_Git log unavailable._\n\n");
        }
    }

    // 4. Cargo.toml dependencies (key deps, not all)
    let cargo_toml_path = crate_dir.join("Cargo.toml");
    if cargo_toml_path.exists() {
        if let Ok(toml_content) = std::fs::read_to_string(&cargo_toml_path) {
            let dep_lines: Vec<&str> = toml_content.lines()
                .skip_while(|l| !l.contains("[dependencies]"))
                .skip(1)
                .take_while(|l| !l.starts_with('['))
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .take(15)
                .collect();
            if !dep_lines.is_empty() {
                packet.push_str("## Key Dependencies\n```toml\n");
                packet.push_str(&dep_lines.join("\n"));
                packet.push_str("\n```\n");
            }
        }
    }

    fs::write(&out_path, &packet).await?;
    println!("\x1b[32m[OK]\x1b[0m Context packet written to: {}", out_path.display());
    println!("     Crate:  {}", crate_name);
    println!("     Size:   {} bytes", packet.len());
    Ok(())
}

async fn handle_task(config: &KoadConfig, manifest: PathBuf, done: bool) -> Result<()> {
    let run_dir = config.home.join("run");
    fs::create_dir_all(&run_dir).await?;
    let tasks_file = run_dir.join("tasks.json");

    // Load existing tasks state
    let mut tasks: serde_json::Value = if tasks_file.exists() {
        let raw = fs::read_to_string(&tasks_file).await?;
        serde_json::from_str(&raw).unwrap_or(serde_json::json!({"active": []}))
    } else {
        serde_json::json!({"active": []})
    };

    let agent_name = std::env::var("KOAD_AGENT_NAME")
        .unwrap_or_else(|_| config.get_agent_name());
    let agent_role = std::env::var("KOAD_AGENT_ROLE")
        .unwrap_or_else(|_| "unknown".to_string());
    let worktree = std::env::current_dir()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    if done {
        // Release active task for this agent
        if let Some(arr) = tasks["active"].as_array_mut() {
            arr.retain(|t| t["agent"].as_str() != Some(&agent_name));
        }
        fs::write(&tasks_file, serde_json::to_string_pretty(&tasks)?).await?;
        println!("\x1b[32m[DONE]\x1b[0m Task released for agent '{}'.", agent_name);
        return Ok(());
    }

    // Validate manifest exists
    if !manifest.exists() {
        println!("\x1b[31m[BLOCKED]\x1b[0m Manifest not found: {}", manifest.display());
        return Ok(());
    }

    // Parse manifest — extract key fields from markdown frontmatter or content
    let content = fs::read_to_string(&manifest).await?;

    // Extract task ID (look for "Task:" or filename stem)
    let task_id = manifest.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .to_string();

    // Extract required role from manifest content (look for "**Agent:**" or "## ASSIGNMENT")
    let required_agent = content.lines()
        .find(|l| l.contains("**Agent:**") || l.contains("Agent:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().trim_matches('*').trim().to_lowercase())
        .unwrap_or_default();

    // Check worktree collision: any active task in the same worktree?
    let mut blockers: Vec<String> = vec![];

    if let Some(active) = tasks["active"].as_array() {
        for t in active {
            let t_worktree = t["worktree"].as_str().unwrap_or("");
            let t_agent = t["agent"].as_str().unwrap_or("");
            let t_id = t["task_id"].as_str().unwrap_or("");

            if t_worktree == worktree && t_agent != agent_name {
                blockers.push(format!("Worktree collision: agent '{}' is working on '{}' in this worktree.", t_agent, t_id));
            }
            if t_agent == agent_name {
                blockers.push(format!("Agent '{}' already has active task '{}'. Use --done to release.", agent_name, t_id));
            }
        }
    }

    // Role check (fuzzy — check if agent name or role is in the required field)
    if !required_agent.is_empty() {
        let name_match = agent_name.to_lowercase().contains(&required_agent) || required_agent.contains(&agent_name.to_lowercase());
        let role_match = agent_role.to_lowercase().contains(&required_agent);
        if !name_match && !role_match && required_agent != "unknown" {
            blockers.push(format!("Role mismatch: manifest requires '{}', agent is '{}' ({}).", required_agent, agent_name, agent_role));
        }
    }

    // Validate referenced files exist (look for file paths in the manifest)
    let missing_files: Vec<String> = content.lines()
        .filter(|l| l.contains("src/") && (l.contains(".rs") || l.contains(".toml")))
        .filter_map(|l| {
            // Extract a path-like token
            l.split_whitespace()
                .find(|t| t.contains("src/") && t.contains(".rs"))
                .map(|t| t.trim_matches(&['`', '*', '(', ')', '[', ']', '\'', '"', ':', ',', '.'] as &[char]).to_string())
        })
        .filter(|p| {
            let full = config.home.join("crates").join(p.trim_start_matches("crates/"));
            !p.is_empty() && !full.exists()
        })
        .collect();

    if !missing_files.is_empty() {
        for f in &missing_files {
            blockers.push(format!("Referenced file not found: {}", f));
        }
    }

    if blockers.is_empty() {
        // Register active task
        let entry = serde_json::json!({
            "task_id": task_id,
            "agent": agent_name,
            "role": agent_role,
            "worktree": worktree,
            "manifest": manifest.to_string_lossy(),
            "registered_at": chrono::Utc::now().to_rfc3339(),
        });

        if let Some(arr) = tasks["active"].as_array_mut() {
            arr.push(entry);
        }
        fs::write(&tasks_file, serde_json::to_string_pretty(&tasks)?).await?;

        println!("\x1b[32m[READY]\x1b[0m Task '{}' validated and registered.", task_id);
        println!("       Agent:    {}", agent_name);
        println!("       Worktree: {}", worktree);
    } else {
        println!("\x1b[31m[BLOCKED]\x1b[0m Task '{}' cannot proceed:", task_id);
        for reason in &blockers {
            println!("  - {}", reason);
        }
    }

    Ok(())
}

async fn verify_kapv(path: &Path) -> Result<()> {
    let dirs = [
        "bank",
        "config",
        "identity",
        "instructions",
        "memory",
        "sessions",
        "tasks",
    ];
    for d in dirs {
        let dir_path = path.join(d);
        if !dir_path.exists() {
            fs::create_dir_all(&dir_path).await.context(format!("Failed to auto-heal missing KAPV dir: {}", d))?;
        } else if !dir_path.is_dir() {
            anyhow::bail!("KAPV entry '{}' exists but is not a directory.", d);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vault_dirs_exist() {
        // Placeholder for KAPV logic tests
    }
}
// rebuild
