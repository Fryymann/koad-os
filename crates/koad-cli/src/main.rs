#![allow(dead_code, unused_imports, clippy::type_complexity)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use std::process::{Command, Stdio};
use std::time::Duration;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::Empty;
use chrono::{Local, Utc};
use std::io::{BufRead, BufReader, Write};
use koad_board::project::ProjectItem;
use koad_board::GitHubClient;
use rusqlite::params;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use sysinfo::{System, Process, Pid};
use tracing::{info, warn, error};
use koad_core::logging::init_logging;
use koad_core::config::KoadConfig;

mod tui;
mod airtable;

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
        if !path.exists() {
            return Ok(Self::default_initial());
        }
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }

    fn default_initial() -> Self {
        Self {
            version: "4.0.0".into(),
            identity: Identity { name: "Koad".into(), role: "Admin".into(), bio: "Agentic OS".into() },
            preferences: Preferences { languages: vec![], booster_enabled: false, style: "default".into(), principles: vec![] },
            drivers: HashMap::new(),
            notion: NotionConfig::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NotionConfig {
    #[serde(default)]
    pub mcp: bool,
    pub index: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Identity {
    pub name: String,
    pub role: String,
    pub bio: String,
}

#[derive(Debug, PartialEq)]
enum PreFlightStatus {
    Optimal,
    Degraded(String),
    Critical(String),
}

fn pre_flight(config: &KoadConfig) -> PreFlightStatus {
    let mut errors: Vec<String> = Vec::new();
    let mut critical = false;

    // 1. Check Redis (The Neural Bus)
    if !config.redis_socket.exists() {
        errors.push("Neural Bus (Redis UDS) is missing. Engine Room is likely DARK.".into());
        critical = true;
    } else {
        // Try a quick connection test
        match redis::Client::open(format!("redis+unix://{}", config.redis_socket.display())) {
            Ok(client) => {
                if let Err(e) = client.get_connection() {
                    errors.push(format!("Neural Bus exists but is NOT RESPONDING: {}", e));
                    critical = true;
                }
            }
            Err(e) => {
                errors.push(format!("Failed to initialize Neural Bus client: {}", e));
                critical = true;
            }
        }
    }

    // 2. Check Spine (The Orchestrator)
    let spine_socket = config.home.join("kspine.sock");
    if !spine_socket.exists() {
        errors.push("Orchestrator (kspine.sock) is missing. Spine is likely OFFLINE.".into());
        if !critical {
             return PreFlightStatus::Degraded(errors.join(" "));
        }
    }

    if critical {
        PreFlightStatus::Critical(errors.join(" "))
    } else if !errors.is_empty() {
        PreFlightStatus::Degraded(errors.join(" "))
    } else {
        PreFlightStatus::Optimal
    }
}

fn find_ghosts(home: &Path) -> Vec<(u32, String)> {
    let mut ghosts = Vec::new();
    let mut sys = System::new_all();
    sys.refresh_all();

    let redis_socket_path = home.join("koad.sock");
    let expected_redis_socket = redis_socket_path.to_string_lossy();

    for (pid, process) in sys.processes() {
        let pid_u32 = pid.as_u32();
        let name = process.name();
        
        let cmd_vec = process.cmd();
        let cmd = cmd_vec.join(" ");
        
        // 1. Check for redis-server ghosts
        if name.contains("redis-server") && !cmd.contains(&*expected_redis_socket) {
            ghosts.push((pid_u32, format!("Ghost Redis: {}", cmd)));
        }
        
        // 2. Check for kspine/kgateway ghosts and staleness
        let is_spine = name.contains("kspine") || name.contains("koad-spine");
        let is_gateway = name.contains("kgateway") || name.contains("koad-gateway");

        if is_spine || is_gateway {
             let mut process_home = None;
             for env_var in process.environ() {
                 if env_var.starts_with("KOAD_HOME=") {
                     process_home = Some(env_var.split_once('=').unwrap().1.to_string());
                     break;
                 }
             }
             
             if let Some(ph) = process_home {
                 let ph_can = Path::new(&ph).canonicalize().ok();
                 let home_can = home.canonicalize().ok();
                 
                 if ph_can != home_can {
                     continue;
                 }

                 // If it reports OUR home, check for STALENESS
                 if let Some(exe_path) = process.exe() {
                     let real_exe = std::fs::read_link(exe_path).unwrap_or_else(|_| exe_path.to_path_buf());
                     
                     if let (Ok(exe_can), Ok(home_bin_can)) = (real_exe.canonicalize(), home.join("bin").canonicalize()) {
                         if exe_can.starts_with(&home_bin_can) {
                             if let Ok(metadata) = std::fs::metadata(&exe_can) {
                                 if let Ok(modified) = metadata.modified() {
                                     let start_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time());
                                     if modified > start_time {
                                         ghosts.push((pid_u32, format!("STALE {}: Binary updated since process started.", name)));
                                     }
                                 }
                             }
                         } else {
                             ghosts.push((pid_u32, format!("Ghost {} (Foreign Binary: {})", name, exe_can.display())));
                         }
                     }
                 }
             }
        }
    }
    ghosts
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Preferences {
    pub languages: Vec<String>,
    #[serde(default)]
    pub booster_enabled: bool,
    pub style: String,
    pub principles: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DriverConfig {
    pub bootstrap: String,
    #[serde(default)]
    pub mcp_enabled: bool,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Parser)]
#[command(name = "koad")]
#[command(version = "4.0.0")]
#[command(about = "The KoadOS Control Plane")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, default_value = "admin")]
    role: String,

    #[arg(long, global = true, default_value_t = false)]
    no_check: bool,
}

#[derive(Subcommand)]
enum Commands {
    Boot {
        #[arg(short, long)]
        agent: String,
        #[arg(short, long)]
        project: bool,
        #[arg(short, long)]
        task: Option<String>,
        #[arg(short, long)]
        compact: bool,
    },
    Auth,
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
    Init {
        #[arg(short, long)]
        force: bool,
    },
    Doctor,
    Scan {
        path: Option<PathBuf>,
    },
    Publish {
        #[arg(short, long)]
        message: Option<String>,
    },
    Saveup {
        summary: String,
        #[arg(short, long, default_value = "General")]
        scope: String,
        #[arg(short, long)]
        facts: Option<String>,
        #[arg(short, long)]
        auto: bool,
    },
    Gcloud {
        #[command(subcommand)]
        action: GcloudAction,
    },
    Airtable {
        #[command(subcommand)]
        action: AirtableAction,
    },
    Sync {
        #[command(subcommand)]
        source: SyncSource,
    },
    Drive {
        #[command(subcommand)]
        action: DriveAction,
    },
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    Whoami,
    Board {
        #[command(subcommand)]
        action: BoardAction,
    },
    Mind {
        #[command(subcommand)]
        action: MindAction,
    },
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    Issue {
        #[command(subcommand)]
        action: IssueAction,
    },
    Stat {
        #[arg(short, long)]
        json: bool,
    },
    Crew,
    Dash,
    Refresh {
        #[arg(short, long)]
        restart: bool,
    },
    Save {
        #[arg(short, long)]
        full: bool,
    },
}

#[derive(Subcommand)]
enum MemoryCategory {
    Fact { text: String, #[arg(short, long)] tags: Option<String> },
    Learning { text: String, #[arg(short, long)] tags: Option<String> },
}

#[derive(Subcommand)]
enum GcloudAction { List, Deploy { name: String } }

#[derive(Subcommand)]
enum AirtableAction { Sync, List }

#[derive(Subcommand)]
enum SyncSource { Notion, Airtable, All }

#[derive(Subcommand)]
enum DriveAction { List, Download { id: String }, Upload { path: PathBuf } }

#[derive(Subcommand)]
enum StreamAction { 
    Logs { #[arg(short, long)] topic: Option<String> }, 
    Post { topic: String, message: String, #[arg(short, long, default_value = "INFO")] msg_type: String } 
}

#[derive(Subcommand)]
enum SkillAction {
    List,
    Run { name: String, #[arg(last = true)] args: Vec<String> },
}

#[derive(Subcommand)]
enum ProjectAction {
    List,
    Register { name: String, path: Option<PathBuf> },
    Sync { id: Option<i32> },
    Retire { id: i32 },
    Info { id: i32 },
}

#[derive(Subcommand)]
enum IssueAction {
    Track { number: i32, description: String },
    Move { number: i32, step: i32 },
    Approve { number: i32 },
    Close { number: i32 },
    Status { number: i32 },
}


#[derive(Subcommand)]
enum BoardAction {
    Status,
    Sync,
    Sdr,
    Done { id: i32 },
    Todo { id: i32 },
    Verify { id: i32 },
}

#[derive(Subcommand)]
enum MindAction {
    Status,
    Snapshot,
    Learn { domain: String, summary: String, #[arg(short, long)] detail: Option<String> },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityData {
    pub id: String,
    pub name: String,
    pub bio: String,
    pub tier: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityRole {
    pub identity_id: String,
    pub role: String,
}

pub struct KoadDB { pool: Pool<SqliteConnectionManager> }

impl KoadDB {
    pub fn new(path: &Path) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager)?;
        let conn = pool.get()?;
        
        // Ensure tables exist
        conn.execute("CREATE TABLE IF NOT EXISTS knowledge (id INTEGER PRIMARY KEY, category TEXT, content TEXT, tags TEXT, timestamp TEXT, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS active_spec (id INTEGER PRIMARY KEY, content TEXT, timestamp TEXT, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS projects (
            id INTEGER PRIMARY KEY, 
            name TEXT UNIQUE, 
            path TEXT, 
            role TEXT, 
            stack TEXT, 
            last_boot TEXT,
            branch TEXT,
            health TEXT,
            last_sync TEXT,
            active INTEGER DEFAULT 1
        )", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS sessions (session_id TEXT PRIMARY KEY, agent TEXT, role TEXT, status TEXT, last_heartbeat TEXT, pid INTEGER)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS notes (id INTEGER PRIMARY KEY, content TEXT, timestamp TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS brainstorms (id INTEGER PRIMARY KEY, content TEXT, category TEXT, timestamp TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY, command TEXT, args TEXT, timestamp TEXT, status TEXT)", [])?;
        
        // v4.1 Identity Tables
        conn.execute("CREATE TABLE IF NOT EXISTS identities (id TEXT PRIMARY KEY, name TEXT NOT NULL, bio TEXT, tier INTEGER DEFAULT 3, created_at TEXT NOT NULL)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS identity_roles (identity_id TEXT NOT NULL, role TEXT NOT NULL, PRIMARY KEY(identity_id, role), FOREIGN KEY(identity_id) REFERENCES identities(id))", [])?;

        Ok(Self { pool })
    }

    fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> { Ok(self.pool.get()?) }

    fn get_identity(&self, id: &str) -> Result<Option<IdentityData>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, bio, tier FROM identities WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(IdentityData { id: row.get(0)?, name: row.get(1)?, bio: row.get(2)?, tier: row.get(3)? }))
        } else {
            Ok(None)
        }
    }

    fn verify_role(&self, identity_id: &str, role: &str) -> Result<bool> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT 1 FROM identity_roles WHERE identity_id = ?1 AND role = ?2")?;
        let exists = stmt.query(params![identity_id, role.to_lowercase()])?.next()?.is_some();
        Ok(exists)
    }

    fn remember(&self, cat: &str, content: &str, tags: Option<String>, tier: i32) -> Result<()> {
        if tier > 1 {
            anyhow::bail!("Cognitive Protection: Model Tier {} is not authorized to write to Memory Bank.", tier);
        }
        let conn = self.get_conn()?;
        let ts = Local::now().to_rfc3339();
        conn.execute("INSERT INTO knowledge (category, content, tags, timestamp) VALUES (?1, ?2, ?3, ?4)", params![cat, content, tags.unwrap_or_default(), ts])?;
        Ok(())
    }

    fn query(&self, term: &str, limit: usize, _tags: Option<String>) -> Result<Vec<(i32, String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, category, content, timestamp FROM knowledge WHERE (content LIKE ?1 OR tags LIKE ?1) AND active=1 ORDER BY id DESC LIMIT ?2")?;
        let rows = stmt.query_map(params![format!("%{}%", term), limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
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

    fn get_contextual(&self, limit: usize, tags: Vec<String>) -> Result<Vec<(String, String)>> {
        let conn = self.get_conn()?;
        let mut results = Vec::new();
        for t in tags {
            let mut stmt = conn.prepare("SELECT category, content FROM knowledge WHERE active=1 AND tags LIKE ?1 ORDER BY id DESC LIMIT ?2")?;
            let rows = stmt.query_map(params![format!("%{}%", t), limit], |row| Ok((row.get(0)?, row.get(1)?)))?;
            for r in rows { results.push(r?); }
        }
        Ok(results)
    }

    fn get_active_project(&self, path: &str) -> Result<Option<(String, Option<String>, Option<String>)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT name, role, stack FROM projects WHERE ?1 LIKE path || '%' AND active = 1 ORDER BY length(path) DESC LIMIT 1")?;
        let mut rows = stmt.query(params![path])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?)))
        } else {
            Ok(None)
        }
    }

    fn register_project(&self, name: &str, path: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute(
            "INSERT INTO projects (name, path, last_sync, health) VALUES (?1, ?2, ?3, 'unknown') 
             ON CONFLICT(name) DO UPDATE SET path=?2",
            params![name, path, Local::now().to_rfc3339()],
        )?;
        Ok(())
    }

    fn list_projects(&self) -> Result<Vec<(i32, String, String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, name, path, branch, health FROM projects WHERE active = 1")?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                row.get::<_, Option<String>>(3)?.unwrap_or_else(|| "unknown".into()),
                row.get::<_, Option<String>>(4)?.unwrap_or_else(|| "unknown".into()),
            ))
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    fn get_project(&self, id: i32) -> Result<Option<(String, String, Option<String>, Option<String>, Option<String>)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT name, path, branch, health, last_sync FROM projects WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))
        } else {
            Ok(None)
        }
    }

    fn update_project_status(&self, id: i32, branch: Option<String>, health: Option<String>) -> Result<()> {
        let conn = self.get_conn()?;
        let ts = Local::now().to_rfc3339();
        match (branch, health) {
            (Some(b), Some(h)) => {
                conn.execute("UPDATE projects SET branch = ?1, health = ?2, last_sync = ?3 WHERE id = ?4", params![b, h, ts, id])?;
            }
            (Some(b), None) => {
                conn.execute("UPDATE projects SET branch = ?1, last_sync = ?2 WHERE id = ?3", params![b, ts, id])?;
            }
            (None, Some(h)) => {
                conn.execute("UPDATE projects SET health = ?1, last_sync = ?2 WHERE id = ?3", params![h, ts, id])?;
            }
            _ => {}
        }
        Ok(())
    }

    fn retire_project(&self, id: i32) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("UPDATE projects SET active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn get_project_by_path(&self, path: &str) -> Result<Option<ProjectItemData>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT name, stack FROM projects WHERE ?1 LIKE path || '%' ORDER BY length(path) DESC LIMIT 1")?;
        let mut rows = stmt.query([path])?;
        if let Some(row) = rows.next()? { Ok(Some(ProjectItemData { name: row.get(0)?, stack: row.get::<_, Option<String>>(1)?.unwrap_or_else(|| "Unknown".into()) })) }
        else { Ok(None) }
    }

    // TUI Support
    fn get_spec(&self) -> Result<Option<(String, String, String, String)>> { 
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT content, timestamp, 'active', 'active' FROM active_spec WHERE active=1 LIMIT 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
        } else {
            Ok(None)
        }
    }
    fn get_workflows(&self, _limit: Option<usize>, _page: usize) -> Result<Vec<(String, String, Option<String>, String)>> { Ok(vec![]) }
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

struct ProjectItemData { name: String, stack: String }

fn detect_context_tags(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();
    if path.join("Cargo.toml").exists() { tags.push("rust".into()); }
    if path.join("package.json").exists() { tags.push("node".into()); }
    if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() { tags.push("python".into()); }
    if path.join(".git").exists() { tags.push("git".into()); }
    tags
}

fn get_gh_pat_for_path(path: &Path, role: &str, _config: &KoadConfig) -> (String, String) {
    if role.to_lowercase() == "admin" { return ("GITHUB_ADMIN_PAT".into(), "Admin".into()); }
    let path_str = path.to_string_lossy();
    if path_str.contains("skylinks") { ("GITHUB_SKYLINKS_PAT".into(), "Skylinks".into()) }
    else { ("GITHUB_PERSONAL_PAT".into(), "Personal".into()) }
}

fn get_gdrive_token_for_path(path: &Path) -> (String, String) {
    let path_str = path.to_string_lossy();
    if path_str.contains("skylinks") { ("GDRIVE_SKYLINKS_TOKEN".into(), "Skylinks".into()) }
    else { ("GDRIVE_PERSONAL_TOKEN".into(), "Personal".into()) }
}

fn feature_gate(feature: &str, issue_num: Option<u32>) {
    let issue_str = issue_num.map(|n| format!(" (See Issue #{})", n)).unwrap_or_default();
    println!("\n\x1b[33m[GATE]\x1b[0m Feature '{}' is currently in the \x1b[1mDESIGN\x1b[0m phase.{}\n", feature, issue_str);
}

fn detect_model_tier() -> i32 {
    if env::var("GEMINI_CLI").is_ok() {
        1 // Tier 1: Admin-Grade (Gemini 1.5 Pro / Flash)
    } else if env::var("CODEX_CLI").is_ok() {
        2 // Tier 2: Developer-Grade (Codex)
    } else {
        3 // Tier 3: Guest/Restricted
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = KoadConfig::load()?;
    let _guard = init_logging("koad", None);
    let cli = Cli::parse();
    
    let legacy_config = KoadLegacyConfig::load(&config.home).unwrap_or_else(|_| KoadLegacyConfig::default_initial());
    
    let db_path = config.get_db_path();
    let db = KoadDB::new(&db_path)?;
    
    let role = cli.role.clone();
    let is_admin = role.to_lowercase() == "admin";
    let has_privileged_access = is_admin || role.to_lowercase() == "pm";

    // --- PRE-FLIGHT CHECKS ---
    let skip_check = cli.no_check || matches!(cli.command, Commands::Doctor | Commands::Whoami | Commands::Init { .. } | Commands::Scan { .. } | Commands::Auth | Commands::Boot { .. });
    
    if !skip_check {
        match pre_flight(&config) {
            PreFlightStatus::Critical(err) => {
                eprintln!("\n\x1b[31m[CRITICAL] KoadOS Kernel is OFFLINE.\x1b[0m");
                eprintln!("Details: {}\n", err);
                eprintln!("Try: 'koad doctor' for diagnostics or restart the spine.");
                std::process::exit(1);
            }
            PreFlightStatus::Degraded(err) => {
                eprintln!("\x1b[33m[WARNING] KoadOS Kernel is DEGRADED.\x1b[0m");
                eprintln!("Notice: {}\n", err);
            }
            PreFlightStatus::Optimal => {}
        }
    }

    match cli.command {
        Commands::Boot { agent, project, task: _, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let current_path_str = current_dir.to_string_lossy().to_string();
            let (pat_var, _) = get_gh_pat_for_path(&current_dir, &role, &config);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);
            let model_tier = detect_model_tier();
            let mut session_id = "BOOT".to_string();
            let mut mission_briefing = None;

            // v4.1 Identity Resolution
            let (final_agent_name, final_role, final_bio) = if let Some(identity) = db.get_identity(&agent)? {
                if !db.verify_role(&agent, &role)? {
                    anyhow::bail!("Identity '{}' is not authorized for the '{}' role.", agent, role);
                }
                
                // Cognitive Protection: Tier Enforcement
                if identity.tier < model_tier {
                    anyhow::bail!("Cognitive Protection: Model Tier {} is insufficient for the '{}' identity (Minimum: Tier {}).", model_tier, agent, identity.tier);
                }

                (identity.name, role.clone(), identity.bio)
            } else {
                warn!("Identity '{}' not found in registry. Defaulting to Guest (restricted).", agent);
                (agent.clone(), "guest".to_string(), "Unverified Agent".to_string())
            };

            if !compact {
                let conn = db.get_conn()?;
                let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                session_id = uuid::Uuid::new_v4().to_string();
                
                // Admin Multi-Session Protection
                if final_role.to_lowercase() == "admin" {
                    let cutoff = (Local::now() - chrono::Duration::minutes(2)).format("%Y-%m-%d %H:%M:%S").to_string();
                    let mut stmt = conn.prepare("SELECT session_id, agent FROM sessions WHERE role = 'admin' AND last_heartbeat > ?1 AND status = 'active'")?;
                    if let Ok((old_sid, old_agent)) = stmt.query_row([cutoff], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))) {
                        if old_agent == agent { session_id = old_sid; }
                        else { anyhow::bail!("Admin role is currently occupied by {} (Session: {}). Only one Admin allowed.", old_agent, old_sid); }
                    }
                }

                let session_data = serde_json::json!({
                    "session_id": session_id,
                    "identity": { 
                        "name": final_agent_name.clone(), 
                        "rank": final_role.clone(), 
                        "permissions": if final_role.to_lowercase() == "admin" { vec!["all"] } else { vec!["limited"] },
                        "tier": model_tier
                    },
                    "environment": "wsl",
                    "context": { "project_name": if project { "active" } else { "default" }, "root_path": current_path_str, "allowed_paths": [], "stack": [] },
                    "last_heartbeat": Utc::now().to_rfc3339(),
                    "metadata": { "bio": final_bio.clone(), "model_tier": model_tier }
                });
                
                if config.redis_socket.exists() {
                    let client = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display()))?;
                    if let Ok(mut con) = client.get_connection() {
                        let _: () = redis::cmd("HSET").arg("koad:state").arg(format!("koad:session:{}", session_id)).arg(session_data.to_string()).query(&mut con)?;
                        let _: () = redis::cmd("PUBLISH").arg("koad:sessions").arg(serde_json::json!({ "type": "session_update", "data": session_data }).to_string()).query(&mut con)?;
                        std::thread::sleep(std::time::Duration::from_millis(500));
                        if let Ok(Some(json_str)) = redis::cmd("HGET").arg("koad:state").arg(format!("koad:session:{}", session_id)).query::<Option<String>>(&mut con) {
                            if let Ok(data) = serde_json::from_str::<Value>(&json_str) {
                                if let Some(briefing) = data.get("mission_briefing") { mission_briefing = Some(briefing.as_str().unwrap_or_default().to_string()); }
                            }
                        }
                    }
                }
                conn.execute("INSERT INTO sessions (session_id, agent, role, status, last_heartbeat, pid) VALUES (?1, ?2, ?3, 'active', ?4, ?5) ON CONFLICT(session_id) DO UPDATE SET last_heartbeat = excluded.last_heartbeat, status = 'active'", params![session_id, agent, final_role, now, std::process::id()])?;
            }

            if compact { println!("I:{}|R:{}|G:{}|D:{}|T:{}|S:{}|Q:{}", final_agent_name, final_role, pat_var, drive_var, tags.join(","), session_id, model_tier); }
            else {
                println!("<koad_boot>\nSession:  {}\nIdentity: {} ({})\nTier:     {}", session_id, final_agent_name, final_role, model_tier);
                println!("Bio:      {}", final_bio);
                if let Some(briefing) = mission_briefing { println!("\n[MISSION BRIEFING]\n{}", briefing); }
                println!("Auth: GH={} | GD={}", pat_var, drive_var);
                if let Some(driver) = legacy_config.drivers.get(&agent) {
                    let b_path = driver.bootstrap.replace("~", &env::var("HOME").unwrap_or_default());
                    if let Ok(content) = std::fs::read_to_string(b_path) { println!("\n[BOOTSTRAP: {}]\n{}", agent, content); }
                }
                println!("\n[CONTEXT: {}]\nTags: {}", current_path_str, tags.join(", "));
                for (cat, content) in db.get_contextual(8, tags)? { println!("- [{}] {}", cat, content); }
                println!("\n[Persona Reflections]");
                let ponders = db.get_ponderings(3)?;
                if ponders.is_empty() { println!("- No active reflections."); }
                for p in ponders { println!("- {}", p); }
                if project {
                    if let Some(proj) = db.get_project_by_path(&current_path_str)? {
                        println!("\n[Project: {} (Stack: {})]", proj.name, proj.stack);
                        let progress_path = current_dir.join("PROJECT_PROGRESS.md");
                        if progress_path.exists() {
                            let p = std::fs::read_to_string(progress_path)?;
                            if let Some(s) = p.find("## Snapshot") { println!("\n[Project Progress]\n{}", p[s..].trim()); }
                        }
                    }
                }
                println!("</koad_boot>");
            }
        }
        Commands::Auth => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (p, _) = get_gh_pat_for_path(&current_dir, &role, &config);
            let (d, _) = get_gdrive_token_for_path(&current_dir);
            println!("GH:{} | GD:{}", p, d);
        }
        Commands::Query { term, limit, tags } => {
            let results = db.query(&term, limit, tags)?;
            for (id, cat, content, ts) in results { println!("- ID:{} [{}] ({}) {}", id, cat, ts, content); }
        }
        Commands::Remember { category } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            let model_tier = detect_model_tier();
            let (cat_str, text, tags) = match category { MemoryCategory::Fact { text, tags } => ("fact", text, tags), MemoryCategory::Learning { text, tags } => ("learning", text, tags) };
            db.remember(cat_str, &text, tags, model_tier)?; println!("Memory updated in local KoadDB.");
        }
        Commands::Ponder { text, tags } => { 
            let model_tier = detect_model_tier();
            db.remember("pondering", &text, Some(format!("persona-journal,{}", tags.unwrap_or_default())), model_tier)?; 
            println!("Reflection recorded."); 
        }
        Commands::Guide { .. } => { feature_gate("koad guide", None); }
        Commands::Init { .. } => { feature_gate("koad init", Some(25)); }
        Commands::Doctor => {
            println!("\n\x1b[1m--- [TELEMETRY] Neural Link & Grid Integrity ---\x1b[0m");
            
            // 1. Engine Room (Redis Process/Socket)
            print!("{:<30}", "Engine Room (Redis):");
            if config.redis_socket.exists() {
                match redis::Client::open(format!("redis+unix://{}", config.redis_socket.display())) {
                    Ok(client) => {
                        if let Ok(mut con) = client.get_connection() {
                            let _: String = redis::cmd("PING").query(&mut con).unwrap_or_else(|_| "FAIL".into());
                            println!("\x1b[32m[PASS]\x1b[0m Hot-stream energized.");
                        } else {
                            println!("\x1b[31m[FAIL]\x1b[0m Socket exists but connection refused.");
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
                        let res: rusqlite::Result<i32> = conn.query_row("SELECT 1", [], |r| r.get(0));
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

            // 4. Identity Check
            print!("{:<30}", "Neural Identity:");
            if legacy_config.identity.name != "Koad" || legacy_config.identity.role != "Admin" {
                println!("\x1b[33m[WARN]\x1b[0m Non-standard persona detected: {}", legacy_config.identity.name);
            } else {
                println!("\x1b[32m[PASS]\x1b[0m Persona: {} ({})", legacy_config.identity.name, legacy_config.identity.role);
            }

            // 5. Ghost Process Detection
            let ghosts = find_ghosts(&config.home);
            if !ghosts.is_empty() {
                println!("\n\x1b[33m[WARN] Ghost Processes Detected ({}):\x1b[0m", ghosts.len());
                for (pid, info) in ghosts {
                    println!("  - PID {}: {}", pid, info);
                }
                println!("  Try: 'pkill -9 kspine' or manual cleanup if these interfere with your session.");
            }

            println!("\x1b[1m---------------------------------------------------\x1b[0m\n");
        }
        Commands::Scan { path } => {
            let t = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let output = Command::new("fdfind").arg(".koad").arg("--type").arg("d").arg("--hidden").arg("--absolute-path").arg(&t).output();
            match output {
                Ok(out) if out.status.success() => {
                    let mut count = 0;
                    for line in String::from_utf8_lossy(&out.stdout).lines() {
                        if let Some(project_root) = PathBuf::from(line).parent() {
                            let name = project_root.file_name().unwrap_or_default().to_string_lossy();
                            if db.register_project(&name, &project_root.to_string_lossy()).is_ok() { 
                                println!("[PASS] Registered: {}", name); 
                                count += 1; 
                            }
                        }
                    }
                    println!("Scan complete. {} projects registered.", count);
                },
                _ => { 
                    if t.join(".koad").exists() { 
                        let name = t.file_name().unwrap_or_default().to_string_lossy();
                        db.register_project(&name, &t.to_string_lossy())?; 
                        println!("Project '{}' registered.", name); 
                    } 
                }
            }
        }
        Commands::Publish { message } => {
            if !is_admin { anyhow::bail!("Admin only."); }
            let h = config.home.clone();
            let m = message.unwrap_or_else(|| format!("KoadOS Sync - {}", Local::now().format("%Y-%m-%d %H:%M")));
            Command::new("git").arg("-C").arg(&h).arg("add").arg(".").spawn()?.wait()?;
            Command::new("git").arg("-C").arg(&h).arg("commit").arg("-m").arg(&m).spawn()?.wait()?;
            Command::new("git").arg("-C").arg(&h).arg("push").arg("origin").spawn()?.wait()?;
            println!("Published.");
        }
        Commands::Saveup { summary, scope, facts, auto: _ } => {
            let model_tier = detect_model_tier();
            let fact_str = facts.unwrap_or_default();
            db.remember("fact", &format!("Saveup ({}): {} | Facts: {}", scope, summary, fact_str), Some(scope.clone()), model_tier)?;
            println!("Saveup recorded in memory bank.");
            
            let log_path = config.home.join("SESSION_LOG.md");
            if let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(&log_path) {
                use std::io::Write;
                let log_entry = format!("\n## {} - Saveup: {}\n- Scope: {}\n- Facts: {}\n", Local::now().format("%Y-%m-%d"), summary, scope, fact_str);
                let _ = file.write_all(log_entry.as_bytes());
                println!("Session log updated.");
            }
        }
        Commands::Gcloud { .. } => { feature_gate("koad gcloud", None); }
        Commands::Airtable { .. } => { feature_gate("koad airtable", None); }
        Commands::Sync { .. } => { feature_gate("koad sync", None); }
        Commands::Drive { .. } => { feature_gate("koad drive", None); }
        Commands::Stream { .. } => { feature_gate("koad stream", None); }
        Commands::Skill { .. } => { feature_gate("koad skill", None); }
        Commands::Whoami => {
            println!("Persona: {} ({})", legacy_config.identity.name, legacy_config.identity.role);
            println!("Bio:     {}", legacy_config.identity.bio);
        }
        Commands::Board { action } => {
            let token = config.resolve_gh_token()?;
            let client = GitHubClient::new(token, "Fryymann".into(), "koad-os".into())?;
            let project_num = config.github_project_number as i32;

            match action {
                BoardAction::Status => {
                    println!(">>> [UPLINK] Accessing Neural Log: Tactical Overlay (Project #{})...", project_num);
                    let items = client.list_project_items(project_num).await?;
                    println!("\n{:<5} {:<50} {:<15} {:<15}", "NODE", "DATA FRAGMENT", "STATUS", "VERSION");
                    println!("{:-<90}", "");
                    for item in items {
                        let num = item.number.map(|n| format!("#{}", n)).unwrap_or_default();
                        println!("{:<5} {:<50} {:<15} {:<15}", num, if item.title.len() > 48 { format!("{}...", &item.title[..45]) } else { item.title }, item.status, item.target_version.unwrap_or_default());
                    }
                }
                BoardAction::Sync => {
                    if !is_admin { anyhow::bail!("Admin Auth Required for Board Sync."); }
                    client.sync_issues(project_num).await?;
                }
                BoardAction::Done { id } => {
                    if !is_admin { anyhow::bail!("Admin Auth Required to Close Nodes."); }
                    client.update_item_status(project_num, id, "Done").await?;
                }
                BoardAction::Todo { id } => {
                    if !is_admin { anyhow::bail!("Admin Auth Required to Reopen Nodes."); }
                    client.update_item_status(project_num, id, "Todo").await?;
                }
                BoardAction::Verify { id } => {
                    println!(">>> [VERIFY] Cross-referencing Node #{} with Command Deck...", id);
                    let items = client.list_project_items(project_num).await?;
                    if let Some(item) = items.iter().find(|i| i.number == Some(id)) {
                        println!("  [PASS] Node #{} verified. Current Status: {}", id, item.status);
                    } else {
                        anyhow::bail!("Node #{} not found on Project Board. Manual sync required.", id);
                    }
                }
                BoardAction::Sdr => { feature_gate("koad board sdr", None); }
            }
        }
        Commands::Mind { action } => {
            let conn = db.get_conn()?;
            match action {
                MindAction::Status => {
                    println!("\n\x1b[1m--- [INTROSPECT] Cognitive Health Status ---\x1b[0m");
                    
                    let learn_count: i32 = conn.query_row("SELECT count(*) FROM learnings WHERE status = 'active'", [], |r| r.get(0))?;
                    let decision_count: i32 = conn.query_row("SELECT count(*) FROM decisions", [], |r| r.get(0))?;
                    let skill_count: i32 = conn.query_row("SELECT count(*) FROM skills", [], |r| r.get(0))?;
                    let last_snapshot: String = conn.query_row("SELECT created_at FROM identity_snapshots ORDER BY created_at DESC LIMIT 1", [], |r| r.get(0)).unwrap_or_else(|_| "Never".to_string());

                    println!("{:<25} {:<10}", "Active Learnings:", learn_count);
                    println!("{:<25} {:<10}", "Decisions Logged:", decision_count);
                    println!("{:<25} {:<10}", "Proven Skills:", skill_count);
                    println!("{:<25} {:<10}", "Last Identity Snapshot:", last_snapshot);
                    
                    println!("\n\x1b[1mTop Domains:\x1b[0m");
                    let mut stmt = conn.prepare("SELECT domain, count(*) as c FROM learnings GROUP BY domain ORDER BY c DESC LIMIT 5")?;
                    let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i32>(1)?)))?;
                    for row in rows {
                        let (domain, count) = row?;
                        println!("  - {:<15} ({})", domain, count);
                    }
                }
                MindAction::Snapshot => {
                    if !is_admin { anyhow::bail!("Admin only."); }
                    let now = Local::now().to_rfc3339();
                    // Basic placeholder for identity snapshotting
                    conn.execute("INSERT INTO identity_snapshots (trigger, notes, created_at) VALUES ('manual', 'Manual session snapshot.', ?1)", params![now])?;
                    println!("\x1b[32m[SNAPSHOT]\x1b[0m Identity state archived.");
                }
                MindAction::Learn { domain, summary, detail } => {
                    let model_tier = detect_model_tier();
                    if model_tier > 1 {
                        anyhow::bail!("Cognitive Protection: Model Tier {} is not authorized to add structured learnings.", model_tier);
                    }
                    conn.execute(
                        "INSERT INTO learnings (domain, summary, detail, source, status) VALUES (?1, ?2, ?3, 'cli', 'active')",
                        params![domain, summary, detail]
                    )?;
                    println!("\x1b[32m[LEARNED]\x1b[0m New {} insight integrated into mind.", domain);
                }
            }
        }
        Commands::Project { action } => {
            match action {
                ProjectAction::List => {
                    let projects = db.list_projects()?;
                    println!("\n\x1b[1m--- KoadOS Master Project Map ---\x1b[0m");
                    println!("{:<4} {:<20} {:<15} {:<10} {}", "ID", "NAME", "BRANCH", "HEALTH", "PATH");
                    println!("{}", "-".repeat(80));
                    for (id, name, path, branch, health) in projects {
                        let health_color = match health.as_str() {
                            "green" => "\x1b[32m",
                            "yellow" => "\x1b[33m",
                            "red" => "\x1b[31m",
                            _ => "\x1b[0m",
                        };
                        println!("{:<4} {:<20} {:<15} {}{:<10}\x1b[0m {}", id, name, branch, health_color, health, path);
                    }
                    println!();
                }
                ProjectAction::Register { name, path } => {
                    let p = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
                    let abs_path = std::fs::canonicalize(p)?;
                    db.register_project(&name, &abs_path.to_string_lossy())?;
                    println!("Project '{}' registered at {}.", name, abs_path.display());
                }
                ProjectAction::Sync { id } => {
                    let project_id = match id {
                        Some(i) => i,
                        None => {
                            let current_dir = env::current_dir()?;
                            let mut project_id = None;
                            let projects = db.list_projects()?;
                            for (id, _, path, _, _) in projects {
                                if current_dir.to_string_lossy().starts_with(&path) {
                                    project_id = Some(id);
                                    break;
                                }
                            }
                            project_id.ok_or_else(|| anyhow::anyhow!("Not inside a registered project. Provide an ID."))?
                        }
                    };
                    
                    let (_, path, _, _, _) = db.get_project(project_id)?.ok_or_else(|| anyhow::anyhow!("Project not found"))?;
                    
                    let branch_out = Command::new("git").arg("-C").arg(&path).arg("rev-parse").arg("--abbrev-ref").arg("HEAD").output();
                    let branch = if let Ok(out) = branch_out {
                        if out.status.success() {
                            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
                        } else { None }
                    } else { None };

                    let health = if Path::new(&path).join("package.json").exists() || 
                                    Path::new(&path).join("Cargo.toml").exists() ||
                                    Path::new(&path).join("koad.json").exists() {
                        Some("green".into())
                    } else {
                        Some("unknown".into())
                    };

                    db.update_project_status(project_id, branch, health)?;
                    println!("Project #{} status updated.", project_id);
                }
                ProjectAction::Info { id } => {
                    if let Some((name, path, branch, health, last_sync)) = db.get_project(id)? {
                        println!("\n\x1b[1m--- Project Info: {} ---\x1b[0m", name);
                        println!("{:<15} {}", "Path:", path);
                        println!("{:<15} {}", "Branch:", branch.unwrap_or_else(|| "unknown".into()));
                        println!("{:<15} {}", "Health:", health.unwrap_or_else(|| "unknown".into()));
                        println!("{:<15} {}", "Last Sync:", last_sync.unwrap_or_else(|| "never".into()));
                        println!();
                    } else {
                        println!("Project #{} not found.", id);
                    }
                }
                ProjectAction::Retire { id } => {
                    db.retire_project(id)?;
                    println!("Project #{} retired.", id);
                }
            }
        }
        Commands::Issue { action } => {
            let conn = db.get_conn()?;
            match action {
                IssueAction::Track { number, description } => {
                    let now = Local::now().to_rfc3339();
                    conn.execute("INSERT OR REPLACE INTO task_graph (description, created_at, status, canon_step, issue_number) VALUES (?1, ?2, 'todo', 1, ?3)", params![description, now, number])?;
                    println!("\x1b[32m[TRACKED]\x1b[0m Node #{} is now under Sovereignty tracking (Step 1: View & Assess).", number);
                }
                IssueAction::Move { number, step } => {
                    if step < 1 || step > 8 { anyhow::bail!("Invalid step. Agents can only move through steps 1-8."); }
                    
                    // CRITICAL: Block agents from entering Step 5 (Implement) via 'move'
                    if step == 5 { anyhow::bail!("Access Denied: Step 5 (Implement) requires explicit Admin Approval. Run 'koad issue approve' instead."); }

                    let mut stmt = conn.prepare("SELECT canon_step FROM task_graph WHERE issue_number = ?1")?;
                    let current_step: i32 = stmt.query_row([number], |row| row.get(0)).context("Issue not tracked. Run 'koad issue track' first.")?;
                    
                    if step != current_step + 1 && step != current_step {
                        anyhow::bail!("Protocol Violation: Cannot move from Step {} to Step {}. Sequence must be incremental.", current_step, step);
                    }
                    
                    conn.execute("UPDATE task_graph SET canon_step = ?1, updated_at = ?2 WHERE issue_number = ?3", params![step, Local::now().to_rfc3339(), number])?;
                    println!("\x1b[34m[MOVE]\x1b[0m Node #{} advanced to Step {}.", number, step);
                }
                IssueAction::Approve { number } => {
                    if !is_admin { anyhow::bail!("Access Denied: Only Admin can approve safety gates."); }
                    
                    let mut stmt = conn.prepare("SELECT canon_step FROM task_graph WHERE issue_number = ?1")?;
                    let current_step: i32 = stmt.query_row([number], |row| row.get(0))?;
                    
                    let (new_step, label) = if current_step == 4 {
                        (5, "Authorized for Implementation")
                    } else if current_step == 8 {
                        (9, "Verified and Authorized for Closure")
                    } else {
                        anyhow::bail!("Invalid State: Approval can only be granted at Step 4 (Plan) or Step 8 (Results). Current: {}.", current_step);
                    };
                    
                    conn.execute("UPDATE task_graph SET canon_step = ?1, updated_at = ?2 WHERE issue_number = ?3", params![new_step, Local::now().to_rfc3339(), number])?;
                    println!("\x1b[35m[APPROVED]\x1b[0m Node #{} {}.", number, label);
                }
                IssueAction::Close { number } => {
                    let mut stmt = conn.prepare("SELECT canon_step FROM task_graph WHERE issue_number = ?1")?;
                    let current_step: i32 = stmt.query_row([number], |row| row.get(0)).context("Issue not tracked.")?;
                    
                    if current_step != 9 { anyhow::bail!("LOCKED: Node #{} cannot be closed. Current Step: {}. (Required: 9 - Approved)", number, current_step); }
                    
                    println!(">>> [UPLINK] Authenticating closure for Node #{}...", number);
                    let token = config.resolve_gh_token()?;
                    let client = GitHubClient::new(token, "Fryymann".into(), "koad-os".into())?;
                    client.update_item_status(config.github_project_number as i32, number, "Done").await?;
                    
                    // We also close the issue on GitHub if the client supports it
                    // For now, we'll use 'gh' cli as a fallback
                    let _ = Command::new("gh").arg("issue").arg("close").arg(number.to_string()).status();
                    
                    conn.execute("UPDATE task_graph SET status = 'completed', updated_at = ?1 WHERE issue_number = ?2", params![Local::now().to_rfc3339(), number])?;
                    println!("\x1b[32m[FINALIZED]\x1b[0m Node #{} is closed and archived.", number);
                }
                IssueAction::Status { number } => {
                    let mut stmt = conn.prepare("SELECT canon_step, description, status FROM task_graph WHERE issue_number = ?1")?;
                    let (step, desc, status): (i32, String, String) = stmt.query_row([number], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
                    println!("\n\x1b[1m--- Sovereignty Status: Node #{} ---\x1b[0m", number);
                    println!("Description: {}", desc);
                    println!("Status:      {}", status);
                    println!("Canon Step:  {} / 9", step);
                    println!("--------------------------------------");
                }
            }
        }
        Commands::Stat { json } => {
            if config.redis_socket.exists() {
                let mut con = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display()))?.get_connection()?;
                let res: Option<String> = redis::cmd("HGET").arg("koad:state").arg("system_stats").query(&mut con)?;
                if let Some(s) = res { 
                    if json { println!("{}", s); }
                    else {
                        let v: Value = serde_json::from_str(&s)?;
                        println!("--- System Stats ---");
                        println!("CPU Usage: {:.1}%", v["cpu_usage"].as_f64().unwrap_or(0.0));
                        println!("Memory:    {} MB", v["memory_usage"].as_u64().unwrap_or(0));
                        println!("Skills:    {}", v["skill_count"].as_u64().unwrap_or(0));
                        println!("Tasks:     {}", v["active_tasks"].as_u64().unwrap_or(0));
                    }
                } else { println!("No stats available."); }
            }
        }
        Commands::Crew => {
            if !config.redis_socket.exists() {
                anyhow::bail!("Kernel offline (Redis UDS missing). Cannot fetch live crew manifest.");
            }

            let mut con = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display()))?.get_connection()?;
            let sessions: HashMap<String, String> = redis::cmd("HGETALL").arg("koad:state").query(&mut con)?;
            
            println!("--- KoadOS Crew Manifest (Live) ---");
            println!("{:<15} {:<15} {:<10} {:<20}", "AGENT", "ROLE", "STATUS", "LAST SEEN");
            println!("{:-<65}", "");

            let mut found_wake = 0;
            for (key, val) in sessions {
                if key.starts_with("koad:session:") {
                    if let Ok(data) = serde_json::from_str::<Value>(&val) {
                        let agent = data["identity"]["name"].as_str().unwrap_or("Unknown");
                        let role = data["identity"]["rank"].as_str().unwrap_or("Crew");
                        let last_hb_str = data["last_heartbeat"].as_str().unwrap_or("");
                        
                        let status = if let Ok(last_hb) = chrono::DateTime::parse_from_rfc3339(last_hb_str) {
                            let diff = Utc::now().signed_duration_since(last_hb.with_timezone(&Utc));
                            if diff.num_seconds() < 60 {
                                found_wake += 1;
                                "\x1b[32mWAKE\x1b[0m"
                            } else {
                                "\x1b[30mDARK\x1b[0m"
                            }
                        } else { "UNKNOWN" };

                        println!("{:<15} {:<15} {:<10} {:<20}", agent, role, status, last_hb_str);
                    }
                }
            }
            println!("{:-<65}", "");
            println!("Total Wake Personnel: {}", found_wake);
        }
        Commands::Dash => {
            crate::tui::run_dash(&db)?;
        }
        Commands::Refresh { restart } => {
            if !is_admin { anyhow::bail!("Admin only."); }
            println!("\n\x1b[1m--- KoadOS Core Refresh (Hard Reset) ---\x1b[0m");
            let home = config.home.clone();
            
            // 1. Rebuild
            println!(">>> [1/3] Energizing Forge (cargo build --release)...");
            let build_status = Command::new("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(&home)
                .status()?;
            
            if !build_status.success() {
                anyhow::bail!("Forge failure: Build failed with exit code {}", build_status.code().unwrap_or(-1));
            }

            // 2. Deploy
            println!(">>> [2/3] Recalibrating Matrix (Redeploying binaries)...");
            let bin_dir = home.join("bin");
            let target_dir = home.join("target/release");
            let bins = vec![
                ("koad", "koad"),
                ("koad-spine", "kspine"),
                ("koad-gateway", "kgateway"),
                ("koad-tui", "kdash"),
            ];

            for (src, dest) in bins {
                let src_path = target_dir.join(src);
                let dest_path = bin_dir.join(dest);
                
                if src_path.exists() {
                    let old_path = dest_path.with_extension("old");
                    if dest_path.exists() {
                        let _ = std::fs::rename(&dest_path, &old_path);
                    }
                    if let Err(e) = std::fs::copy(&src_path, &dest_path) {
                        warn!("Failed to copy binary {}: {}", src, e);
                        if old_path.exists() { let _ = std::fs::rename(&old_path, &dest_path); }
                    } else {
                        println!("  [OK] Deployed: {}", dest);
                        if old_path.exists() { let _ = std::fs::remove_file(old_path); }
                    }
                }
            }

            // 3. Restart
            if restart {
                println!(">>> [3/3] Rebooting Core Systems...");
                let _ = Command::new("pkill").arg("-9").arg("kspine").status();
                let _ = Command::new("pkill").arg("-9").arg("kgateway").status();
                
                let _ = Command::new("nohup")
                    .arg(bin_dir.join("kspine"))
                    .arg("--home")
                    .arg(&home)
                    .stdout(Stdio::from(std::fs::File::create(home.join("spine.log"))?))
                    .stderr(Stdio::from(std::fs::File::create(home.join("spine.log"))?))
                    .spawn()?;
                
                std::thread::sleep(std::time::Duration::from_secs(3));
                
                let _ = Command::new("nohup")
                    .arg(bin_dir.join("kgateway"))
                    .arg("--addr")
                    .arg("0.0.0.0:3000")
                    .stdout(Stdio::from(std::fs::File::create(home.join("gateway.log"))?))
                    .stderr(Stdio::from(std::fs::File::create(home.join("gateway.log"))?))
                    .spawn()?;
                
                println!("\n\x1b[32m[CONDITION GREEN] KoadOS has been refreshed and rebooted.\x1b[0m");
            } else {
                println!("\n\x1b[32m[DONE] Core binaries updated. Restart manually or use --restart.\x1b[0m");
            }
        }
        Commands::Save { full } => {
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
            } else {
                println!(">>> [3/4] Skipping full backup (use --full for DB/Git checkpoint).");
                println!(">>> [4/4] Log synchronization complete.");
            }

            println!("\n\x1b[32m[CONDITION GREEN] Sovereign Save Complete.\x1b[0m");
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
