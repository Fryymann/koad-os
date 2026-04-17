//! # KoadOS Deployment & Scaffolding
//!
//! Implements `koad deploy station` and `koad deploy outpost` for
//! initializing hierarchical workspace structures.

use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use crate::cli::DeployAction;

pub async fn handle_deploy_action(action: DeployAction, config: &KoadConfig) -> Result<()> {
    match action {
        DeployAction::Station { name } => handle_deploy_station(&name, config).await,
        DeployAction::Outpost { name } => handle_deploy_outpost(&name, config).await,
    }
}

async fn handle_deploy_station(name: &str, config: &KoadConfig) -> Result<()> {
    let cwd = env::current_dir()?;
    println!(
        "\n\x1b[1;34m--- Deploying KoadOS Station: {} ---\x1b[0m",
        name
    );
    println!("  Target: {}\n", cwd.display());

    // 1. Base Directory Structure
    let dirs = ["data", "docs", "config", "logs", "updates"];
    for d in &dirs {
        let p = cwd.join(d);
        if !p.exists() {
            fs::create_dir_all(&p)?;
            println!("  \x1b[32m[CREATE]\x1b[0m {}/", d);
        }
    }

    // 2. Hidden Support Directory (.koados-station)
    let station_dir = cwd.join(".koados-station");
    let support_dirs = ["agents", "quests", "cache"];
    for d in &support_dirs {
        let p = station_dir.join(d);
        if !p.exists() {
            fs::create_dir_all(&p)?;
            println!("  \x1b[32m[CREATE]\x1b[0m .koados-station/{}/", d);
        }
        // Create .gitkeep to track directory structure
        fs::File::create(p.join(".gitkeep"))?;
    }

    // 3. STATION.md
    let station_md = format!(
        "# Station: {name}\n\n\
         **Role:** Project Hub / Service Node\n\
         **Citadel:** {citadel_home}\n\n\
         ## Overview\n\n\
         This directory is a KoadOS Station. It serves as a central hub for related outposts\n\
         and provides shared context for agents operating within this sector.\n\n\
         ## Sectors\n\n\
         - `data/` — Shared station-level datasets\n\
         - `docs/` — Canonical specs and architectural guides\n\
         - `config/` — Station-level configuration and identity overrides\n\
         - `updates/` — Chronological event stream for this station\n\
         - `.koados-station/` — Station-specific agent context and quests (Local Only)\n",
        name = name,
        citadel_home = config.home.display()
    );
    write_file(&cwd, "STATION.md", &station_md)?;

    // 4. .koad-os symlink
    create_citadel_link(&cwd, &config.home)?;

    println!(
        "\n\x1b[1;32m[SUCCESS]\x1b[0m Station '{}' deployed successfully.",
        name
    );
    println!("Next: Register this station in your Citadel with 'koad project register'.");

    Ok(())
}

async fn handle_deploy_outpost(name: &str, config: &KoadConfig) -> Result<()> {
    let cwd = env::current_dir()?;
    println!(
        "\n\x1b[1;34m--- Deploying KoadOS Outpost: {} ---\x1b[0m",
        name
    );
    println!("  Target: {}\n", cwd.display());

    // 1. Base Directory Structure
    let dirs = ["docs", "config", "updates"];
    for d in &dirs {
        let p = cwd.join(d);
        if !p.exists() {
            fs::create_dir_all(&p)?;
            println!("  \x1b[32m[CREATE]\x1b[0m {}/", d);
        }
    }

    // 2. Hidden Support Directory (.koados-outpost)
    let outpost_dir = cwd.join(".koados-outpost");
    let support_dirs = ["agents", "quests", "cache"];
    for d in &support_dirs {
        let p = outpost_dir.join(d);
        if !p.exists() {
            fs::create_dir_all(&p)?;
            println!("  \x1b[32m[CREATE]\x1b[0m .koados-outpost/{}/", d);
        }
        // Create .gitkeep to track directory structure
        fs::File::create(p.join(".gitkeep"))?;
    }

    // 3. OUTPOST.md
    let outpost_md = format!(
        "# Outpost: {name}\n\n\
         **Role:** Active Project / Mission Sector\n\
         **Citadel:** {citadel_home}\n\n\
         ## Overview\n\n\
         This directory is a KoadOS Outpost. It represents an atomic project or mission\n\
         where agents execute specific tasks and maintain a dedicated context.\n\n\
         ## Sectors\n\n\
         - `docs/` — Project-specific documentation and task manifests\n\
         - `config/` — Outpost-level overrides (e.g. project-specific PATs)\n\
         - `updates/` — Chronological event stream for this project\n\
         - `.koados-outpost/` — Project-specific agent context and quests (Local Only)\n",
        name = name,
        citadel_home = config.home.display()
    );
    write_file(&cwd, "OUTPOST.md", &outpost_md)?;

    // 4. .koad-os symlink
    create_citadel_link(&cwd, &config.home)?;

    println!(
        "\n\x1b[1;32m[SUCCESS]\x1b[0m Outpost '{}' deployed successfully.",
        name
    );
    Ok(())
}

fn write_file(root: &Path, name: &str, content: &str) -> Result<()> {
    let p = root.join(name);
    if !p.exists() {
        fs::write(&p, content)?;
        println!("  \x1b[32m[CREATE]\x1b[0m {}", name);
    }
    Ok(())
}

fn create_citadel_link(target: &Path, citadel_home: &Path) -> Result<()> {
    let link_path = target.join(".koad-os");
    if !link_path.exists() {
        symlink(citadel_home, &link_path).context(
            "Failed to create .koad-os symlink. Ensure you have appropriate permissions.",
        )?;
        println!(
            "  \x1b[32m[LINK]\x1b[0m .koad-os -> {}",
            citadel_home.display()
        );
    }
    Ok(())
}
