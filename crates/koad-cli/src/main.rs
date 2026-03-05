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

    // 4.1 Role Resolution (Auto-lookup for non-Koad agents if default is used)
    let mut role = cli.role.clone();
    if role == "admin" {
        if let Commands::Boot { ref agent, .. } = cli.command {
            if agent != "Koad" {
                if let Ok(Some(db_role)) = db.get_primary_role(agent) {
                    role = db_role;
                }
            }
        }
    }
    let is_admin = role == "admin";

    // 4.2 Path-Aware Project Context
    let mut config = config;
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    if current_dir.to_string_lossy().contains("skylinks") {
        config.github_project_number = 4;
    }

    // 5. Pre-Flight

    // 6. Command Routing
    match cli.command {
        Commands::Boot {
            agent,
            project,
            task,
            compact,
        } => {
            handle_boot_command(agent, project, task, compact, role, &config, &legacy_config)
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
        Commands::Board { action } => {
            crate::handlers::board::handle_board(action, &config).await?;
        }
        Commands::Project { action } => {
            crate::handlers::project::handle_project(action, &config).await?;
        }
        Commands::Status { json, full } => {
            handle_status_command(json, full, &config, &db).await?;
        }
        Commands::Whoami => {
            println!(
                "Persona: {} ({})\nBio:     {}",
                legacy_config.identity.name,
                legacy_config.identity.role,
                legacy_config.identity.bio
            );
        }
        Commands::Dash => {
            crate::handlers::status::handle_status_command(false, true, &config, &db).await?;
        }
    }

    Ok(())
}
