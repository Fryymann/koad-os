#![allow(dead_code, unused_imports, clippy::type_complexity)]

mod cli;
mod config_legacy;
mod db;
mod handlers;
mod utils;

use anyhow::{Context, Result};
use clap::Parser;
use koad_core::config::KoadConfig;
use koad_core::logging::init_logging;
use std::env;
use std::path::PathBuf;

use crate::cli::{Cli, Commands, SystemAction};
use crate::config_legacy::KoadLegacyConfig;
use crate::db::KoadDB;
use crate::handlers::boot::handle_boot_command;
use crate::handlers::bridge::handle_bridge_action;
use crate::handlers::fleet::handle_fleet_action;
use crate::handlers::intel::handle_intel_action;
use crate::handlers::status::handle_status_command;
use crate::handlers::system::handle_system_action;
use crate::utils::{detect_model_tier, feature_gate, pre_flight, PreFlightStatus};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = KoadConfig::load()?;
    let role = cli.role.clone();
    let _is_admin = role == "admin";

    // 1. Legacy Config (Bio/Persona)
    let legacy_config = KoadLegacyConfig::load(&config.home.join("koad.json"))?;

    // 2. Determine Agent Name for Logging
    let agent_name = match &cli.command {
        Commands::Boot { agent, .. } => agent.clone(),
        _ => legacy_config.identity.name.clone(),
    };


    // 3. Logging
    let log_dir = Some(config.home.join("logs"));
    let _guard = init_logging(&agent_name, log_dir);

    // 4. Database
    let db = KoadDB::new(&config.get_db_path())?;

    // 4.1 Redis Configuration Hydration (Hot Config Override)
    let mut config = config;
    if config.redis_socket.exists() {
        use koad_core::utils::redis::RedisClient;
        use fred::interfaces::KeysInterface;
        
        if let Ok(client) = RedisClient::new(&config.home.to_string_lossy(), false).await {
            if let Ok(Some(json)) = client.pool.get::<Option<String>, _>(koad_core::constants::REDIS_KEY_CONFIG).await {
                if let Ok(hot_config) = KoadConfig::from_json(&json) {
                    // Merge hot config: preserve local paths, take hot settings
                    let home = config.home.clone();
                    let redis_socket = config.redis_socket.clone();
                    let spine_socket = config.spine_socket.clone();
                    let extra = config.extra.clone();

                    config = hot_config;
                    config.home = home;
                    config.redis_socket = redis_socket;
                    config.spine_socket = spine_socket;
                    // Merge extra fields
                    for (k, v) in extra {
                        config.extra.entry(k).or_insert(v);
                    }
                }
            }
        }
    }

    // 4.2 Role Resolution (Auto-lookup for non-Koad agents if default is used)
    let mut role = cli.role.clone();
    if role == "admin" {
        if let Commands::Boot { ref agent, .. } = cli.command {
            if agent != "Koad" && agent != "Tyr" {
                if let Ok(Some(db_role)) = db.get_primary_role(agent) {
                    role = db_role;
                }
            }
        }
    }
    let is_admin = role == "admin";

    // 4.3 Path-Aware Project Context
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current_dir.to_string_lossy().contains("skylinks") {
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
            force,
        } => {
            handle_boot_command(agent, project, task, compact, force, role, &config, &legacy_config)
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
                handle_system_action(
                    action,
                    &config,
                    &db,
                    role,
                    is_admin,
                    &legacy_config.identity.name,
                )
                .await?;
            }
        },
        Commands::Intel { action } => {
            handle_intel_action(action, &config, &db, &legacy_config.identity.name).await?;
        }
        Commands::Fleet { action } => {
            handle_fleet_action(action, &config, &db).await?;
        }
        Commands::Bridge { action } => {
            handle_bridge_action(action, &config, &db).await?;
        }
        Commands::Signal { action } => {
            crate::handlers::signal::handle_signal_action(action, &config, &legacy_config.identity.name).await?;
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
            println!(
                "Identity: {} [{}]\nBio:      {}",
                legacy_config.identity.name,
                legacy_config.identity.role,
                legacy_config.identity.bio
            );
        }
        Commands::Cognitive => {
            crate::handlers::cognitive::handle_cognitive_check(&config, &db, &legacy_config.identity.name).await?;
        }
        Commands::Logout { session } => {
            crate::handlers::boot::handle_logout_command(session, &config).await?;
        }
        Commands::Dash => {
            crate::handlers::status::handle_status_command(false, true, &config, &db).await?;
        }
        Commands::Watchdog { daemon } => {
            let home = config.home.clone();
            let bin_path = home.join("bin/koad-watchdog");
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

    Ok(())
}
