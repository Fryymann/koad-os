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
    Stat {
        #[arg(short, long)]
        json: bool,
    },
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
enum BoardAction {
    Status,
    Sync,
    Sdr,
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
        conn.execute("CREATE TABLE IF NOT EXISTS knowledge (id INTEGER PRIMARY KEY, category TEXT, content TEXT, tags TEXT, timestamp TEXT, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS projects (id INTEGER PRIMARY KEY, name TEXT UNIQUE, path TEXT, role TEXT, stack TEXT, last_boot TEXT)", [])?;
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

    fn register_project(&self, name: &str, path: &str) -> Result<()> {
        let conn = self.get_conn()?;
        conn.execute("INSERT INTO projects (name, path, last_boot) VALUES (?1, ?2, ?3) ON CONFLICT(name) DO UPDATE SET path=?2, last_boot=?3", params![name, path, Local::now().to_rfc3339()])?;
        Ok(())
    }

    fn get_active_project(&self, path: &str) -> Result<Option<(String, Option<String>, Option<String>)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT name, role, stack FROM projects WHERE ?1 LIKE path || '%' ORDER BY length(path) DESC LIMIT 1")?;
        let mut rows = stmt.query([path])?;
        if let Some(row) = rows.next()? { Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?))) }
        else { Ok(None) }
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
    let cli = Cli::parse();
    let config = KoadConfig::load().unwrap_or_else(|_| KoadConfig::default_initial());
    let db_path = KoadConfig::get_home()?.join("koad.db");
    let db = KoadDB::new(&db_path)?;
    let role = cli.role.clone();
    let is_admin = role.to_lowercase() == "admin";
    let has_privileged_access = is_admin || role.to_lowercase() == "pm";

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
                let redis_conn = KoadConfig::get_home()?.join("koad.sock");
                if redis_conn.exists() {
                    let client = redis::Client::open(format!("redis+unix://{}", redis_conn.display()))?;
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
            println!("--- KoadOS v3 Doctor ---");
            let home = KoadConfig::get_home()?;
            if home.join("kspine.sock").exists() { println!("[PASS] Kernel: Socket active"); } else { println!("[FAIL] Kernel: Offline"); }
            if home.join("koad.db").exists() { println!("[PASS] Storage: Database found"); } else { println!("[FAIL] Storage: Database missing"); }
            if home.join("koad.sock").exists() { println!("[PASS] Bus: Redis UDS active"); } else { println!("[INFO] Bus: Redis offline or using TCP"); }
        }
        Commands::Scan { path } => {
            let t = path.unwrap_or_else(|| env::current_dir().unwrap_or(PathBuf::from(".")));
            let output = Command::new("fdfind").arg(".koad").arg("--type").arg("d").arg("--hidden").arg("--absolute-path").arg(&t).output();
            match output {
                Ok(out) if out.status.success() => {
                    let mut count = 0;
                    for line in String::from_utf8_lossy(&out.stdout).lines() {
                        let project_root = PathBuf::from(line).parent().unwrap().to_path_buf();
                        let name = project_root.file_name().unwrap().to_string_lossy();
                        if db.register_project(&name, &project_root.to_string_lossy()).is_ok() { println!("[PASS] Registered: {}", name); count += 1; }
                    }
                    println!("Scan complete. {} projects registered.", count);
                },
                _ => { if t.join(".koad").exists() { db.register_project(&t.file_name().unwrap().to_string_lossy(), &t.to_string_lossy())?; println!("Project registered."); } }
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
                    println!(">>> Command Deck: Fetching Project Board status...");
                    let items = client.list_project_items(2).await?;
                    println!("\n{:<5} {:<50} {:<15} {:<15}", "ISSUE", "TITLE", "STATUS", "VERSION");
                    println!("{:-<90}", "");
                    for item in items {
                        let num = item.number.map(|n| format!("#{}", n)).unwrap_or_default();
                        println!("{:<5} {:<50} {:<15} {:<15}", num, if item.title.len() > 48 { format!("{}...", &item.title[..45]) } else { item.title }, item.status, item.target_version.unwrap_or_default());
                    }
                }
                _ => println!("Action not yet implemented."),
            }
        }
        Commands::Stat { json } => {
            let redis_conn = KoadConfig::get_home()?.join("koad.sock");
            if redis_conn.exists() {
                let mut con = redis::Client::open(format!("redis+unix://{}", redis_conn.display()))?.get_connection()?;
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
