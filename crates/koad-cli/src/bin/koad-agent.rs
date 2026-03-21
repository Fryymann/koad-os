//! # KoadOS Agent Bootstrap Tool
//!
//! Provides the foundational "Ghost-Body Hydration" flow for KoadOS agents.
//! This tool coordinates with the Citadel for session leasing and CASS for 
//! context hydration, ensuring a secure and informed boot process.

use anyhow::{Context, Result};
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
}

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

        let vault_path = match find_vault(&agent_name, &config, identity_config) {
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
                let now = chrono::Utc::now();
                let timestamp = now.to_rfc3339();

                let mut hasher = DefaultHasher::new();
                timestamp.hash(&mut hasher);
                let cache_hash = hasher.finish();

                println!("export KOAD_AGENT_NAME=\"{}\";", agent_name);
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

                    if let Some(prefs) = &identity_config.preferences {
                        for key in &prefs.access_keys {
                            if let Ok(val) = std::env::var(key) {
                                println!("export {}=\"{}\";", key, val);
                            }
                        }
                        if prefs.access_keys.contains(&"KOADOS_PAT_GITHUB_ADMIN".to_string()) {
                            if let Ok(val) = std::env::var("KOADOS_PAT_GITHUB_ADMIN") {
                                println!("export GITHUB_PAT=\"{}\";", val);
                                let github_user = std::env::var("KOADOS_MAIN_GITHUB_USER")
                                    .unwrap_or_else(|_| config.get_github_owner(None));
                                println!("export GITHUB_OWNER=\"{}\";", github_user);
                                println!("export GITHUB_PROJECT_NUMBER=2;");
                            }
                        }
                    }

                    // --- [Citadel Handshake (Phase 1)] ---
                    let project_root = std::env::current_dir()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    if let Ok(client) =
                        CitadelSessionClient::connect(config.network.citadel_grpc_addr.clone()).await
                    {
                        let mut client: CitadelSessionClient<tonic::transport::Channel> = client;
                        let request = tonic::Request::new(LeaseRequest {
                            context: Some(TraceContext {
                                trace_id: format!("BOOT-{}", cache_hash),
                                origin: "Bridge".to_string(),
                                actor: agent_name.clone(),
                                timestamp: Some(prost_types::Timestamp {
                                    seconds: now.timestamp(),
                                    nanos: 0,
                                }),
                                level: WorkspaceLevel::LevelUnspecified as i32,
                            }),
                            agent_name: agent_name.clone(),
                            project_root: project_root.clone(),
                            force: true,
                            body_id: cache_hash.to_string(),
                            driver_id: "cli".to_string(),
                            metrics: None,
                        });

                        if let Ok(response) = client.create_lease(request).await {
                            let res = response.into_inner();
                            println!("export KOAD_SESSION_ID=\"{}\";", res.session_id);
                            println!("export KOAD_SESSION_TOKEN=\"{}\";", res.token);
                        }
                    }

                    // Telemetry (Phase 0)
                    println!(
                        "/home/ideans/.koad-os/scripts/koad-telemetry.sh boot {} {};",
                        agent_name, cache_hash
                    );
                    println!(
                        "trap \"/home/ideans/.koad-os/scripts/koad-telemetry.sh shutdown {} {}\" EXIT;",
                        agent_name, cache_hash
                    );

                    // --- [CASS TCH Hydration (Phase 2)] ---
                    let mut cass_packet = String::new();
                    match HydrationServiceClient::connect(config.network.cass_grpc_addr.clone()).await {
                        Ok(mut cass_client) => {
                            let hydration_req = tonic::Request::new(HydrationRequest {
                                agent_name: agent_name.clone(),
                                project_root: project_root.clone(),
                                level: WorkspaceLevel::LevelUnspecified as i32,
                                token_budget: 4000,
                                task_id: String::new(),
                            });

                            match cass_client.hydrate(hydration_req).await {
                                Ok(hydration_res) => {
                                    cass_packet = hydration_res.into_inner().markdown_packet;
                                    cass_packet_size = cass_packet.len();
                                }
                                Err(_) => boot_status = "FAIL (Hydration)",
                            }
                        }
                        Err(_) => boot_status = "FAIL (CASS Connection)",
                    }

                    // --- AI Anchor Generation ---
                    let mut anchor_content = format!(
                        "# KoadOS Agent Identity Anchor\nGenerated At: {}\n\n## Identity\nName: {}\nRole: {}\nRank: {}\n\n## Bio\n{}\n\n## MANDATORY: Session Hydration\nIf you have not done so, or if you need to refresh your context, run:\n`agent-boot {}`\n\n## 📂 Filesystem Protocol: Scoped MCP\nAll filesystem operations MUST be performed via the `koadFsMcp` toolset (read_text_file, write_file, list_directory, etc.). Raw shell commands for file manipulation are strictly prohibited to ensure Sanctuary compliance.\n\n## 🧭 Navigation Protocol: Game Map HUD\nUse `koad map` for instant situational awareness. \n- `koad map look` → Describe surroundings & POIs.\n- `koad map exits` → Show available paths.\n- `koad map goto <alias>` → Fast-travel to pinned locations.\n- `koad map nearby` → Scan for related configs/tasks.\n\n## ⚡ Efficiency Policy: The 'No-Read' Rule\nTo minimize token burn, you are STRICTLY FORBIDDEN from reading entire source files unless they are under 50 lines. \n1. **Use your Context Packet:** Structural maps of relevant crates are provided in the CASS section below. Use them first.\n2. **Discovery:** Use `grep_search` to locate specific logic or patterns.\n3. **Targeted Reading:** Use `read_file` ONLY with `start_line` and `end_line` parameters for surgical extraction.\n",
                        timestamp, identity_config.name, identity_config.role, identity_config.rank, identity_config.bio, agent_key
                    );

                    if !cass_packet.is_empty() {
                        anchor_content.push_str("\n## 🧠 Temporal Context Packet (CASS)\n");
                        anchor_content.push_str(&cass_packet);
                    }

                    let _ = fs::write(home.join(".gemini/GEMINI.md"), &anchor_content).await;
                    let _ = fs::write(home.join(".claude/CLAUDE.md"), &anchor_content).await;
                    let _ = fs::write(home.join(".codex/AGENTS.md"), &anchor_content).await;
                }

                // PATH Hydration
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

                let git_status = Command::new("git")
                    .arg("status")
                    .arg("-s")
                    .output()
                    .await
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                    .unwrap_or_default();

                let mut brief_content = format!(
                    "# Session Brief: {}\nGenerated At: {}\n\n## Git Status\n```\n{}\n```\n",
                    agent_name,
                    timestamp,
                    git_status.trim()
                );
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
            let agent_key = agent.to_lowercase();
            let identity_config = config.identities.get(&agent_key);
            let vault_path = find_vault(&agent, &config, identity_config)?;
            verify_kapv(&vault_path).await?;
            println!("\x1b[32m[OK]\x1b[0m Vault for '{}' is valid.", agent);
        },
        Commands::Info { agent } => {
            println!("Agent Identity: {}", agent);
            // ... add more info here if needed
        }
    }
    Ok(())
}

fn find_vault(agent: &str, config: &KoadConfig, identity: Option<&koad_core::config::AgentIdentityConfig>) -> Result<PathBuf> {
    // 1. Check if an explicit vault path is configured
    if let Some(id_config) = identity {
        if let Some(v_path) = &id_config.vault {
            let expanded = if v_path.starts_with("~") {
                let home = dirs::home_dir().context("Could not find home directory for tilde expansion.")?;
                PathBuf::from(v_path.replacen("~", &home.to_string_lossy(), 1))
            } else {
                PathBuf::from(v_path)
            };
            if expanded.exists() {
                return Ok(expanded);
            }
        }
    }

    // 2. Fallback to discovery paths
    let agent_lower = agent.to_lowercase();
    let paths = [
        dirs::home_dir()
            .context("No home")?
            .join(format!(".{}", &agent_lower)),
        config.home.join(format!(".agents/.{}", &agent_lower)),
    ];
    for path in paths {
        if path.exists() {
            return Ok(path);
        }
    }
    anyhow::bail!("Vault for agent '{}' not found. Please configure 'vault' in your identity TOML or ensure it exists at a standard path.", agent)
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
