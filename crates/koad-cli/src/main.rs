#![allow(dead_code, unused_imports, clippy::type_complexity)]

mod cli;
mod db;
mod utils;
mod config_legacy;
mod handlers;

use anyhow::{Context, Result};
use clap::Parser;
use koad_core::config::KoadConfig;
use koad_core::logging::init_logging;
use std::env;
use std::path::PathBuf;

use crate::cli::{Cli, Commands, SystemAction};
use crate::db::KoadDB;
use crate::utils::{detect_model_tier, feature_gate, pre_flight, PreFlightStatus};
use crate::config_legacy::KoadLegacyConfig;
use crate::handlers::boot::handle_boot_command;
use crate::handlers::system::handle_system_action;
use crate::handlers::intel::handle_intel_action;
use crate::handlers::fleet::handle_fleet_action;
use crate::handlers::bridge::handle_bridge_action;
use crate::handlers::status::handle_status_command;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = KoadConfig::load()?;
    let role = cli.role.clone();
    let is_admin = role == "admin";

    // 1. Logging
    let _guard = init_logging("koad", Some(config.home.clone()));

    // 2. Legacy Config (Bio/Persona)
    let legacy_config = KoadLegacyConfig::load(&config.home.join("koad.json"))?;

    // 3. Database
    let db = KoadDB::new(&config.get_db_path())?;

    // 4. Pre-Flight
    let skip_check = cli.no_check || matches!(cli.command, Commands::Whoami | Commands::Status { .. } | Commands::Boot { .. } | Commands::System { action: SystemAction::Config { .. } });
    if !skip_check {
        match pre_flight(&config) {
            PreFlightStatus::Optimal => {}
            PreFlightStatus::Degraded(msg) => { eprintln!("\x1b[33m[DEGRADED]\x1b[0m {}", msg); }
            PreFlightStatus::Critical(msg) => {
                eprintln!("\x1b[31m[CRITICAL]\x1b[0m {}", msg);
                if !is_admin { anyhow::bail!("System critical. Non-admin access restricted."); }
            }
        }
    }

    // 5. Command Routing
    match cli.command {
        Commands::Boot { agent, project, task, compact } => {
            handle_boot_command(agent, project, task, compact, role, &config).await?;
        }
        Commands::System { action } => {
            handle_system_action(action, &config, &db, role, is_admin).await?;
        }
        Commands::Intel { action } => {
            handle_intel_action(action, &config, &db).await?;
        }
        Commands::Fleet { action } => {
            handle_fleet_action(action, &config, &db).await?;
        }
        Commands::Bridge { action } => {
            handle_bridge_action(action, &config, &db).await?;
        }
        Commands::Status { json, full } => {
            handle_status_command(json, full, &config, &db).await?;
        }
        Commands::Whoami => {
            println!("Persona: {} ({})\nBio:     {}", legacy_config.identity.name, legacy_config.identity.role, legacy_config.identity.bio);
        }
        Commands::Dash => {
            crate::handlers::status::handle_status_command(false, true, &config, &db).await?;
        }
    }

    Ok(())
}
