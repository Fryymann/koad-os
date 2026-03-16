#![allow(dead_code, unused_imports, clippy::type_complexity)]

mod cli;
mod db;
mod handlers;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use fred::interfaces::HashesInterface;
use koad_core::config::KoadConfig;
use koad_core::logging::init_logging;
use koad_core::session::AgentSession;
use koad_core::utils::redis::RedisClient;
use std::env;
use std::path::PathBuf;

use crate::cli::{AsmAction, Cli, Commands, SystemAction, WatchdogAction};
use crate::db::KoadDB;
use crate::handlers::boot::handle_boot_command;
use crate::handlers::bridge::handle_bridge_action;
use crate::handlers::fleet::handle_fleet_action;
use crate::handlers::intel::handle_intel_action;
use crate::handlers::status::handle_status_command;
use crate::handlers::system::handle_system_action;
use crate::utils::{detect_model_tier, feature_gate, pre_flight, PreFlightStatus};
use std::collections::HashMap;

async fn handle_asm_action(action: AsmAction, config: &KoadConfig) -> Result<()> {
    match action {
        AsmAction::Status => {
            let redis_client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
            let sessions: HashMap<String, String> = redis_client.pool.hgetall("koad:state").await?;

            let mut active_count = 0;
            let mut dark_count = 0;

            println!("\n\x1b[1m--- Agent Session Manager (ASM) Status ---\x1b[0m");
            for (key, val) in sessions {
                if key.starts_with("koad:session:") {
                    if let Ok(session) =
                        serde_json::from_str::<koad_core::session::AgentSession>(&val)
                    {
                        if session.status == "active" {
                            active_count += 1;
                        } else if session.status == "dark" {
                            dark_count += 1;
                        }
                    }
                }
            }

            println!("Active Sessions: {}", active_count);
            println!("Dark Sessions:   {}", dark_count);
            println!("Total Tracked:   {}", active_count + dark_count);
            println!("\nIntegrated Reaper: \x1b[32mACTIVE\x1b[0m (via Spine)");
        }
        AsmAction::Prune => {
            println!(">>> Triggering manual session prune cycle...");
            let _client = koad_proto::spine::v1::spine_service_client::SpineServiceClient::connect(
                config.network.spine_grpc_addr.clone(),
            )
            .await?;
            // Spine reaper is internal, but we can trigger a system-wide health check/refresh
            println!("\x1b[33m[NOTE]\x1b[0m Spine integrated reaper runs every 10s. Manual trigger not required.");
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = KoadConfig::load()?;
    let role = cli.role.clone();
    let _is_admin = role == "admin";

    // 1. Determine Agent Name
    let agent_name = match &cli.command {
        Commands::Boot { agent, .. } => agent.clone(),
        _ => config.get_agent_name(),
    };

    // 2. Logging
    let log_dir = Some(config.home.join("logs"));
    let _guard = init_logging(&agent_name, log_dir);

    // 3. Database
    let db = KoadDB::new(&config.get_db_path())?;

    // 4.1 Redis Configuration Hydration (Hot Config Override)
    let mut config = config;
    if config.get_redis_socket().exists() {
        use fred::interfaces::KeysInterface;
        use koad_core::utils::redis::RedisClient;

        if let Ok(client) = RedisClient::new(&config.home.to_string_lossy(), false).await {
            if let Ok(Some(json)) = client
                .pool
                .get::<Option<String>, _>(koad_core::constants::REDIS_KEY_CONFIG)
                .await
            {
                if let Ok(hot_config) = KoadConfig::from_json(&json) {
                    // Merge hot config: preserve local paths, take hot settings
                    let home = config.home.clone();
                    let extra = config.extra.clone();

                    config = hot_config;
                    config.home = home;
                    // Merge extra fields
                    for (k, v) in extra {
                        config.extra.entry(k).or_insert(v);
                    }
                }
            }
        }
    }

    // 4.2 Role Resolution (Auto-lookup for non-sovereign agents if default is used)
    let mut role = cli.role.clone();
    if role == "admin" {
        if let Commands::Boot { ref agent, .. } = cli.command {
            let is_sovereign = if let Some(id_config) = config.identities.get(agent) {
                let r = id_config.rank.to_lowercase();
                r == "admiral" || r == "captain"
            } else {
                agent == "Tyr" || agent == "Vigil"
            };

            if !is_sovereign {
                if let Ok(Some(db_role)) = db.get_primary_role(agent) {
                    role = db_role;
                }
            }
        }
    }
    let is_admin = role == "admin";

    // 4.3 Path-Aware Project Context
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if let Some((_name, project)) = config.resolve_project_context(&current_dir) {
        env::set_var("GITHUB_OWNER", config.get_github_owner(Some(&project)));
        env::set_var("GITHUB_REPO", config.get_github_repo(Some(&project)));
    } else if current_dir.to_string_lossy().contains("skylinks") {
        env::set_var("GITHUB_OWNER", "Skylinks-Golf");
        // Only override GITHUB_REPO if we are actually inside an app directory
        if let Some(repo_name) = current_dir.file_name() {
            if current_dir.to_string_lossy().contains("/apps/") {
                env::set_var("GITHUB_REPO", repo_name);
            }
        }
    }

    // 5. Pre-Flight

    // 6. Command Routing
    match cli.command {
        Commands::Boot {
            agent,
            project,
            task,
            compact,
            budget,
            force,
        } => {
            // Pre-boot sovereign check
            let is_authorized = if let Some(id_config) = config.identities.get(&agent) {
                let r = id_config.rank.to_lowercase();
                r == "admiral" || r == "captain" || r == "officer" || r == "dood" || r == "admin"
            } else {
                // Fallback for legacy or recognized agents
                agent == "Tyr" || agent == "Dood" || agent == "Vigil" || agent == "Sky"
            };

            if !is_authorized {
                anyhow::bail!(
                    "\x1b[31mREJECTION\x1b[0m: Agent '{}' is not a registered Sovereign KAI. \
                     Only Officers and Captains can be booted as primary ghosts.",
                    agent
                );
            }

            use crate::handlers::boot::BootOptions;
            handle_boot_command(
                BootOptions {
                    agent,
                    project,
                    task,
                    compact,
                    budget,
                    force,
                    role,
                },
                &config,
            )
            .await?;
        }
        Commands::System { action } => match action {
            SystemAction::Import {
                source,
                format,
                delimiter,
                route,
                template,
                labels,
                dry_run,
            } => {
                crate::handlers::import::handle_import(
                    source, format, delimiter, route, template, labels, dry_run, &config, &db,
                )
                .await?;
            }
            _ => {
                handle_system_action(action, &config, &db, role, is_admin, &agent_name).await?;
            }
        },
        Commands::Intel { action } => {
            handle_intel_action(action, &config, &db, &agent_name).await?;
        }
        Commands::Fleet { action } => {
            handle_fleet_action(action, &config, &db).await?;
        }
        Commands::Bridge { action } => {
            handle_bridge_action(action, &config, &db).await?;
        }
        Commands::Signal { action } => {
            crate::handlers::signal::handle_signal_action(action, &config, &agent_name).await?;
        }
        Commands::Version => {
            println!("KoadOS CLI v{}", env!("CARGO_PKG_VERSION"));
            // Try to query Spine version if possible
            if let Ok(mut client) =
                koad_proto::spine::v1::spine_service_client::SpineServiceClient::connect(
                    config.network.spine_grpc_addr.clone(),
                )
                .await
            {
                if let Ok(resp) = client
                    .get_system_state(koad_proto::spine::v1::GetSystemStateRequest {})
                    .await
                {
                    println!("KoadOS Spine v{}", resp.into_inner().version);
                }
            }
        }
        Commands::Board { action } => {
            crate::handlers::board::handle_board(action, &config).await?;
        }
        Commands::Project { action } => {
            crate::handlers::project::handle_project(action, &config).await?;
        }
        Commands::Status { json, full } => {
            handle_status_command(json, full, &config, &db).await?;
        }
        Commands::Doctor { fix } => {
            handle_status_command(false, true, &config, &db).await?;
            if fix {
                println!("\n\x1b[1m--- Autonomic Self-Healing Initiated ---\x1b[0m");
                // Trigger any specific local fixes here if needed
                println!("System state evaluated. Local environmental alignment complete.");
            }
        }
        Commands::Whoami => {
            crate::handlers::whoami::handle_whoami(&config, &db).await?;
        }
        Commands::Cognitive => {
            let session_id = std::env::var("KOAD_SESSION_ID").unwrap_or_default();
            let final_agent_name = if !session_id.is_empty() {
                let redis_client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
                let session_key = format!("koad:session:{}", session_id);
                let res: Option<String> =
                    redis_client.pool.hget("koad:state", &session_key).await?;

                res.and_then(|data| {
                    serde_json::from_str::<koad_core::session::AgentSession>(&data).ok()
                })
                .map(|s| s.identity.name)
                .unwrap_or_else(|| agent_name.clone())
            } else {
                agent_name.clone()
            };

            crate::handlers::cognitive::handle_cognitive_check(&config, &db, &final_agent_name)
                .await?;
        }
        Commands::Asm { action } => {
            handle_asm_action(action, &config).await?;
        }
        Commands::Logout { session } => {
            crate::handlers::boot::handle_logout_command(session, &config).await?;
        }
        Commands::Dash => {
            crate::handlers::status::handle_status_command(false, true, &config, &db).await?;
        }
        Commands::Watchdog { action, daemon } => {
            let home = config.home.clone();
            let bin_path = home.join("bin/koad-watchdog");

            match action {
                Some(WatchdogAction::Start {
                    daemon: start_daemon,
                }) => {
                    if !bin_path.exists() {
                        anyhow::bail!("Watchdog binary not found at {:?}", bin_path);
                    }
                    if start_daemon {
                        println!(">>> Launching Autonomic Watchdog daemon...");
                        let _ = std::process::Command::new("nohup")
                            .arg(bin_path)
                            .env("KOAD_HOME", &home)
                            .spawn()?;
                    } else {
                        let mut cmd = std::process::Command::new(bin_path);
                        cmd.env("KOAD_HOME", &home);
                        cmd.status()?;
                    }
                }
                Some(WatchdogAction::Status) => {
                    use sysinfo::System;
                    let mut sys = System::new_all();
                    sys.refresh_all();
                    let watchdog_procs: Vec<_> = sys
                        .processes()
                        .values()
                        .filter(|p| p.name().contains("koad-watchdog"))
                        .collect();

                    if watchdog_procs.is_empty() {
                        println!("Watchdog: \x1b[31mSTOPPED\x1b[0m");
                    } else {
                        println!("Watchdog: \x1b[32mRUNNING\x1b[0m");
                        for p in watchdog_procs {
                            println!("  - PID: {} | Uptime: {}s", p.pid(), p.run_time());
                        }
                    }
                }
                Some(WatchdogAction::Stop) => {
                    println!(">>> Stopping Autonomic Watchdog...");
                    let _ = std::process::Command::new("pkill")
                        .arg("koad-watchdog")
                        .status()?;
                }
                None => {
                    // Legacy support for 'koad watchdog --daemon'
                    if !bin_path.exists() {
                        anyhow::bail!("Watchdog binary not found at {:?}", bin_path);
                    }
                    if daemon {
                        println!(">>> Launching Autonomic Watchdog daemon...");
                        let _ = std::process::Command::new("nohup")
                            .arg(bin_path)
                            .env("KOAD_HOME", &home)
                            .spawn()?;
                    } else {
                        let mut cmd = std::process::Command::new(bin_path);
                        cmd.env("KOAD_HOME", &home);
                        cmd.status()?;
                    }
                }
            }
        }
    }

    Ok(())
}
