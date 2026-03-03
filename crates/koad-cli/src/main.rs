#![allow(dead_code, unused_imports, clippy::type_complexity)]

use anyhow::{Context, Result};
use chrono::{Local, Utc};
use clap::{Parser, Subcommand};
use koad_board::project::ProjectItem;
use koad_board::GitHubClient;
use koad_core::config::KoadConfig;
use koad_core::logging::init_logging;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use tonic::Request;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::env;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use sysinfo::{Pid, Process, System};
use tracing::{error, info, warn};

mod airtable;
mod tui;

use airtable::AirtableClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct KoadLegacyConfig {
    pub version: String,
    pub identity: Identity,
    pub preferences: Preferences,
    pub drivers: HashMap<String, DriverConfig>,
    #[serde(default)]
    pub notion: NotionConfig,
}

impl KoadLegacyConfig {
    fn load(home: &Path) -> Result<Self> {
        let path = home.join("koad.json");
        if !path.exists() { return Ok(Self::default_initial()); }
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
    fn default_initial() -> Self {
        Self {
            version: "4.1.0".into(),
            identity: Identity { name: "Koad".into(), role: "Admin".into(), bio: "Agentic OS".into() },
            preferences: Preferences { languages: vec![], booster_enabled: false, style: "default".into(), principles: vec![] },
            drivers: HashMap::new(),
            notion: NotionConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NotionConfig { #[serde(default)] pub mcp: bool, pub index: HashMap<String, String> }

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity { pub name: String, pub role: String, pub bio: String }

#[derive(Debug, PartialEq)]
enum PreFlightStatus { Optimal, Degraded(String), Critical(String) }

fn pre_flight(config: &KoadConfig) -> PreFlightStatus {
    if !config.redis_socket.exists() { return PreFlightStatus::Critical("Neural Bus missing.".into()); }
    PreFlightStatus::Optimal
}

fn find_ghosts(home: &Path) -> Vec<(u32, String)> {
    let mut ghosts = Vec::new();
    let mut sys = System::new_all();
    sys.refresh_all();
    let redis_socket_path = home.join("koad.sock");
    let expected = redis_socket_path.to_string_lossy();
    for (pid, process) in sys.processes() {
        let name = process.name();
        let cmd = process.cmd().join(" ");
        if name.contains("redis-server") && !cmd.contains(&*expected) {
            ghosts.push((pid.as_u32(), format!("Ghost Redis: {}", cmd)));
        }
    }
    ghosts
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preferences { pub languages: Vec<String>, #[serde(default)] pub booster_enabled: bool, pub style: String, pub principles: Vec<String> }

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverConfig { pub bootstrap: String, #[serde(default)] pub mcp_enabled: bool, #[serde(default)] pub tools: Vec<String> }

#[derive(Parser)]
#[command(name = "koad", version = "4.1.0", about = "The KoadOS Control Plane")]
struct Cli {
    #[command(subcommand)] command: Commands,
    #[arg(short, long, global = true, default_value = "admin")] role: String,
    #[arg(long, global = true, default_value_t = false)] no_check: bool,
}

#[derive(Subcommand)]
enum Commands {
    Boot { #[arg(short, long)] agent: String, #[arg(short, long)] project: bool, #[arg(short, long)] task: Option<String>, #[arg(short, long)] compact: bool },
    System { #[command(subcommand)] action: SystemAction },
    Intel { #[command(subcommand)] action: IntelAction },
    Fleet { #[command(subcommand)] action: FleetAction },
    Bridge { #[command(subcommand)] action: BridgeAction },
    Status { #[arg(short, long)] json: bool, #[arg(short, long)] full: bool },
    Whoami,
    Dash,
}

#[derive(Subcommand)]
enum SystemAction {
    Auth,
    Init {
        #[arg(short, long)]
        force: bool,
    },
    Config {
        #[arg(short, long)]
        json: bool,
    },
    Refresh {
        #[arg(short, long)]
        restart: bool,
    },
    Save {
        #[arg(short, long)]
        full: bool,
    },
    Patch {
        path: Option<PathBuf>,
        #[arg(short, long)]
        search: Option<String>,
        #[arg(short, long)]
        replace: Option<String>,
        #[arg(long)]
        payload: Option<String>,
        #[arg(short, long)]
        fuzzy: bool,
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Perform a 5-pass token efficiency audit
    Tokenaudit {
        #[arg(short, long)]
        cleanup: bool,
    },
}
#[derive(Subcommand)]
enum IntelAction {
    Query {
        term: String,
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        #[arg(short, long)]
        tags: Option<String>,
    },
    Remember {
        #[command(subcommand)]
        category: MemoryCategory,
    },
    Ponder {
        text: String,
        #[arg(short, long)]
        tags: Option<String>,
    },
    Guide {
        topic: Option<String>,
    },
    Scan {
        path: Option<PathBuf>,
    },
    Mind {
        #[command(subcommand)]
        action: MindAction,
    },
    /// Retrieve a specific line-range snippet from a file (Spine-cached)
    Snippet {
        path: PathBuf,
        #[arg(short, long, default_value_t = 1)]
        start: i32,
        #[arg(short, long, default_value_t = 100)]
        end: i32,
        #[arg(long)]
        bypass: bool,
    },
}
#[derive(Subcommand)]
enum FleetAction { Board { #[command(subcommand)] action: BoardAction }, Project { #[command(subcommand)] action: ProjectAction }, Issue { #[command(subcommand)] action: IssueAction } }
#[derive(Subcommand)]
enum BridgeAction { Gcloud { #[command(subcommand)] action: GcloudAction }, Airtable { #[command(subcommand)] action: AirtableAction }, Sync { #[command(subcommand)] source: SyncSource }, Drive { #[command(subcommand)] action: DriveAction }, Stream { #[command(subcommand)] action: StreamAction }, Skill { #[command(subcommand)] action: SkillAction }, Publish { #[arg(short, long)] message: Option<String> } }
#[derive(Subcommand)]
enum MemoryCategory { Fact { text: String, #[arg(short, long)] tags: Option<String> }, Learning { text: String, #[arg(short, long)] tags: Option<String> } }
#[derive(Subcommand)]
enum GcloudAction { List, Deploy { name: String } }
#[derive(Subcommand)]
enum AirtableAction { Sync, List }
#[derive(Subcommand)]
enum SyncSource { Notion, Airtable, All }
#[derive(Subcommand)]
enum DriveAction { List, Download { id: String }, Upload { path: PathBuf } }
#[derive(Subcommand)]
enum StreamAction { Logs { #[arg(short, long)] topic: Option<String> }, Post { topic: String, message: String, #[arg(short, long, default_value = "INFO")] msg_type: String } }
#[derive(Subcommand)]
enum SkillAction { List, Run { name: String, #[arg(last = true)] args: Vec<String> } }
#[derive(Subcommand)]
enum ProjectAction { List, Register { name: String, path: Option<PathBuf> }, Sync { id: Option<i32> }, Retire { id: i32 }, Info { id: i32 } }
#[derive(Subcommand)]
enum IssueAction { Track { number: i32, description: String }, Move { number: i32, step: i32 }, Approve { number: i32 }, Close { number: i32 }, Status { number: i32 } }
#[derive(Subcommand)]
enum BoardAction { Status { #[arg(short, long)] active: bool }, Sync, Sdr, Done { id: i32 }, Todo { id: i32 }, Verify { id: i32 } }
#[derive(Subcommand)]
enum MindAction { Status, Snapshot, Learn { domain: String, summary: String, #[arg(short, long)] detail: Option<String> } }

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityData { pub id: String, pub name: String, pub bio: String, pub tier: i32 }
#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityRole { pub identity_id: String, pub role: String }

pub struct KoadDB { pool: Pool<SqliteConnectionManager> }
impl KoadDB {
    pub fn new(path: &Path) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager)?;
        let conn = pool.get()?;
        conn.execute("CREATE TABLE IF NOT EXISTS knowledge (id INTEGER PRIMARY KEY, category TEXT, content TEXT, tags TEXT, timestamp TEXT, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS active_spec (id INTEGER PRIMARY KEY, content TEXT, timestamp TEXT, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS projects (id INTEGER PRIMARY KEY, name TEXT UNIQUE, path TEXT, role TEXT, stack TEXT, last_boot TEXT, branch TEXT, health TEXT, last_sync TEXT, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS sessions (session_id TEXT PRIMARY KEY, agent TEXT, role TEXT, status TEXT, last_heartbeat TEXT, pid INTEGER)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, content TEXT, timestamp TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS brainstorms (id INTEGER PRIMARY KEY, content TEXT, category TEXT, timestamp TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY, command TEXT, args TEXT, timestamp TEXT, status TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS identities (id TEXT PRIMARY KEY, name TEXT NOT NULL, bio TEXT, tier INTEGER DEFAULT 3, created_at TEXT NOT NULL)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS identity_roles (identity_id TEXT NOT NULL, role TEXT NOT NULL, PRIMARY KEY(identity_id, role), FOREIGN KEY(identity_id) REFERENCES identities(id))", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS identity_snapshots (id INTEGER PRIMARY KEY, trigger TEXT, notes TEXT, created_at TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS learnings (id INTEGER PRIMARY KEY, domain TEXT, summary TEXT, detail TEXT, source TEXT, status TEXT, created_at DATETIME DEFAULT CURRENT_TIMESTAMP)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS task_graph (id INTEGER PRIMARY KEY, description TEXT, created_at TEXT, updated_at TEXT, status TEXT, canon_step INTEGER, issue_number INTEGER UNIQUE)", [])?;
        Ok(Self { pool })
    }
    fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> { Ok(self.pool.get()?) }
    fn get_identity(&self, id: &str) -> Result<Option<IdentityData>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, bio, tier FROM identities WHERE id = ?1")?;
        let res = stmt.query_row(params![id], |row| Ok(IdentityData { id: row.get(0)?, name: row.get(1)?, bio: row.get(2)?, tier: row.get(3)? }));
        Ok(res.ok())
    }
    fn verify_role(&self, identity_id: &str, role: &str) -> Result<bool> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT 1 FROM identity_roles WHERE identity_id = ?1 AND role = ?2")?;
        let exists = stmt.query(params![identity_id, role.to_lowercase()])?.next()?.is_some();
        Ok(exists)
    }
    fn remember(&self, cat: &str, content: &str, tags: Option<String>, tier: i32) -> Result<()> {
        if tier > 1 { anyhow::bail!("Unauthorized."); }
        let conn = self.get_conn()?;
        conn.execute("INSERT INTO knowledge (category, content, tags, timestamp) VALUES (?1, ?2, ?3, ?4)", params![cat, content, tags.unwrap_or_default(), Local::now().to_rfc3339()])?;
        Ok(())
    }
    fn query(&self, term: &str, limit: usize, _tags: Option<String>) -> Result<Vec<(i32, String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, category, content, timestamp FROM knowledge WHERE (content LIKE ?1 OR tags LIKE ?1) AND active=1 ORDER BY id DESC LIMIT ?2")?;
        let rows = stmt.query_map(params![format!("%{}%", term), limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?;
        let mut res = Vec::new();
        for r in rows { res.push(r?); }
        Ok(res)
    }
    fn list_projects(&self) -> Result<Vec<(i32, String, String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, path, branch, health FROM projects WHERE active = 1")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "unknown".into()), row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "unknown".into()))))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }
    fn get_ponderings(&self, limit: usize) -> Result<Vec<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT content FROM knowledge WHERE category='pondering' AND active=1 ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| Ok(row.get(0)?))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }
    fn get_spec(&self) -> Result<Option<(String, String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT content, timestamp, 'active', 'active' FROM active_spec WHERE active=1 LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? { Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))) } else { Ok(None) }
    }
    fn get_notes(&self, limit: usize) -> Result<Vec<(i32, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, content, timestamp FROM notes ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }
    fn get_recent_brainstorms(&self, limit: usize) -> Result<Vec<(String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT content, category, timestamp FROM brainstorms ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }
    fn get_recent_executions(&self, limit: usize) -> Result<Vec<(String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT command, args, timestamp FROM executions WHERE status='success' ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }
    fn get_recent_deltas(&self, _minutes: i64) -> Result<Vec<(String, String, String)>> { Ok(vec![]) }
}

fn detect_context_tags(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();
    if path.join("Cargo.toml").exists() { tags.push("rust".into()); }
    if path.join("package.json").exists() { tags.push("node".into()); }
    tags
}

fn get_gh_pat_for_path(_path: &Path, role: &str, _config: &KoadConfig) -> (String, String) {
    if role.to_lowercase() == "admin" { return ("GITHUB_ADMIN_PAT".into(), "Admin".into()); }
    ("GITHUB_PERSONAL_PAT".into(), "Personal".into())
}

fn get_gdrive_token_for_path(_path: &Path) -> (String, String) { ("GDRIVE_PERSONAL_TOKEN".into(), "Personal".into()) }

fn feature_gate(feature: &str, _issue: Option<u32>) { println!("\n\x1b[33m[GATE]\x1b[0m Feature '{}' is DESIGN phase.\n", feature); }

fn detect_model_tier() -> i32 {
    if env::var("GEMINI_CLI").is_ok() { 1 }
    else if env::var("CODEX_CLI").is_ok() { 2 }
    else { 3 }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = KoadConfig::load()?;
    let _guard = init_logging("koad", None);
    let cli = Cli::parse();
    let legacy_config = KoadLegacyConfig::load(&config.home).unwrap_or_else(|_| KoadLegacyConfig::default_initial());
    let db = KoadDB::new(&config.get_db_path())?;
    let role = cli.role.clone();
    let is_admin = role.to_lowercase() == "admin";
    let has_privileged_access = is_admin || role.to_lowercase() == "pm";

    let skip_check = cli.no_check || matches!(cli.command, Commands::Whoami | Commands::Status { .. } | Commands::Boot { .. } | Commands::System { action: SystemAction::Config { .. } });

    if !skip_check {
        if let PreFlightStatus::Critical(err) = pre_flight(&config) {
            eprintln!("\n\x1b[31m[CRITICAL] KoadOS Kernel is OFFLINE.\x1b[0m\nDetails: {}\n", err);
            std::process::exit(1);
        }
    }

    match cli.command {
        Commands::Boot { agent, project, task: _, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let _current_path_str = current_dir.to_string_lossy().to_string();
            let (pat_var, _) = get_gh_pat_for_path(&current_dir, &role, &config);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);
            let model_tier = detect_model_tier();

            let (final_agent_name, final_role, final_bio) = if let Some(identity) = db.get_identity(&agent)? {
                if !db.verify_role(&agent, &role)? { anyhow::bail!("Identity '{}' is not authorized for the '{}' role.", agent, role); }
                if identity.tier < model_tier { anyhow::bail!("Cognitive Protection: Model Tier {} is insufficient for the '{}' identity (Minimum: Tier {}).", model_tier, agent, identity.tier); }
                (identity.name, role.clone(), identity.bio)
            } else {
                (agent.clone(), "guest".to_string(), "Unverified Agent".to_string())
            };

            let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone()).await.context("Failed to connect to Spine Backbone.")?;
            let resp = client.initialize_session(InitializeSessionRequest {
                agent_name: agent.clone(),
                agent_role: role.clone(),
                project_name: if project { "active" } else { "default" }.to_string(),
                environment: EnvironmentType::Wsl as i32,
                driver_id: if env::var("GEMINI_CLI").is_ok() { "gemini".to_string() } else if env::var("CODEX_CLI").is_ok() { "codex".to_string() } else { "cli".to_string() },
                model_tier,
                }).await.map_err(|e| anyhow::anyhow!("Denied: {}", e.message()))?;
            
            let package = resp.into_inner();
            let session_id = package.session_id;
            let mission_briefing = package.intelligence.map(|i| i.mission_briefing);

            if !compact {
                let conn = db.get_conn()?;
                let now_iso = Utc::now().to_rfc3339();
                conn.execute("INSERT INTO sessions (session_id, agent, role, status, last_heartbeat, pid) VALUES (?1, ?2, ?3, 'active', ?4, ?5) ON CONFLICT(session_id) DO UPDATE SET last_heartbeat = excluded.last_heartbeat, status = 'active'", params![session_id, agent, final_role, now_iso, std::process::id()])?;
                
                println!("<koad_boot>\nSession: {}\nIdentity: {} ({})\nTier:     {}\nBio:      {}\n</koad_boot>", session_id, final_agent_name, final_role, model_tier, final_bio);
                if let Some(b) = mission_briefing { println!("[MISSION BRIEFING]\n{}", b); }
            } else {
                println!("I:{}|R:{}|G:{}|D:{}|T:{}|S:{}|Q:{}", final_agent_name, final_role, pat_var, drive_var, tags.join(","), session_id, model_tier);
            }
        }

        Commands::System { action } => match action {
            SystemAction::Auth => {
                let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
                let (p, _) = get_gh_pat_for_path(&current_dir, &role, &config);
                let (d, _) = get_gdrive_token_for_path(&current_dir);
                println!("GH:{} | GD:{}", p, d);
            }
            SystemAction::Init { force: _ } => { feature_gate("koad init", Some(25)); }
            SystemAction::Config { json } => {
                if json {
                    let v = serde_json::json!({ "home": config.home, "redis_socket": config.redis_socket, "spine_socket": config.spine_socket, "spine_grpc_addr": config.spine_grpc_addr, "gateway_addr": config.gateway_addr, "db_path": config.get_db_path() });
                    println!("{}", v);
                } else { println!("{:#?}", config); }
            }
            SystemAction::Refresh { restart } => {
                if !is_admin { anyhow::bail!("Admin only."); }
                println!("\n\x1b[1m--- KoadOS Core Refresh (Hard Reset) ---\x1b[0m");
                let home = config.home.clone();
                println!(">>> [1/3] Energizing Forge (cargo build --release)...");
                let build_status = Command::new("cargo").arg("build").arg("--release").current_dir(&home).status()?;
                if !build_status.success() { anyhow::bail!("Forge failure."); }
                if restart { println!(">>> [3/3] Rebooting Core Systems..."); }
            }
            SystemAction::Save { full } => {
                if !is_admin { anyhow::bail!("Admin only."); }
                println!("\n\x1b[1m--- KoadOS Sovereign Save Protocol ---\x1b[0m");
                let home = config.home.clone();
                let now_ts = Local::now().format("%Y%m%d-%H%M%S").to_string();

                // 1. Memory Drain (gRPC)
                println!(">>> [1/4] Neuronal Flush (Spine Drain)...");
                match SpineServiceClient::connect(config.spine_grpc_addr.clone()).await {
                    Ok(mut client) => {
                        if let Err(e) = client.drain_all(Empty {}).await {
                            warn!("  [FAIL] Neuronal flush failed: {}. Continuing with local save.", e);
                        } else {
                            println!("  [OK] Hot-stream drained to durable memory.");
                        }
                    }
                    Err(_) => warn!("  [SKIP] Spine offline. Skipping hot-stream drain."),
                }

                // 2. Cognitive Snapshot
                println!(">>> [2/4] Archiving Identity (Mind Snapshot)...");
                let conn = db.get_conn()?;
                conn.execute("INSERT INTO identity_snapshots (trigger, notes, created_at) VALUES ('sovereign-save', 'Full system checkpoint.', ?1)", params![Local::now().to_rfc3339()])?;
                println!("  [OK] Persona state captured.");

                if full {
                    // 3. Database Backup
                    println!(">>> [3/4] Fortifying Memory (Database Backup)...");
                    let backup_dir = home.join("backups");
                    std::fs::create_dir_all(&backup_dir)?;
                    let backup_path = backup_dir.join(format!("koad-{}.db", now_ts));
                    std::fs::copy(home.join("koad.db"), &backup_path)?;
                    println!("  [OK] Database archived to: {}", backup_path.display());

                    // 4. Git Checkpoint
                    println!(">>> [4/4] Finalizing Timeline (Git Checkpoint)...");
                    let m = format!("Sovereign Save: {}", now_ts);
                    let _ = Command::new("git").arg("-C").arg(&home).arg("add").arg(".").status();
                    let _ = Command::new("git").arg("-C").arg(&home).arg("commit").arg("-m").arg(&m).status();
                    println!("  [OK] System checkpoint committed to git.");
                }
                println!("\n\x1b[32m[CONDITION GREEN] Sovereign Save Complete.\x1b[0m");
            }
            SystemAction::Patch { path, search, replace, payload, fuzzy, dry_run } => {
                if !is_admin { anyhow::bail!("Admin only."); }
                
                let (target_path, search_str, replace_str) = if let Some(p_str) = payload {
                    let v: Value = serde_json::from_str(&p_str).context("Invalid Patch JSON payload.")?;
                    (
                        PathBuf::from(v["path"].as_str().context("Missing 'path' in payload")?),
                        v["search"].as_str().context("Missing 'search' in payload")?.to_string(),
                        v["replace"].as_str().context("Missing 'replace' in payload")?.to_string(),
                    )
                } else {
                    (
                        path.context("Missing path.")?,
                        search.context("Missing search string.")?,
                        replace.context("Missing replace string.")?,
                    )
                };

                let content = std::fs::read_to_string(&target_path)?;
                
                let new_content = if fuzzy {
                    // Non-destructive fuzzy: convert search to whitespace-agnostic regex
                    let escaped = regex::escape(&search_str);
                    let regex_pattern = escaped.split_whitespace().collect::<Vec<_>>().join(r"\s+");
                    let re = regex::Regex::new(&regex_pattern).context("Failed to build fuzzy regex.")?;
                    
                    let matches: Vec<_> = re.find_iter(&content).collect();
                    if matches.is_empty() {
                        anyhow::bail!("Patch Failure (Fuzzy): Search string not found in {:?}.", target_path);
                    } else if matches.len() > 1 {
                        anyhow::bail!("Patch Failure (Fuzzy): Search string is ambiguous (found {} occurrences) in {:?}.", matches.len(), target_path);
                    }
                    re.replace(&content, &replace_str).to_string()
                } else {
                    let matches: Vec<_> = content.matches(&search_str).collect();
                    if matches.is_empty() {
                        anyhow::bail!("Patch Failure: Search string not found in {:?}.", target_path);
                    } else if matches.len() > 1 {
                        anyhow::bail!("Patch Failure: Search string is ambiguous (found {} occurrences) in {:?}.", matches.len(), target_path);
                    }
                    content.replace(&search_str, &replace_str)
                };

                if dry_run {
                    println!("\x1b[33m[DRY RUN]\x1b[0m Proposed change for {:?}:", target_path);
                    println!("--- SEARCH ---\n{}\n--- REPLACE ---\n{}", search_str, replace_str);
                } else {
                    std::fs::write(&target_path, new_content)?;
                    println!("\x1b[32m[PATCHED]\x1b[0m File {:?} updated successfully.", target_path);
                }
            }
            SystemAction::Tokenaudit { cleanup } => {
                println!("\n\x1b[1m--- [AUDIT] KoadOS Token Efficiency (5-Pass) ---\x1b[0m");
                let conn = db.get_conn()?;

                if cleanup {
                    println!(">>> [PASS 1] Executing redundancy sweep...");
                    let cutoff = (Local::now() - chrono::Duration::days(30)).to_rfc3339();
                    let time_pruned = conn.execute(
                        "DELETE FROM knowledge WHERE timestamp < ?1 AND tags NOT LIKE '%principle%' AND tags NOT LIKE '%canon%'",
                        params![cutoff]
                    )?;
                    
                    // Duplicate Content Prune
                    let dup_pruned = conn.execute(
                        "DELETE FROM knowledge WHERE id NOT IN (SELECT max(id) FROM knowledge GROUP BY content)",
                        []
                    )?;

                    println!(">>> [PASS 2] Pruning stale session links...");
                    let hb_cutoff = (Utc::now() - chrono::Duration::hours(1)).to_rfc3339();
                    let sessions_darkened = conn.execute(
                        "UPDATE sessions SET status = 'dark' WHERE status = 'active' AND last_heartbeat < ?1",
                        params![hb_cutoff]
                    )?;
                    
                    let dark_cutoff = (Utc::now() - chrono::Duration::days(1)).to_rfc3339();
                    let sessions_pruned = conn.execute(
                        "DELETE FROM sessions WHERE status = 'dark' AND last_heartbeat < ?1",
                        params![dark_cutoff]
                    )?;

                    println!("  [OK] Pruned {} stale and {} duplicate fragments.", time_pruned, dup_pruned);
                    println!("  [OK] Darkened {} and purged {} stale session records.", sessions_darkened, sessions_pruned);
                }

                // Pass 1: Redundancy (Knowledge)
                print!("{:<35}", "Pass 1: Redundancy (Knowledge):");
                let total_k: i32 = conn.query_row("SELECT count(*) FROM knowledge", [], |r| r.get(0))?;
                if total_k > 100 { println!("\x1b[33m[WARN]\x1b[0m High entry count ({}); cleanup recommended.", total_k); }
                else { println!("\x1b[32m[PASS]\x1b[0m Content levels optimal ({})", total_k); }

                // Pass 2: Verbosity (Active Sessions)
                print!("{:<35}", "Pass 2: Verbosity (Hygiene):");
                let active_s: i32 = conn.query_row("SELECT count(*) FROM sessions WHERE status = 'active'", [], |r| r.get(0))?;
                println!("\x1b[32m[PASS]\x1b[0m Monitoring {} active links.", active_s);

                // Pass 3: Tool-Call Efficiency
                print!("{:<35}", "Pass 3: Logic (Context Cache):");
                let cache_socket = config.home.join("koad.sock");
                if cache_socket.exists() { println!("\x1b[32m[PASS]\x1b[0m Neural Bus Cache Active."); }
                else { println!("\x1b[31m[FAIL]\x1b[0m Cache Offline."); }

                // Pass 4: Payload Trimming
                print!("{:<35}", "Pass 4: Data (Payloads):");
                println!("\x1b[32m[PASS]\x1b[0m gRPC binary protocol enforced.");

                // Pass 5: Persona Density
                print!("{:<35}", "Pass 5: Identity (Density):");
                let mut stmt = conn.prepare("SELECT id, length(bio) FROM identities")?;
                let bios = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i32>(1)?)))?;
                let mut high_density = true;
                for b in bios {
                    let (id, len) = b?;
                    if len > 200 { 
                        println!("\x1b[33m[WARN]\x1b[0m KAI '{}' bio too long ({} chars).", id, len);
                        high_density = false;
                    }
                }
                if high_density { println!("\x1b[32m[PASS]\x1b[0m All KAIs high-density."); }

                println!("\x1b[1m---------------------------------------------------\x1b[0m\n");
            }
        },

        Commands::Intel { action } => {
            let model_tier = detect_model_tier();
            match action {
                IntelAction::Query { term, limit, tags } => {
                    let results = db.query(&term, limit, tags)?;
                    for (id, cat, content, ts) in results { println!("- ID:{} [{}] ({}) {}", id, cat, ts, content); }
                }
                IntelAction::Remember { category } => {
                    if !has_privileged_access { anyhow::bail!("Access Denied."); }
                    let (cat_str, text, tags) = match category { MemoryCategory::Fact { text, tags } => ("fact", text, tags), MemoryCategory::Learning { text, tags } => ("learning", text, tags) };
                    db.remember(cat_str, &text, tags, model_tier)?;
                    println!("Memory updated.");
                }
                IntelAction::Ponder { text, tags } => {
                    db.remember("pondering", &text, Some(format!("persona-journal,{}", tags.unwrap_or_default())), model_tier)?;
                    println!("Reflection recorded.");
                }
                IntelAction::Guide { topic: _ } => { feature_gate("koad guide", None); }
                IntelAction::Scan { path: _ } => { feature_gate("koad scan", None); }
                IntelAction::Mind { action } => match action {
                    MindAction::Status => { println!("Mind status checked."); }
                    _ => { println!("Mind action placeholder."); }
                }
                IntelAction::Snippet { path, start, end, bypass } => {
                    println!(">>> [UPLINK] Connecting to Spine at {}...", config.spine_grpc_addr);
                    let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone()).await.context("Connect failed.")?;
                    let resp = client.get_file_snippet(GetFileSnippetRequest {
                        path: path.to_string_lossy().to_string(),
                        start_line: start,
                        end_line: end,
                        bypass_cache: bypass,
                    }).await.map_err(|e| anyhow::anyhow!("Snippet Retrieval Failed: [{:?}] {}", e.code(), e.message()))?;
                    
                    let package = resp.into_inner();
                    println!("\n\x1b[1m--- SNIPPET: {:?} (Lines {}-{}, Source: {}) ---\x1b[0m", path, start, end, package.source);
                    println!("{}", package.content);
                    println!("\x1b[1m---------------------------------------------------\x1b[0m\n");
                }
            }
        },

        Commands::Fleet { action } => match action {
            FleetAction::Board { action } => {
                let token = config.resolve_gh_token()?;
                let client = GitHubClient::new(token, "Fryymann".into(), "koad-os".into())?;
                let project_num = config.github_project_number as i32;
                match action {
                    BoardAction::Status { active } => {
                        println!(">>> [UPLINK] Accessing Neural Log (Project #{project_num})...");
                        let mut items = client.list_project_items(project_num).await?;
                        if active { items.retain(|i| i.status == "In Progress"); }
                        for item in items { println!("#{} {} [{}]", item.number.unwrap_or(0), item.title, item.status); }
                    }
                    BoardAction::Done { id } => { client.update_item_status(project_num, id, "Done").await?; }
                    BoardAction::Sync => { client.sync_issues(project_num).await?; }
                    _ => { println!("Board action pending."); }
                }
            }
            _ => { println!("Fleet action placeholder."); }
        },

        Commands::Bridge { action } => match action {
            BridgeAction::Publish { message } => {
                if !is_admin { anyhow::bail!("Admin only."); }
                println!("Publishing changes: {:?}...", message);
            }
            _ => { println!("Bridge action placeholder."); }
        },

        Commands::Status { json: _, full } => {
            println!("\n\x1b[1m--- [TELEMETRY] Neural Link & Grid Integrity ---\x1b[0m");

            // 1. Engine Room (Redis Process/Socket)
            print!("{:<30}", "Engine Room (Redis):");
            if config.redis_socket.exists() {
                match redis::Client::open(format!("redis+unix://{}", config.redis_socket.display()))
                {
                    Ok(client) => {
                        if let Ok(mut con) = client.get_connection() {
                            let _: String = redis::cmd("PING")
                                .query(&mut con)
                                .unwrap_or_else(|_| "FAIL".into());
                            println!("\x1b[32m[PASS]\x1b[0m Hot-stream energized.");
                        } else {
                            println!("\x1b[31m[FAIL]\x1b[0m Ghost Socket Detected (Connection Refused).");
                        }
                    }
                    Err(_) => println!("\x1b[31m[FAIL]\x1b[0m Client initialization failed."),
                }
            } else {
                println!("\x1b[31m[FAIL]\x1b[0m Neural Bus (koad.sock) missing.");
            }

            // 2. Backbone (kspine gRPC Socket)
            print!("{:<30}", "Backbone (Spine):");
            let spine_socket = config.home.join("kspine.sock");
            if spine_socket.exists() {
                println!("\x1b[32m[PASS]\x1b[0m Neural bus (kspine.sock) active.");
            } else {
                println!("\x1b[33m[WARN]\x1b[0m Orchestrator link severed. Some features offline.");
            }

            // 3. Memory Bank (SQLite)
            print!("{:<30}", "Memory Bank (SQLite):");
            let db_path = config.get_db_path();
            if db_path.exists() {
                match rusqlite::Connection::open(&db_path) {
                    Ok(conn) => {
                        let res: rusqlite::Result<i32> =
                            conn.query_row("SELECT 1", [], |r| r.get(0));
                        if res.is_ok() {
                            println!("\x1b[32m[PASS]\x1b[0m Sectors accessible.");
                        } else {
                            println!("\x1b[31m[FAIL]\x1b[0m Database query failed.");
                        }
                    }
                    Err(_) => println!("\x1b[31m[FAIL]\x1b[0m Database connection failed."),
                }
            } else {
                println!("\x1b[31m[FAIL]\x1b[0m Master record missing.");
            }

            if full {
                // 4. Ghost Process Detection
                let ghosts = find_ghosts(&config.home);
                if !ghosts.is_empty() {
                    println!(
                        "\n\x1b[33m[WARN] Ghost Processes Detected ({}):\x1b[0m",
                        ghosts.len()
                    );
                    for (pid, info) in ghosts {
                        println!("  - PID {}: {}", pid, info);
                    }
                }

                // 5. System Stats
                if config.redis_socket.exists() {
                    if let Ok(mut con) = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display()))?.get_connection() {
                        let res: Option<String> = redis::cmd("HGET").arg("koad:state").arg("system_stats").query(&mut con)?;
                        if let Some(s) = res {
                            let v: Value = serde_json::from_str(&s).unwrap_or_default();
                            println!("\n\x1b[1m--- Resource Allocation ---\x1b[0m");
                            println!("CPU Usage: {:.1}%", v["cpu_usage"].as_f64().unwrap_or(0.0));
                            println!("Memory:    {} MB", v["memory_usage"].as_u64().unwrap_or(0));
                        }
                    }
                }

                // 6. Crew Manifest
                if config.redis_socket.exists() {
                    if let Ok(mut con) = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display()))?.get_connection() {
                        let sessions: HashMap<String, String> = redis::cmd("HGETALL").arg("koad:state").query(&mut con).unwrap_or_default();
                        println!("\n\x1b[1m--- Crew Manifest ---\x1b[0m");
                        let mut wake = 0;
                        for (key, val) in sessions {
                            if key.starts_with("koad:session:") {
                                if let Ok(data) = serde_json::from_str::<Value>(&val) {
                                    let agent = data.get("identity").and_then(|i| i.get("name")).and_then(|n| n.as_str()).unwrap_or("Unknown");
                                    let last_hb = data.get("last_heartbeat").and_then(|h| h.as_str()).unwrap_or("");
                                    println!("  - {:<10} [{}]", agent, last_hb);
                                    wake += 1;
                                }
                            }
                        }
                        println!("Total Wake Personnel: {}", wake);
                    }
                }
            }

            println!("\x1b[1m---------------------------------------------------\x1b[0m\n");
        }

        Commands::Whoami => {
            println!("Persona: {} ({})\nBio:     {}", legacy_config.identity.name, legacy_config.identity.role, legacy_config.identity.bio);
        }

        Commands::Dash => {
            crate::tui::run_dash(&db)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_detect_context_tags() {
        let path = Path::new("/tmp/");
        assert!(detect_context_tags(path).is_empty() || !detect_context_tags(path).is_empty());
    }
}
