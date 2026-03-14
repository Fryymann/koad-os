//! KoadOS Agent Bootstrap Tool
//!
//! This binary provides the foundational "Ghost-Body Hydration" flow. It automates
//! the discovery of agent vaults (KAPV), verifies their integrity, and generates
//! the shell environment needed for an agent session to start.
//!
//! ## Architecture
//! `koad-agent` is a standalone, minimal CLI designed to run *before* the primary
//! Citadel gRPC services are initialized. It acts as the "Bootstrap Bridge."

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use koad_core::config::KoadConfig;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;

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

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = KoadConfig::load()?;

    match cli.command {
        Commands::Boot { agent, shell } => {
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

                // --- Identity & Credentials ---
                let agent_key = agent.to_lowercase();
                if let Some(identity_config) = config.identities.get(&agent_key) {
                    println!("export KOAD_AGENT_ROLE=\"{}\";", identity_config.role);
                    println!("export KOAD_AGENT_RANK=\"{}\";", identity_config.rank);

                    if let Some(prefs) = &identity_config.preferences {
                        for key in &prefs.access_keys {
                            if let Ok(val) = std::env::var(key) {
                                println!("export {}=\"{}\";", key, val);
                            }
                        }

                        // Default GITHUB_PAT if available in access_keys
                        if prefs.access_keys.contains(&"GITHUB_ADMIN_PAT".to_string()) {
                            if let Ok(val) = std::env::var("GITHUB_ADMIN_PAT") {
                                println!("export GITHUB_PAT=\"{}\";", val);
                                println!("export GITHUB_OWNER=\"Fryymann\";");
                                println!("export GITHUB_PROJECT_NUMBER=2;");
                            }
                        }
                    }

                    // --- Generate AI Anchors ---
                    let anchor_content = format!(
                        "# KoadOS Agent Identity Anchor\n\
                         Generated At: {}\n\n\
                         ## Identity\n\
                         Name: {}\n\
                         Role: {}\n\
                         Rank: {}\n\n\
                         ## Bio\n\
                         {}\n\n\
                         ## MANDATORY: Session Hydration\n\
                         If you have not done so, or if you need to refresh your context, run:\n\
                         `eval $(koad-agent boot {})`\n\n\
                         This command injects your identity, credentials, and generates your session brief at:\n\
                         `~/.koad-os/cache/session-brief-{}.md`\n",
                        timestamp, identity_config.name, identity_config.role, identity_config.rank, identity_config.bio, agent_key, agent_key
                    );

                    let home = dirs::home_dir().unwrap_or_default();
                    let gemini_dir = home.join(".gemini");
                    if gemini_dir.exists() {
                        let _ = fs::write(gemini_dir.join("GEMINI.md"), &anchor_content).await;
                    }
                    let claude_dir = home.join(".claude");
                    if claude_dir.exists() {
                        let _ = fs::write(claude_dir.join("CLAUDE.md"), &anchor_content).await;
                    }
                }

                // --- PATH Hydration ---
                let home = dirs::home_dir().unwrap_or_default();
                let cargo_bin = home.join(".cargo/bin");
                let koad_bin = config.home.join("bin");
                if !koad_bin.exists() {
                    let _ = fs::create_dir_all(&koad_bin).await;
                }
                println!("export PATH=\"{}:{}:$PATH\";", koad_bin.display(), cargo_bin.display());

                // --- Generate Session Brief ---
                let cache_dir = config.home.join("cache");
                if !cache_dir.exists() {
                    let _ = fs::create_dir_all(&cache_dir).await;
                }
                let session_brief_path = cache_dir.join(format!("session-brief-{}.md", agent_key));

                let git_status = Command::new("git")
                    .arg("status")
                    .arg("-s")
                    .output()
                    .await
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                    .unwrap_or_else(|| "Not in a git repository or git error.".to_string());

                let git_log = Command::new("git")
                    .arg("log")
                    .arg("-n")
                    .arg("5")
                    .arg("--oneline")
                    .output()
                    .await
                    .ok()
                    .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
                    .unwrap_or_else(|| "No commits found.".to_string());

                let mut brief_content = format!("# Session Brief: {}\nGenerated At: {}\n\n## Git Status\n```\n{}\n```\n\n## Recent Commits\n```\n{}\n```\n\n", agent, timestamp, git_status.trim(), git_log.trim());

                let working_memory_path = vault_path.join("memory").join("WORKING_MEMORY.md");
                if working_memory_path.exists() {
                    if let Ok(memory_content) = fs::read_to_string(&working_memory_path).await {
                        brief_content.push_str(&format!("## Working Memory\n{}\n", memory_content));
                    }
                } else {
                    brief_content.push_str("## Working Memory\n(No WORKING_MEMORY.md found)\n");
                }

                let _ = fs::write(&session_brief_path, brief_content).await;

                // --- Shell Functions & Utilities ---
                println!("function koad-clear() {{ clear && printf '\\e[3J'; }};");
                println!(
                    "function koad-reboot() {{ \
                            local agent=$1; \
                            if [ -z \"$agent\" ]; then agent=$KOAD_AGENT_NAME; fi; \
                            unset $(env | grep KOAD_ | cut -d= -f1); \
                            eval $(koad-agent boot $agent); \
                          }};"
                );

                // Directory-aware PAT selector
                println!(
                    "function koad-auth() {{ \
                            if [[ \"$PWD\" == *\"/data/skylinks\"* ]]; then \
                                export GITHUB_PAT=\"$GITHUB_SKYLINKS_FULLACCESS_TOKEN\"; \
                                export GITHUB_OWNER=\"Skylinks-Golf\"; \
                                echo \"[AUTH] Switched to Skylinks context.\"; \
                            else \
                                export GITHUB_PAT=\"$GITHUB_ADMIN_PAT\"; \
                                export GITHUB_OWNER=\"Fryymann\"; \
                                echo \"[AUTH] Switched to Fryymann (Admin) context.\"; \
                            fi; \
                          }};"
                );

                // Refresh utility
                println!("function koad-refresh() {{ \
                            echo \"[REFRESH] Regenerating session brief...\"; \
                            eval $(koad-agent boot $KOAD_AGENT_NAME); \
                          }};");

                if agent_key == "scribe" {
                    println!("alias map='koad system map --update';");
                    println!("alias distill='koad intel query --compact';");
                } else if agent_key == "tyr" {
                    println!("alias audit='koad status --full';");
                    println!("alias canon='cat /home/ideans/.koad-os/docs/protocols/CONTRIBUTOR_CANON.md';");
                }

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
        Commands::Verify { agent } => {
            let vault_path = find_vault(&agent, &config)?;
            match verify_kapv(&vault_path) {
                Ok(_) => println!(
                    "\x1b[32m[OK]\x1b[0m Vault for agent '{}' is KAPV v1.1 compliant.",
                    agent
                ),
                Err(e) => println!("\x1b[31m[FAIL]\x1b[0m Vault integrity check failed: {}", e),
            }
        }
        Commands::Info { agent } => {
            let vault_path = find_vault(&agent, &config)?;
            let identity_path = vault_path.join("identity/IDENTITY.md");
            if identity_path.exists() {
                let content = std::fs::read_to_string(identity_path)?;
                println!("{}", content);
            } else {
                println!("Identity card not found for agent '{}'.", agent);
            }
        }
    }

    Ok(())
}

/// Resolves the physical path to an agent's personal vault.
///
/// ## Search Order
/// 1. Standard home directory: `~/.<agent>`
/// 2. Internal workspace agents: `.koad-os/.agents/.<agent>`
/// 3. Remote SLE mount: `/mnt/c/data/skylinks/.<agent>`
///
/// # Errors
/// Returns an error if the vault path cannot be found in any standard location.
fn find_vault(agent: &str, config: &KoadConfig) -> Result<PathBuf> {
    let agent_lower = agent.to_lowercase();

    let paths = [
        dirs::home_dir()
            .context("Could not resolve home dir")?
            .join(format!(".{}", &agent_lower)),
        dirs::home_dir()
            .context("Could not resolve home dir")?
            .join(&agent_lower),
        config.home.join(format!(".agents/.{}", &agent_lower)),
        PathBuf::from(format!("/mnt/c/data/skylinks/.{}", &agent_lower)),
    ];

    for path in paths {
        if path.exists() {
            return Ok(path);
        }
    }

    anyhow::bail!(
        "Vault for agent '{}' not found in any standard location.",
        agent
    )
}

/// Verifies that a directory structure complies with the KAPV v1.1 standard.
///
/// # Errors
/// Returns an error if any required KAPV directory (bank, config, identity, etc.)
/// is missing or is not a directory. Also checks for the `GEMINI.md` anchor.
fn verify_kapv(path: &Path) -> Result<()> {
    let required_dirs = [
        "bank",
        "config",
        "identity",
        "instructions",
        "memory",
        "sessions",
        "tasks",
    ];
    for dir in required_dirs {
        let p = path.join(dir);
        if !p.exists() {
            anyhow::bail!("Missing required KAPV directory: '{}' (Path: {:?})", dir, p);
        }
        if !p.is_dir() {
            anyhow::bail!("KAPV entry '{}' is not a directory (Path: {:?})", dir, p);
        }
    }

    if !path.join("GEMINI.md").exists() {
        anyhow::bail!("Missing required KAPV boot anchor: 'GEMINI.md'");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_verify_kapv_full_compliance() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();

        // Create standard structure
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
            std::fs::create_dir(path.join(d))?;
        }
        std::fs::write(path.join("GEMINI.md"), b"test")?;

        assert!(verify_kapv(path).is_ok());
        Ok(())
    }

    #[test]
    fn test_verify_kapv_missing_dir() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();

        // Missing "bank"
        std::fs::write(path.join("GEMINI.md"), b"test")?;

        assert!(verify_kapv(path).is_err());
        Ok(())
    }

    #[test]
    fn test_verify_kapv_missing_anchor() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path();

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
            std::fs::create_dir(path.join(d))?;
        }
        // Missing GEMINI.md

        assert!(verify_kapv(path).is_err());
        Ok(())
    }
}
