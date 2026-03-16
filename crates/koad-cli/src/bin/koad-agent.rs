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
        agent: String,
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

    if let Commands::Boot { agent, shell } = cli.command {
        let vault_path = match find_vault(&agent, &config) {
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

        if let Err(e) = verify_kapv(&vault_path) {
            if shell {
                println!(
                    "echo \"\x1b[31m[ERROR]\x1b[0m Vault verification failed for '{}': {}\";",
                    agent, e
                );
                return Ok(());
            } else {
                return Err(e);
            }
        }

        if shell {
            let now = chrono::Utc::now();
            let timestamp = now.to_rfc3339();

            let mut hasher = DefaultHasher::new();
            timestamp.hash(&mut hasher);
            let cache_hash = hasher.finish();

            println!("export KOAD_AGENT_NAME=\"{}\";", agent);
            println!("export KOAD_VAULT_PATH=\"{}\";", vault_path.display());
            println!("export KOAD_BANK_PATH=\"{}/bank\";", vault_path.display());
            println!(
                "export HISTFILE=\"{}/sessions/bash_history\";",
                vault_path.display()
            );
            println!("export TMPDIR=\"{}/bank/tmp\";", vault_path.display());
            println!("export KOAD_PROMPT_CACHE_HASH=\"{}\";", cache_hash);
            println!("export KOAD_BOOT_MODE=\"dark\";");

            let agent_key = agent.to_lowercase();
            let home = dirs::home_dir().unwrap_or_default();

            if let Some(identity_config) = config.identities.get(&agent_key) {
                println!("export KOAD_AGENT_ROLE=\"{}\";", identity_config.role);
                println!("export KOAD_AGENT_RANK=\"{}\";", identity_config.rank);

                if let Some(prefs) = &identity_config.preferences {
                    for key in &prefs.access_keys {
                        if let Ok(val) = std::env::var(key) {
                            println!("export {}=\"{}\";", key, val);
                        }
                    }
                    if prefs.access_keys.contains(&"GITHUB_ADMIN_PAT".to_string()) {
                        if let Ok(val) = std::env::var("GITHUB_ADMIN_PAT") {
                            println!("export GITHUB_PAT=\"{}\";", val);
                            println!("export GITHUB_OWNER=\"Fryymann\";");
                            println!("export GITHUB_PROJECT_NUMBER=2;");
                        }
                    }
                }

                // --- [Citadel Handshake (Phase 1)] ---
                let project_root = std::env::current_dir()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                if let Ok(mut client) =
                    CitadelSessionClient::connect(config.network.spine_grpc_addr.clone()).await
                {
                    let request = tonic::Request::new(LeaseRequest {
                        context: Some(TraceContext {
                            trace_id: format!("BOOT-{}", cache_hash),
                            origin: "Bridge".to_string(),
                            actor: agent.clone(),
                            timestamp: Some(prost_types::Timestamp {
                                seconds: now.timestamp(),
                                nanos: 0,
                            }),
                            level: WorkspaceLevel::LevelUnspecified as i32,
                        }),
                        agent_name: agent.clone(),
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
                    agent, cache_hash
                );
                println!(
                    "trap \"/home/ideans/.koad-os/scripts/koad-telemetry.sh shutdown {} {}\" EXIT;",
                    agent, cache_hash
                );

                // --- [CASS TCH Hydration (Phase 2)] ---
                let mut cass_packet = String::new();
                if let Ok(mut cass_client) =
                    HydrationServiceClient::connect(config.network.cass_grpc_addr.clone()).await
                {
                    let hydration_req = tonic::Request::new(HydrationRequest {
                        agent_name: agent.clone(),
                        project_root: project_root.clone(),
                        level: WorkspaceLevel::LevelUnspecified as i32,
                        token_budget: 4000,
                        task_id: String::new(),
                    });

                    if let Ok(hydration_res) = cass_client.hydrate(hydration_req).await {
                        cass_packet = hydration_res.into_inner().markdown_packet;
                    }
                }

                // --- AI Anchor Generation ---
                let mut anchor_content = format!(
                    "# KoadOS Agent Identity Anchor\nGenerated At: {}\n\n## Identity\nName: {}\nRole: {}\nRank: {}\n\n## Bio\n{}\n\n## MANDATORY: Session Hydration\nIf you have not done so, or if you need to refresh your context, run:\n`eval $(koad-agent boot {})`\n\n## ⚡ Efficiency Policy: The 'No-Read' Rule\nTo minimize token burn, you are STRICTLY FORBIDDEN from reading entire source files unless they are under 50 lines. \n1. **Use your Context Packet:** Structural maps of relevant crates are provided in the CASS section below. Use them first.\n2. **Discovery:** Use `grep_search` to locate specific logic or patterns.\n3. **Targeted Reading:** Use `read_file` ONLY with `start_line` and `end_line` parameters for surgical extraction.\n",
                    timestamp, identity_config.name, identity_config.role, identity_config.rank, identity_config.bio, agent_key
                );

                if !cass_packet.is_empty() {
                    anchor_content.push_str("\n## 🧠 Temporal Context Packet (CASS)\n");
                    anchor_content.push_str(&cass_packet);
                }

                let _ = fs::write(home.join(".gemini/GEMINI.md"), &anchor_content).await;
                let _ = fs::write(home.join(".claude/CLAUDE.md"), &anchor_content).await;
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
                agent,
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

            println!("function koad-refresh() {{ echo \"[REFRESH] Regenerating session brief...\"; eval $(koad-agent boot $KOAD_AGENT_NAME); }};");
            println!(
                "echo \"\x1b[1;34m--- KoadOS Session: {} ---\x1b[0m\";",
                agent
            );
            println!(
                "echo \"\x1b[32m[BOOT]\x1b[0m Neural link hydrated for agent '{}'.\";",
                agent
            );
        }
    }
    Ok(())
}

fn find_vault(agent: &str, config: &KoadConfig) -> Result<PathBuf> {
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
    anyhow::bail!("Vault for agent '{}' not found.", agent)
}

fn verify_kapv(path: &Path) -> Result<()> {
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
        if !path.join(d).is_dir() {
            anyhow::bail!("Missing KAPV dir: {}", d);
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
