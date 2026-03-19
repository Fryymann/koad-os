//! # KoadOS Navigation Map Handler
//!
//! Implements the core MUD-inspired navigation logic, context resolution, 
//! and dual-mode rendering (Concise vs. Verbose).
//!
//! Following RUST_CANON v1.0 standards for async I/O and zero-panic stability.

use crate::cli::MapAction;
use crate::db::KoadDB;
use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::citadel::v5::WorkspaceLevel;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info, warn};
use walkdir::WalkDir;

/// Handles the 'map' command group.
///
/// # Errors
/// Returns an error if database operations or filesystem metadata extraction fails.
pub async fn handle_map(
    action: Option<MapAction>,
    verbose: bool,
    config: &KoadConfig,
    db: &KoadDB,
) -> Result<()> {
    let agent_name = config.get_agent_name();
    let current_dir = env::current_dir().context("Failed to get current directory")?;

    debug!(agent = %agent_name, path = %current_dir.display(), "Executing map command");

    match action {
        Some(MapAction::Look) | None => {
            render_look(&current_dir, verbose, &agent_name, config, db).await?;
        }
        Some(MapAction::Exits) => {
            render_exits(&current_dir, &agent_name, db).await?;
        }
        Some(MapAction::Goto { target }) => {
            handle_goto(&target, &agent_name, db).await?;
        }
        Some(MapAction::Pin { alias, path, scope }) => {
            let p = path.unwrap_or_else(|| current_dir.to_string_lossy().to_string());
            let expanded_p = expand_tilde(&p).to_string_lossy().to_string();
            let agent_id = if scope == "personal" { Some(agent_name.as_str()) } else { None };
            db.add_pin(&alias, &expanded_p, &scope, agent_id)?;
            info!(alias = %alias, path = %expanded_p, "Fast-travel point established");
            println!(">>> [PIN] Fast-travel point established: [{}] -> {}", alias, expanded_p);
        }
        Some(MapAction::Pins) => {
            let pins = db.get_pins(&agent_name)?;
            println!("═══════════════════════════════════════════════");
            println!("📌 REGISTERED PINS (Fast Travel)");
            for (alias, path, scope) in pins {
                println!("   [{}] -> {} ({})", alias, path, scope);
            }
            println!("═══════════════════════════════════════════════");
        }
        Some(MapAction::Where { entity }) => {
            handle_where(&entity, config, db).await?;
        }
        Some(MapAction::Legend) => {
            render_legend();
        }
        Some(MapAction::History) => {
            let history = db.get_navigation_history(&agent_name, 10)?;
            println!("🧭 BREADCRUMB TRAIL (Recent History)");
            for (path, ts) in history {
                println!("   [{}] {}", ts, path);
            }
        }
        Some(MapAction::MapStatus) => {
            render_region_status(&current_dir, config);
        }
        Some(MapAction::Nearby) => {
            render_nearby(&current_dir, &agent_name, config, db).await?;
        }
    }

    // Log navigation for breadcrumbs
    if let Err(e) = db.log_navigation(&agent_name, &current_dir.to_string_lossy()) {
        warn!(error = %e, "Failed to log navigation breadcrumb");
    }

    Ok(())
}

async fn render_look(
    dir: &Path,
    verbose: bool,
    agent: &str,
    config: &KoadConfig,
    db: &KoadDB,
) -> Result<()> {
    let level = resolve_level(dir, config);
    let level_str = match level {
        WorkspaceLevel::LevelSystem => "SYSTEM",
        WorkspaceLevel::LevelCitadel => "CITADEL",
        WorkspaceLevel::LevelStation => "STATION",
        WorkspaceLevel::LevelOutpost => "OUTPOST",
        _ => "NEUTRAL BODY",
    };

    println!("══════════════ [ {} ] ══════════════", level_str);

    if verbose {
        render_verbose_header(dir, config).await?;
    } else {
        println!("📍 {}", dir.display());
    }

    // List immediate children (Concise)
    let mut entries = fs::read_dir(dir).await.context("Failed to read directory")?;
    while let Some(entry) = entries.next_entry().await? {
        let name = entry.file_name().to_string_lossy().into_owned();
        let prefix = if entry.file_type().await?.is_dir() { "├── " } else { "└── " };
        println!("{}{}", prefix, name);
    }

    // Layers
    render_missions(config).await?;
    render_pins(agent, db).await?;

    Ok(())
}

async fn render_verbose_header(dir: &Path, config: &KoadConfig) -> Result<()> {
    println!("📍 LOCATION: {}", dir.display());
    println!("   Context: {}", resolve_context(dir, config));
    
    // Area stats - use synchronous WalkDir but keep it limited
    let mut count = 0;
    let mut size = 0;
    for entry in WalkDir::new(dir).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            count += 1;
            size += entry.metadata()?.len();
        }
    }
    println!("   Area Stats: {} files | {:.2} KB total", count, size as f64 / 1024.0);
    Ok(())
}

async fn render_missions(config: &KoadConfig) -> Result<()> {
    let db_path = config.home.join("data/notion-sync.db");
    if !db_path.exists() {
        return Ok(());
    }

    if let Ok(conn) = rusqlite::Connection::open(db_path) {
        let mut stmt = conn.prepare("SELECT title FROM pages WHERE is_deleted = 0 LIMIT 3")?;
        let missions: Vec<String> = stmt.query_map([], |row| row.get(0))?
            .filter_map(Result::ok)
            .collect();
        
        if !missions.is_empty() {
            println!("🎯 ACTIVE MISSIONS:");
            for m in missions {
                println!("   [QUEST] {}", m);
            }
        }
    }
    Ok(())
}

async fn render_pins(agent: &str, db: &KoadDB) -> Result<()> {
    let pins = db.get_pins(agent)?;
    if !pins.is_empty() {
        print!("📌 Pins: ");
        let aliases: Vec<String> = pins.into_iter().map(|(a, _, _)| format!("[{}]", a)).collect();
        println!("{}", aliases.join(" "));
    }
    Ok(())
}

async fn render_exits(dir: &Path, agent: &str, db: &KoadDB) -> Result<()> {
    println!("🧭 EXITS");
    if let Some(parent) = dir.parent() {
        println!("   ↑ ../{} (Parent)", parent.file_name().unwrap_or_default().to_string_lossy());
    }
    
    let pins = db.get_pins(agent)?;
    for (alias, path, _) in pins {
        println!("   → [{}] {}", alias, path);
    }
    Ok(())
}

async fn handle_goto(target: &str, agent: &str, db: &KoadDB) -> Result<()> {
    if let Some(path) = db.resolve_pin(target, agent)? {
        println!(">>> [GOTO] Teleporting to: {}", path);
        println!("cd {}", path);
    } else {
        let p = PathBuf::from(target);
        if p.exists() {
            println!("cd {}", p.display());
        } else {
            println!("Error: Target '{}' not found in pins or filesystem.", target);
        }
    }
    Ok(())
}

async fn handle_where(query: &str, config: &KoadConfig, db: &KoadDB) -> Result<()> {
    println!("🔍 Searching for '{}' in the Master Map...", query);
    
    // 1. Check Pins
    let pins = db.get_pins(&config.get_agent_name())?;
    for (alias, path, _) in pins {
        if alias.contains(query) || path.contains(query) {
            println!("   [PIN] {} -> {}", alias, path);
        }
    }

    // 2. Check Projects
    for (name, p_config) in &config.projects {
        if name.contains(query) {
            println!("   [PROJECT] {} -> {}", name, p_config.path.display());
        }
    }

    // 3. Check Missions (Notion)
    let db_path = config.home.join("data/notion-sync.db");
    if db_path.exists() {
        if let Ok(conn) = rusqlite::Connection::open(db_path) {
            let mut stmt = conn.prepare("SELECT title, page_id FROM pages WHERE title LIKE ? AND is_deleted = 0")?;
            let rows = stmt.query_map([format!("%{}%", query)], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            
            for row in rows {
                if let Ok((title, id)) = row {
                    println!("   [MISSION] {} (ID: {})", title, id);
                }
            }
        }
    }

    Ok(())
}

fn resolve_context(path: &Path, config: &KoadConfig) -> String {
    let level = resolve_level(path, config);
    match level {
        WorkspaceLevel::LevelSystem => "System Root".to_string(),
        WorkspaceLevel::LevelCitadel => "Citadel Core".to_string(),
        WorkspaceLevel::LevelStation => "SLE Station".to_string(),
        WorkspaceLevel::LevelOutpost => "Project Outpost".to_string(),
        _ => "Unknown Domain".to_string(),
    }
}

fn resolve_level(path: &Path, config: &KoadConfig) -> WorkspaceLevel {
    if path == config.home {
        WorkspaceLevel::LevelCitadel
    } else if path.starts_with(&config.home) {
        WorkspaceLevel::LevelCitadel
    } else if path.to_string_lossy().contains("skylinks") {
        WorkspaceLevel::LevelStation
    } else if path.to_string_lossy().contains("projects") {
        WorkspaceLevel::LevelOutpost
    } else if path == Path::new("/") {
        WorkspaceLevel::LevelSystem
    } else {
        WorkspaceLevel::LevelUnspecified
    }
}

fn render_legend() {
    println!("═══════════════════════════════════════════════");
    println!("📜 MAP LEGEND");
    println!("   📍 Current Location");
    println!("   🧭 Navigation Exits / History");
    println!("   🎯 Active Mission / Quest Marker");
    println!("   📌 Fast-Travel Pin (Bookmark)");
    println!("   📊 Area Statistics / Analytics");
    println!("   ↑  Parent Directory");
    println!("   →  Pinned Connection");
    println!("═══════════════════════════════════════════════");
}

fn render_region_status(path: &Path, config: &KoadConfig) {
    let context = resolve_context(path, config);
    println!("📊 REGION STATUS: {}", context.to_uppercase());
    
    match resolve_level(path, config) {
        WorkspaceLevel::LevelCitadel => {
            println!("   Integrity: 🟢 CONDITION GREEN");
            println!("   Services:  [CASS] online | [Intel] online | [Forge] online");
        }
        WorkspaceLevel::LevelStation => {
            println!("   Integrity: 🟡 CAUTION");
            println!("   Notes:     Station level links undergoing recalibration.");
        }
        _ => {
            println!("   Integrity: 🟢 STABLE");
        }
    }
}

async fn render_nearby(dir: &Path, agent: &str, _config: &KoadConfig, db: &KoadDB) -> Result<()> {
    println!("🔍 SCANNING PROXIMITY...");
    
    // 1. Find related configs/docs
    if let Some(parent) = dir.parent() {
        let mut entries = fs::read_dir(parent).await?;
        while let Some(entry) = entries.next_entry().await? {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.ends_with(".toml") || name.ends_with(".md") {
                println!("   [POI] Parent Resource: ../{}", name);
            }
        }
    }

    // 2. Local Pins
    let pins = db.get_pins(agent)?;
    for (alias, path, _) in pins {
        let p = PathBuf::from(&path);
        if p.starts_with(dir) || (dir.parent().is_some() && p.starts_with(dir.parent().unwrap())) {
            println!("   [PIN] Nearby Landmark: [{}] -> {}", alias, path);
        }
    }

    Ok(())
}

fn expand_tilde(path: &str) -> PathBuf {
    if path.starts_with('~') {
        if let Some(home) = dirs::home_dir() {
            return home.join(&path[2..]);
        }
    }
    PathBuf::from(path)
}
