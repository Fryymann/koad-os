use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::process::Command;
use std::time::Duration;

use koad_proto::citadel::v5::citadel_session_client::CitadelSessionClient;
use koad_proto::citadel::v5::{LeaseRequest, TraceContext, WorkspaceLevel};
use tonic::transport::Endpoint;

use crate::commands::self_anchor::{
    fetch_cass_packet, latest_journal, log_cass_miss, read_self, render_self_section,
    CASS_CONNECT_TIMEOUT, CASS_HYDRATE_TIMEOUT,
};
use crate::commands::verify::verify_kapv;

/// Timeout for boot-path gRPC connections to local Citadel/CASS services.
const BOOT_SERVICE_TIMEOUT: Duration = Duration::from_secs(3);

pub async fn handle_boot(
    config: &KoadConfig,
    agent: Option<String>,
    name: Option<String>,
    shell: bool,
) -> Result<()> {
    let boot_start = std::time::Instant::now();
    let agent_name = agent.or(name).context("No agent name provided. Use 'koad-agent boot <name>' or 'koad-agent boot --agent <name>'.")?;

    let agent_key = agent_name.to_lowercase();
    let identity_config = config.identities.get(&agent_key);

    // --- [Pre-flight: Body Check] ---
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

    let vault_uri = config
        .resolve_vault_uri(&agent_name)
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
        let git_status;
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
        println!("export CASS_GRPC_ADDR=\"{}\";", config.network.cass_grpc_addr);

        let agent_key = agent_name.to_lowercase();

        // WSL GPU/CUDA path fix
        if Path::new("/usr/lib/wsl/lib").exists() {
            println!(
                "export LD_LIBRARY_PATH=\"/usr/lib/wsl/lib${{LD_LIBRARY_PATH:+:${{LD_LIBRARY_PATH}}}}\";"
            );
        }

        // Fast Display: Show the last known state immediately
        let cache_dir = config.home.join("cache");
        let brief_cache = cache_dir.join(format!("session-brief-{}.md", agent_key));
        if brief_cache.exists() {
            println!(
                "echo -e \"\\x1b[1;30m[QUICK-RESTORE] Loading last cached brief...\\x1b[0m\";"
            );
            println!("cat \"{}\";", brief_cache.display());
            println!(
                "echo -e \"\\x1b[1;30m-------------------------------------------\\x1b[0m\";"
            );
        }
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
                
                if prefs
                    .access_keys
                    .iter()
                    .any(|k| k == "GITHUB_PAT" || k == "KOADOS_PAT_GITHUB_ADMIN")
                {
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
                        request
                            .metadata_mut()
                            .insert("x-actor", agent_name_lease.parse().unwrap());
                        request
                            .metadata_mut()
                            .insert("x-session-id", "BOOT".parse().unwrap());
                        request
                            .metadata_mut()
                            .insert("x-session-token", "NONE".parse().unwrap());

                        match client.create_lease(request).await {
                            Ok(resp) => Some(resp),
                            Err(e) => {
                                eprintln!("{}", koad_core::utils::errors::map_status_err("KoadOS Citadel", e));
                                None
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "{}",
                            koad_core::utils::errors::map_connect_err("KoadOS Citadel", &citadel_addr, e)
                        );
                        None
                    }
                }
            });

            // Hydrate gets its own budget: with BOOT_SERVICE_TIMEOUT on the
            // request too, a healthy but busy CASS was reported as failed.
            let hydration_task = tokio::spawn(async move {
                fetch_cass_packet(
                    &cass_addr,
                    &agent_name_hydra,
                    &project_root_hydra,
                    CASS_CONNECT_TIMEOUT,
                    CASS_HYDRATE_TIMEOUT,
                )
                .await
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

            let pulse_task = {
                let cass_addr = config.network.cass_grpc_addr.clone();
                let agent_name = agent_name.clone();
                let hasher_hash = cache_hash;
                tokio::spawn(async move {
                    match Endpoint::from_shared(cass_addr)
                        .unwrap()
                        .connect_timeout(BOOT_SERVICE_TIMEOUT)
                        .timeout(BOOT_SERVICE_TIMEOUT)
                        .connect()
                        .await
                    {
                        Ok(channel) => {
                            let mut client = koad_proto::cass::v1::pulse_service_client::PulseServiceClient::new(channel);
                            let req = koad_proto::cass::v1::GetPulsesRequest {
                                context: Some(koad_proto::citadel::v5::TraceContext {
                                    trace_id: format!("BOOT-PULSE-{}", hasher_hash),
                                    origin: "Bridge".to_string(),
                                    actor: agent_name,
                                    timestamp: None,
                                    level: koad_proto::citadel::v5::WorkspaceLevel::LevelUnspecified as i32,
                                }),
                                role: "global".to_string(),
                            };
                            client.get_pulses(req).await.ok().map(|r| r.into_inner().pulses)
                        }
                        Err(_) => None,
                    }
                })
            };

            let (lease_res, hydration_res, git_res, pulse_res) =
                tokio::join!(lease_task, hydration_task, git_task, pulse_task);

            if let Ok(Some(lease_response)) = lease_res {
                let res = lease_response.into_inner();
                println!("export KOAD_SESSION_ID=\"{}\";", res.session_id);
                println!("export KOAD_SESSION_TOKEN=\"{}\";", res.token);
            }

            let mut cass_miss = None;
            match hydration_res {
                Ok(Ok(packet)) => {
                    cass_packet = packet;
                    cass_packet_size = cass_packet.len();
                }
                Ok(Err(miss)) => {
                    let line = miss.log_line(&config.network.cass_grpc_addr);
                    eprintln!("koad-agent boot: {line}");
                    log_cass_miss(&config.home, &agent_key, "wsl", &line);
                    cass_miss = Some(miss);
                    boot_status = "FAIL (CASS/Hydration)";
                }
                Err(_) => boot_status = "FAIL (CASS/Hydration)",
            }

            let active_pulses = pulse_res.unwrap_or_default().unwrap_or_default();

            git_status = git_res.unwrap_or_default();

            // Telemetry (Phase 0)
            println!(
                "{}/scripts/koad-telemetry.sh boot {} {};",
                config.home.display(),
                agent_name,
                cache_hash
            );
            // Only set EXIT trap if not running in an agent harness (which intercepts/uses EXIT traps to preserve env)
            if std::env::var("ANTIGRAVITY_AGENT").is_err() && std::env::var("CLAUDE_CODE_ENTRYPOINT").is_err() {
                println!(
                    "trap \"{}/scripts/koad-telemetry.sh shutdown {} {}\" EXIT;",
                    config.home.display(),
                    agent_name,
                    cache_hash
                );
            }


            // --- AI Anchor Generation ---
            let mut anchor_content = format!(
            "# KoadOS Agent Identity Anchor\nGenerated At: {}\n\n## Identity\nName: {}\nRole: {}\nRank: {}\n\n## Bio\n{}\n\n## Session\nIn an agent harness, run the `agent-boot` skill: it mints a Citadel session and saves it to `$KOAD_VAULT_PATH/sessions/current.env` for later commands. In an interactive terminal: `source {}/bin/koad-functions.sh && agent-boot {}`\n",
            timestamp, identity_config.name, identity_config.role, identity_config.rank, identity_config.bio, config.home.display(), agent_key
        );

            // Who I am comes from the vault, so it survives a CASS outage.
            let journal = latest_journal(&vault_path).map(|p| p.display().to_string());
            anchor_content.push_str(&render_self_section(
                read_self(&vault_path).as_deref(),
                journal.as_deref(),
            ));

            // --- [AIS: Live Awareness Section] ---
            if !active_pulses.is_empty() {
                anchor_content.push_str("\n## 🛜 Live Awareness (Global Pulses)\n");
                for p in active_pulses {
                    anchor_content.push_str(&pulse_line(&p.author, &p.message, &p.role));
                }
            }

            anchor_content.push_str(operating_guidance());

            if let Some(miss) = &cass_miss {
                anchor_content.push('\n');
                anchor_content.push_str(&miss.anchor_line());
                anchor_content.push('\n');
            } else if !cass_packet.is_empty() {
                anchor_content.push_str("\n## 🧠 Temporal Context Packet (CASS)\n");
                anchor_content.push_str(&cass_packet);
            }

            // Resolve bootstrap path for the target agent
            let bootstrap_path = identity_config.bootstrap.as_ref().map(|b| {
                let path_str = if b.starts_with('~') {
                    b.replacen('~', &home.to_string_lossy(), 1)
                } else {
                    b.clone()
                };
                PathBuf::from(path_str)
            }).unwrap_or_else(|| {
                vault_path.join("GEMINI.md") // Default fallback inside the vault
            });

            // Ensure parent directory of bootstrap path exists
            if let Some(parent) = bootstrap_path.parent() {
                let _ = fs::create_dir_all(parent).await;
            }

            // Always write the full anchor to the agent's specific bootstrap path
            let _ = fs::write(bootstrap_path.clone(), &anchor_content).await;

            let is_gemini = identity_config.runtime.as_deref() == Some("gemini")
                || std::env::var("KOAD_RUNTIME").as_deref() == Ok("gemini");

            let is_claude = identity_config.runtime.as_deref() == Some("claude")
                || std::env::var("KOAD_RUNTIME").as_deref() == Ok("claude");

            if is_gemini {
                let router_content = format!(
                    "# KoadOS Agent Identity Router\n\
                     Generated At: {}\n\n\
                     ## Active Session Routing\n\
                     - **Active Agent:** {}\n\
                     - **Role:** {}\n\
                     - **Rank:** {}\n\
                     - **Agent Profile Vault (KAPV):** [vault](file://{})\n\
                     - **Bootstrap Anchor:** [bootstrap](file://{})\n\n\
                     ## Instructions\n\
                     You are routing to the active agent profile vault (KAPV). Please IMMEDIATELY load your full identity, system protocols, and temporal context from the bootstrap anchor file:\n\
                     [bootstrap](file://{})\n\n\
                     Follow all instructions, protocols, and context files located within your KAPV.\n\n\
                     ## MANDATORY: Session Hydration\n\
                     If you have not done so, or if you need to refresh your context, run:\n\
                     `source {}/bin/koad-functions.sh && agent-boot`\n",
                    timestamp,
                    identity_config.name,
                    identity_config.role,
                    identity_config.rank,
                    vault_path.display(),
                    bootstrap_path.display(),
                    bootstrap_path.display(),
                    config.home.display()
                );
                let _ = write_identity_anchor(
                    &home,
                    AnchorRuntime::Gemini,
                    &router_content,
                    &identity_config.name,
                )
                .await;
            } else if is_claude {
                let _ = write_identity_anchor(
                    &home,
                    AnchorRuntime::Claude,
                    &anchor_content,
                    &identity_config.name,
                )
                .await;
            } else {
                let _ = write_identity_anchor(
                    &home,
                    AnchorRuntime::Codex,
                    &anchor_content,
                    &identity_config.name,
                )
                .await;
            }
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
        // We now use a centralized SITREP.md at the workspace root instead of isolated
        // working memory files. This ensures global consistency across the crew.
        if let Ok(sitrep) = std::fs::read_to_string("SITREP.md") {
            brief_content.push_str("\n## Tactical Brief (Citadel SITREP)\n");
            brief_content.push_str(&sitrep);
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
    Ok(())
}

/// Safely writes an identity anchor file, ensuring it doesn't overwrite
/// an active anchor for a DIFFERENT agent unless it is stale (>10m).
/// Harness whose instruction file carries the generated identity anchor.
#[derive(Debug, Clone, Copy)]
enum AnchorRuntime {
    Gemini,
    Claude,
    Codex,
}

impl AnchorRuntime {
    /// User-level instruction file for this harness, relative to `$HOME`.
    fn home_relative_path(self) -> &'static str {
        match self {
            AnchorRuntime::Gemini => ".gemini/GEMINI.md",
            AnchorRuntime::Claude => ".claude/CLAUDE.md",
            AnchorRuntime::Codex => ".codex/AGENTS.md",
        }
    }

    /// Project-level file name the harness would also read; tests use it to
    /// prove the project's copy is never touched.
    #[cfg(test)]
    fn file_name(self) -> &'static str {
        match self {
            AnchorRuntime::Gemini => "GEMINI.md",
            AnchorRuntime::Claude => "CLAUDE.md",
            AnchorRuntime::Codex => "AGENTS.md",
        }
    }
}

/// Write the identity anchor to the harness's user-level instruction file.
///
/// Never writes to the current directory: CLAUDE.md, AGENTS.md and GEMINI.md
/// there belong to the project being worked on, and the user-level file
/// already reaches the harness in every directory.
async fn write_identity_anchor(
    home: &Path,
    runtime: AnchorRuntime,
    content: &str,
    agent_name: &str,
) -> Result<()> {
    safe_write_anchor(home.join(runtime.home_relative_path()), content, agent_name).await
}

/// Operating guidance written into every identity anchor.
fn operating_guidance() -> &'static str {
    "\n## Working Environment\n\
     - **Files:** use your harness's own file tools. Models without file tools (local or \
     harness-less) can use `koad-fs-mcp`, a filesystem MCP server scoped to this agent's \
     allowed directories.\n\
     - **Memory:** recall prior work from CASS before rebuilding knowledge (`citadel-memory` \
     MCP or `koad intel query`); store durable lessons with `koad intel remember`.\n\
     - **Handoffs:** messages between agents are files in \
     `$KOAD_HOME/agents/inbox/<slug>.<type>.<agent>.md`.\n\
     - **Orientation (optional):** `koad map look` summarises the current directory.\n"
}

/// One Global Pulse as a Markdown list item.
fn pulse_line(author: &str, message: &str, role: &str) -> String {
    format!("- **{author}**: {message} ({role})\n")
}

async fn safe_write_anchor(path: PathBuf, content: &str, agent_name: &str) -> Result<()> {
    if path.exists() {
        if let Ok(existing) = fs::read_to_string(&path).await {
            // Check if the current agent owns this file
            if existing.contains(&format!("Name: {}", agent_name)) {
                // Same agent, update allowed
                fs::write(&path, content).await?;
                return Ok(());
            }

            // Check age for staleness
            if let Ok(meta) = fs::metadata(&path).await {
                if let Ok(modified) = meta.modified() {
                    let age = std::time::SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or_default();
                    if age < std::time::Duration::from_secs(600) {
                        // File is fresh and owned by another agent, skip.
                        return Ok(());
                    }
                }
            }
        }
    }

    fs::write(&path, content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression guard: the anchor mandated a filesystem MCP that did not
    /// exist, forbade reading whole files, and named Gemini CLI tools.
    #[test]
    fn guidance_matches_current_harnesses() {
        let g = operating_guidance();
        for stale in [
            "MUST be performed via",
            "No-Read",
            "STRICTLY FORBIDDEN",
            "grep_search",
            "read_file",
        ] {
            assert!(!g.contains(stale), "stale guidance still present: {stale}");
        }
        assert!(
            g.contains("koad-fs-mcp"),
            "local models should be pointed at koad-fs-mcp"
        );
    }

    /// Regression guard: pulses were written with terminal escape codes into
    /// a Markdown file.
    #[test]
    fn pulse_line_is_plain_markdown() {
        let line = pulse_line("hermes", "deploy done", "global");
        assert!(!line.contains('\x1b'), "{line:?}");
        assert_eq!(line, "- **hermes**: deploy done (global)\n");
    }

    /// Regression guard: boot used to write the anchor to CLAUDE.md, AGENTS.md
    /// and GEMINI.md in the current directory, clobbering a project's own
    /// instruction files (survival-game on 2026-09-26, skylinks on 2026-05-02).
    /// The anchor must only go to the user-level file under `$HOME`.
    ///
    /// This is the only test in the crate that changes the working directory.
    #[tokio::test]
    async fn anchor_is_written_to_home_and_never_to_the_project() {
        let home = tempfile::tempdir().expect("home tempdir");
        let project = tempfile::tempdir().expect("project tempdir");
        let original_cwd = std::env::current_dir().expect("cwd");

        let runtimes = [
            AnchorRuntime::Gemini,
            AnchorRuntime::Claude,
            AnchorRuntime::Codex,
        ];
        for runtime in runtimes {
            let project_file = project.path().join(runtime.file_name());
            std::fs::write(&project_file, "project instructions").expect("seed project file");
            std::fs::create_dir_all(
                home.path()
                    .join(runtime.home_relative_path())
                    .parent()
                    .unwrap(),
            )
            .expect("home harness dir");
        }

        std::env::set_current_dir(project.path()).expect("enter project");
        for runtime in runtimes {
            let result = write_identity_anchor(home.path(), runtime, "ANCHOR", "Clyde").await;
            assert!(result.is_ok(), "{runtime:?}: {result:?}");
        }
        std::env::set_current_dir(&original_cwd).expect("restore cwd");

        for runtime in runtimes {
            let project_file = project.path().join(runtime.file_name());
            assert_eq!(
                std::fs::read_to_string(&project_file).unwrap(),
                "project instructions",
                "{runtime:?} clobbered the project's {}",
                runtime.file_name()
            );
            let home_file = home.path().join(runtime.home_relative_path());
            assert_eq!(
                std::fs::read_to_string(&home_file).unwrap(),
                "ANCHOR",
                "{runtime:?}"
            );
        }
    }
}
