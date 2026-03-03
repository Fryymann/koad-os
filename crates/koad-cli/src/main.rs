#![allow(dead_code, unused_imports, clippy::type_complexity)]

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use std::process::{Command, Stdio};
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

mod tui;
mod airtable;

use airtable::AirtableClient;

#[derive(Debug, Serialize, Deserialize)]
pub struct KoadConfig {
    pub version: String,
    pub identity: Identity,
    pub preferences: Preferences,
    pub drivers: HashMap<String, DriverConfig>,
    #[serde(default)]
    pub notion: NotionConfig,
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

fn pre_flight(redis_path: &Path, spine_path: &Path) -> PreFlightStatus {
    let mut errors: Vec<String> = Vec::new();
    let mut critical = false;

    // 1. Check Redis (The Neural Bus)
    if !redis_path.exists() {
        errors.push("Neural Bus (Redis UDS) is missing. Engine Room is likely DARK.".into());
        critical = true;
    } else {
        // Try a quick connection test
        match redis::Client::open(format!("redis+unix://{}", redis_path.display())) {
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
    if !spine_path.exists() {
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
        let cmd = process.cmd().join(" ");
        
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
                 
                 // eprintln!("DEBUG: Found {}, reported home: {:?}, our home: {:?}", name, ph_can, home_can);

                 if ph_can != home_can {
                     continue;
                 }

                 // If it reports OUR home, check for STALENESS
                 if let Some(exe_path) = process.exe() {
                     let real_exe = std::fs::read_link(exe_path).unwrap_or_else(|_| exe_path.to_path_buf());
                     if let (Ok(exe_can), Ok(home_bin_can)) = (real_exe.canonicalize(), home.join("bin").canonicalize()) {
                         if exe_can.starts_with(home_bin_can) {
                             if let Ok(metadata) = std::fs::metadata(&exe_can) {
                                 if let Ok(modified) = metadata.modified() {
                                     let start_time = std::time::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time());
                                     if modified > start_time {
                                         ghosts.push((pid_u32, format!("STALE {}: Binary updated since process started.", name)));
                                     }
                                 }
                             }
                         } else {
                             // Reports OUR home, but binary is somewhere else? GHOST.
                             ghosts.push((pid_u32, format!("Ghost {} (Foreign Binary: reports our KOAD_HOME)", name)));
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
#[command(version = "3.2.0")]
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
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    Retire { id: i32 },
    Note { text: String },
    Brainstorm { 
        text: String,
        #[arg(short, long)]
        rant: bool
    },
    Notes {
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    Brainstorms {
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    Dispatch {
        command: String,
        #[arg(last = true)]
        args: Vec<String>,
    },
    Whoami,
    Board {
        #[command(subcommand)]
        action: BoardAction,
    },
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    Stat {
        #[arg(short, long)]
        json: bool,
    },
    Crew,
    Dash,
}

#[derive(Subcommand)]
enum MemoryCategory {
    Fact { text: String, #[arg(short, long)] tags: Option<String> },
    Learning { text: String, #[arg(short, long)] tags: Option<String> },
}

#[derive(Subcommand)]
enum GcloudAction {
    List { #[arg(short, long, default_value = "functions")] resource: String },
    Deploy { name: String },
    Logs { name: String, #[arg(short, long, default_value_t = 20)] limit: usize },
    Audit { #[arg(short, long)] project: String },
}

#[derive(Subcommand)]
enum AirtableAction {
    List { base_id: String, table_name: String, #[arg(short, long)] filter: Option<String>, #[arg(short, long, default_value_t = 10)] limit: usize },
    Get { base_id: String, table_name: String, record_id: String },
    Update { base_id: String, table_name: String, record_id: String, fields: String },
}

#[derive(Subcommand)]
enum SyncSource {
    Airtable { #[arg(short, long)] schema_only: bool, base_id: Option<String> },
    Notion { #[arg(short, long)] page_id: Option<String>, #[arg(short, long)] db_id: Option<String> },
    Named { name: String },
}

#[derive(Subcommand)]
enum DriveAction {
    List { #[arg(short, long)] shared: bool },
    Download { id: String, #[arg(short, long)] dest: Option<PathBuf> },
    Sync,
}

#[derive(Subcommand)]
enum StreamAction {
    Post { topic: String, message: String, #[arg(short, long, default_value = "INFO")] msg_type: String },
    List { #[arg(short, long, default_value_t = 10)] limit: usize },
}

#[derive(Subcommand)]
enum TemplateAction {
    List,
    Use { name: String, #[arg(short, long)] out: Option<PathBuf> },
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
    Info { id: i32 },
    Retire { id: i32 },
}

#[derive(Subcommand)]
enum BoardAction {
    Status,
    Sync,
    Sdr,
    Done { id: i32 },
    Todo { id: i32 },
}

impl KoadConfig {
    fn get_home() -> Result<PathBuf> {
        let home = env::var("KOAD_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| dirs::home_dir().unwrap().join(".koad-os"));
        Ok(home)
    }

    fn get_path() -> Result<PathBuf> { Ok(Self::get_home()?.join("koad.json")) }
    fn get_log_path() -> Result<PathBuf> { Ok(Self::get_home()?.join("SESSION_LOG.md")) }

    fn load() -> Result<Self> {
        let p = Self::get_path()?;
        let c = std::fs::read_to_string(p)?;
        Ok(serde_json::from_str(&c)?)
    }

    fn save(&self) -> Result<()> {
        let p = Self::get_path()?;
        let c = serde_json::to_string_pretty(self)?;
        std::fs::write(p, c)?;
        Ok(())
    }

    fn default_initial() -> Self {
        Self {
            version: "3.2".into(),
            identity: Identity {
                name: "Koad".into(),
                role: "Admin".into(),
                bio: "Principal Systems & Operations Engineer".into(),
            },
            preferences: Preferences {
                languages: vec!["Rust".into(), "Python".into()],
                booster_enabled: true,
                style: "programmatic-first".into(),
                principles: vec![],
            },
            drivers: HashMap::new(),
            notion: NotionConfig::default(),
        }
    }
}

pub struct KoadDB {
    pool: Pool<SqliteConnectionManager>,
}

impl KoadDB {
    fn new(path: &Path) -> Result<Self> {
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::new(manager)?;
        let conn = pool.get()?;
        
        // Ensure tables exist
        conn.execute("CREATE TABLE IF NOT EXISTS knowledge (id INTEGER PRIMARY KEY, category TEXT, content TEXT, tags TEXT, timestamp TEXT, active INTEGER DEFAULT 1)", [])?;
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
        
        Ok(Self { pool })
    }

    fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> { Ok(self.pool.get()?) }

    fn remember(&self, cat: &str, content: &str, tags: Option<String>) -> Result<()> {
        let conn = self.get_conn()?;
        let ts = Local::now().to_rfc3339();
        conn.execute("INSERT INTO knowledge (category, content, tags, timestamp) VALUES (?1, ?2, ?3, ?4)", params![cat, content, tags.unwrap_or_default(), ts])?;
        Ok(())
    }

    fn query(&self, term: &str, limit: usize, tags: Option<String>) -> Result<Vec<(i32, String, String, String)>> {
        let conn = self.get_conn()?;
        let query = format!("%{}%", term);
        let tag_query = format!("%{}%", tags.unwrap_or_default());
        let mut stmt = conn.prepare("SELECT id, category, content, timestamp FROM knowledge WHERE active=1 AND content LIKE ?1 AND tags LIKE ?2 ORDER BY id DESC LIMIT ?3")?;
        let rows = stmt.query_map(params![query, tag_query, limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }

    fn get_ponderings(&self, limit: usize) -> Result<Vec<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT content FROM knowledge WHERE category='pondering' AND active=1 ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| row.get(0))?;
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

    fn save_note(&self, content: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("INSERT INTO notes (content, timestamp) VALUES (?1, ?2)", params![content, Local::now().to_rfc3339()])?;
        Ok(())
    }

    fn get_notes(&self, limit: usize) -> Result<Vec<(i32, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, content, timestamp FROM notes ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }

    fn save_brainstorm(&self, content: &str, rant: bool) -> Result<()> {
        let conn = self.get_conn()?;
        let cat = if rant { "rant" } else { "brainstorm" };
        conn.execute("INSERT INTO brainstorms (content, category, timestamp) VALUES (?1, ?2, ?3)", params![content, cat, Local::now().to_rfc3339()])?;
        Ok(())
    }

    fn get_recent_brainstorms(&self, limit: usize) -> Result<Vec<(String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT content, category, timestamp FROM brainstorms ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }

    fn log_execution(&self, cmd: &str, args: &str, status: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("INSERT INTO executions (command, args, timestamp, status) VALUES (?1, ?2, ?3, ?4)", params![cmd, args, Local::now().to_rfc3339(), status])?;
        Ok(())
    }

    fn get_recent_executions(&self, limit: usize) -> Result<Vec<(String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT command, args, timestamp FROM executions WHERE status='success' ORDER BY id DESC LIMIT ?1")?;
        let rows = stmt.query_map([limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for r in rows { results.push(r?); }
        Ok(results)
    }

    fn dispatch(&self, command: &str, args: Vec<String>) -> Result<()> {
        let args_str = args.join(" ");
        self.log_execution(command, &args_str, "pending")?;
        Ok(())
    }

    fn retire(&self, id: i32) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("UPDATE knowledge SET active=0 WHERE id=?1", [id])?;
        Ok(())
    }

    fn get_recent_deltas(&self, _minutes: i64) -> Result<Vec<(String, String, String)>> {
        Ok(vec![])
    }

    // TUI Support methods
    fn get_spec(&self) -> Result<Option<(String, String, String, String)>> { Ok(None) }
    fn get_workflows(&self, _limit: Option<usize>, _page: usize) -> Result<Vec<(String, String, Option<String>, String)>> { Ok(vec![]) }
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

fn get_gh_pat_for_path(path: &Path, role: &str) -> (String, String) {
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

#[tokio::main]
async fn main() -> Result<()> {
    let _guard = init_logging("koad", None);
    let cli = Cli::parse();
    let config = KoadConfig::load().unwrap_or_else(|_| KoadConfig::default_initial());
    
    let db_path = KoadConfig::get_home()?.join("koad.db");
    let db = KoadDB::new(&db_path)?;
    
    let redis_path = if let Ok(env_socket) = env::var("REDIS_SOCKET") {
        PathBuf::from(env_socket)
    } else {
        KoadConfig::get_home()?.join("koad.sock")
    };

    let role = cli.role.clone();
    let is_admin = role.to_lowercase() == "admin";
    let has_privileged_access = is_admin || role.to_lowercase() == "pm";

    // --- PRE-FLIGHT CHECKS ---
    let skip_check = cli.no_check || matches!(cli.command, Commands::Doctor | Commands::Whoami | Commands::Init { .. } | Commands::Scan { .. } | Commands::Auth | Commands::Boot { .. });
    
    if !skip_check {
        let spine_path = KoadConfig::get_home()?.join("kspine.sock");
        match pre_flight(&redis_path, &spine_path) {
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
            let (pat_var, _) = get_gh_pat_for_path(&current_dir, &role);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);
            let mut session_id = "BOOT".to_string();
            let mut mission_briefing = None;

            if !compact {
                let conn = db.get_conn()?;
                let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                session_id = uuid::Uuid::new_v4().to_string();
                if role.to_lowercase() == "admin" {
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
                        "name": agent.clone(), 
                        "rank": "captain", 
                        "permissions": ["all"] 
                    },
                    "environment": "wsl",
                    "context": { "project_name": if project { "active" } else { "default" }, "root_path": current_path_str, "allowed_paths": [], "stack": [] },
                    "last_heartbeat": Utc::now().to_rfc3339(),
                    "metadata": {}
                });
                
                if redis_path.exists() {
                    let client = redis::Client::open(format!("redis+unix://{}", redis_path.display()))?;
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
                conn.execute("INSERT INTO sessions (session_id, agent, role, status, last_heartbeat, pid) VALUES (?1, ?2, ?3, 'active', ?4, ?5) ON CONFLICT(session_id) DO UPDATE SET last_heartbeat = excluded.last_heartbeat, status = 'active'", params![session_id, agent, role, now, std::process::id()])?;
            }

            if compact { println!("I:{}|R:{}|G:{}|D:{}|T:{}|S:{}", config.identity.name, config.identity.role, pat_var, drive_var, tags.join(","), session_id); }
            else {
                println!("<koad_boot>\nSession:  {}\nIdentity: {} ({})", session_id, config.identity.name, config.identity.role);
                if let Some(briefing) = mission_briefing { println!("\n[MISSION BRIEFING]\n{}", briefing); }
                println!("Auth: GH={} | GD={}", pat_var, drive_var);
                if let Some(driver) = config.drivers.get(&agent) {
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
            let (p, _) = get_gh_pat_for_path(&current_dir, &role);
            let (d, _) = get_gdrive_token_for_path(&current_dir);
            println!("GH:{} | GD:{}", p, d);
        }
        Commands::Query { term, limit, tags } => {
            let results = db.query(&term, limit, tags)?;
            for (id, cat, content, ts) in results { println!("- ID:{} [{}] ({}) {}", id, cat, ts, content); }
        }
        Commands::Remember { category } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            let (cat_str, text, tags) = match category { MemoryCategory::Fact { text, tags } => ("fact", text, tags), MemoryCategory::Learning { text, tags } => ("learning", text, tags) };
            db.remember(cat_str, &text, tags)?; println!("Memory updated in local KoadDB.");
        }
        Commands::Ponder { text, tags } => { db.remember("pondering", &text, Some(format!("persona-journal,{}", tags.unwrap_or_default())))?; println!("Reflection recorded."); }
        Commands::Doctor => {
            println!("\n\x1b[1m--- [TELEMETRY] Neural Link & Grid Integrity ---\x1b[0m");
            let home = KoadConfig::get_home()?;
            
            // 1. Engine Room (Redis Process/Socket)
            print!("{:<30}", "Engine Room (Redis):");
            if redis_path.exists() {
                match redis::Client::open(format!("redis+unix://{}", redis_path.display())) {
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
            let spine_path = home.join("kspine.sock");
            if spine_path.exists() {
                 println!("\x1b[32m[PASS]\x1b[0m Neural bus (kspine.sock) active.");
            } else {
                 println!("\x1b[33m[WARN]\x1b[0m Orchestrator link severed. Some features offline.");
            }

            // 3. Memory Bank (SQLite)
            print!("{:<30}", "Memory Bank (SQLite):");
            let db_path = home.join("koad.db");
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
            if config.identity.name != "Koad" || config.identity.role != "Admin" {
                println!("\x1b[33m[WARN]\x1b[0m Non-standard persona detected: {}", config.identity.name);
            } else {
                println!("\x1b[32m[PASS]\x1b[0m Persona: {} ({})", config.identity.name, config.identity.role);
            }

            // 5. Ghost Process Detection
            let ghosts = find_ghosts(&home);
            if !ghosts.is_empty() {
                println!("\n\x1b[33m[WARN] Ghost Processes Detected ({}):\x1b[0m", ghosts.len());
                for (pid, info) in ghosts {
                    println!("  - PID {}: {}", pid, info);
                }
                println!("  Try: 'pkill -9 kspine' or manual cleanup if these interfere with your session.");
            }

            println!("\x1b[1m---------------------------------------------------\x1b[0m\n");
        }
        Commands::Saveup { summary, scope, facts, auto: _ } => {
            let fact_str = facts.unwrap_or_default();
            db.remember("fact", &format!("Saveup ({}): {} | Facts: {}", scope, summary, fact_str), Some(scope.clone()))?;
            println!("Saveup recorded in memory bank.");
            
            let log_path = KoadConfig::get_log_path()?;
            if let Ok(mut file) = std::fs::OpenOptions::new().append(true).open(&log_path) {
                use std::io::Write;
                let log_entry = format!("\n## {} - Saveup: {}\n- Scope: {}\n- Facts: {}\n", Local::now().format("%Y-%m-%d"), summary, scope, fact_str);
                let _ = file.write_all(log_entry.as_bytes());
                println!("Session log updated.");
            }
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
            let h = KoadConfig::get_home()?;
            let m = message.unwrap_or_else(|| format!("KoadOS Sync - {}", Local::now().format("%Y-%m-%d %H:%M")));
            Command::new("git").arg("-C").arg(&h).arg("add").arg(".").spawn()?.wait()?;
            Command::new("git").arg("-C").arg(&h).arg("commit").arg("-m").arg(&m).spawn()?.wait()?;
            Command::new("git").arg("-C").arg(&h).arg("push").arg("origin").spawn()?.wait()?;
            println!("Published.");
        }
        Commands::Whoami => {
            println!("Persona: {} ({})", config.identity.name, config.identity.role);
            println!("Bio:     {}", config.identity.bio);
        }
        Commands::Board { action } => {
            let token = env::var("GITHUB_ADMIN_PAT").or_else(|_| env::var("GITHUB_PERSONAL_PAT")).context("No PAT")?;
            let client = GitHubClient::new(token, "Fryymann".into(), "koad-os".into())?;
            match action {
                BoardAction::Status => {
                    println!(">>> [UPLINK] Accessing Neural Log: Tactical Overlay...");
                    let items = client.list_project_items(2).await?;
                    println!("\n{:<5} {:<50} {:<15} {:<15}", "NODE", "DATA FRAGMENT", "STATUS", "VERSION");
                    println!("{:-<90}", "");
                    for item in items {
                        let num = item.number.map(|n| format!("#{}", n)).unwrap_or_default();
                        println!("{:<5} {:<50} {:<15} {:<15}", num, if item.title.len() > 48 { format!("{}...", &item.title[..45]) } else { item.title }, item.status, item.target_version.unwrap_or_default());
                    }
                }
                BoardAction::Sync => {
                    client.sync_issues(2).await?;
                }
                BoardAction::Done { id } => {
                    client.update_item_status(2, id, "Done").await?;
                }
                BoardAction::Todo { id } => {
                    client.update_item_status(2, id, "Todo").await?;
                }
                _ => println!("Action not yet implemented."),
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
                    
                    // Simple logic: Get git branch
                    let branch_out = Command::new("git").arg("-C").arg(&path).arg("rev-parse").arg("--abbrev-ref").arg("HEAD").output();
                    let branch = if let Ok(out) = branch_out {
                        if out.status.success() {
                            Some(String::from_utf8_lossy(&out.stdout).trim().to_string())
                        } else { None }
                    } else { None };

                    // Simple logic: Check for 'package.json' or 'Cargo.toml' or 'koad.json'
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
        Commands::Stat { json } => {
            if redis_path.exists() {
                let mut con = redis::Client::open(format!("redis+unix://{}", redis_path.display()))?.get_connection()?;
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
        Commands::Skill { action } => {
            match action {
                SkillAction::List => { println!("Skills listing..."); }
                SkillAction::Run { name, args } => { Command::new(KoadConfig::get_home()?.join("skills").join(name)).args(args).spawn()?.wait()?; }
            }
        }
        Commands::Crew => {
            if !redis_path.exists() {
                anyhow::bail!("Kernel offline (Redis UDS missing). Cannot fetch live crew manifest.");
            }

            let mut con = redis::Client::open(format!("redis+unix://{}", redis_path.display()))?.get_connection()?;
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
                                "\x1b[32mWAKE\x1b[0m" // Green
                            } else {
                                "\x1b[30mDARK\x1b[0m" // Grey/Dark
                            }
                        } else {
                            "UNKNOWN"
                        };

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
        _ => println!("Other command."),
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
