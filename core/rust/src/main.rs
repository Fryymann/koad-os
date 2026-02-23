use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use std::process::Command;
use chrono::{Local, Duration};
use std::io::{BufRead, BufReader, Write};
use rusqlite::{params, Connection};

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
#[command(version = "2.4.0")]
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
        #[arg(short, long, default_value = "gemini")]
        agent: String,
        #[arg(short, long)]
        project: bool,
        #[arg(short, long)]
        task: Option<String>,
        #[arg(short, long)]
        compact: bool,
    },
    Auth,
    Query { term: String },
    Remember {
        #[command(subcommand)]
        category: MemoryCategory,
    },
    /// Record a personal reflection or interpretation (Persona Journaling).
    Ponder {
        text: String,
        #[arg(short, long)]
        tags: Option<String>,
    },
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
    Init {
        #[arg(short, long)]
        force: bool,
    },
    Harvest {
        #[arg(short, long)]
        path: Option<PathBuf>,
        #[arg(short, long)]
        git: bool,
    },
    Sync {
        #[command(subcommand)]
        source: SyncSource,
    },
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },
    Gcloud {
        #[command(subcommand)]
        action: GcloudAction,
    },
    Drive {
        #[command(subcommand)]
        action: DriveAction,
    },
    Retire {
        id: i64,
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
    Scan {
        path: Option<PathBuf>,
    },
    Publish {
        #[arg(short, long)]
        message: Option<String>,
    },
}

#[derive(Subcommand)]
enum DriveAction {
    List { #[arg(short, long)] shared: bool },
    Download { id: String, #[arg(short, long)] dest: Option<PathBuf> },
    Sync,
}

#[derive(Subcommand)]
enum GcloudAction {
    List { #[arg(short, long, default_value = "functions")] resource: String },
    Deploy { name: String },
    Logs { name: String, #[arg(short, long, default_value_t = 20)] limit: u32 },
    Audit { #[arg(short, long, default_value = "ops")] project: String },
}

#[derive(Subcommand)]
enum SyncSource {
    Airtable { #[arg(short, long)] schema_only: bool, #[arg(short, long)] base_id: Option<String> },
    Notion { #[arg(short, long)] page_id: Option<String>, #[arg(short, long)] db_id: Option<String> },
    Named { name: String },
}

#[derive(Subcommand)]
enum StreamAction {
    Post {
        topic: String,
        message: String,
        #[arg(short, long, default_value = "Log")]
        msg_type: String,
    },
    List {
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum MemoryCategory {
    Fact { text: String, #[arg(short, long)] tags: Option<String> },
    Learning { text: String, #[arg(short, long)] tags: Option<String> },
}

#[derive(Subcommand)]
enum SkillAction {
    List,
    Run { name: String, #[arg(last = true)] args: Vec<String> },
}

#[derive(Subcommand)]
enum TemplateAction {
    List,
    Use { name: String, #[arg(short, long)] out: Option<PathBuf> },
}

impl KoadConfig {
    pub fn get_home() -> Result<PathBuf> {
        env::var("KOAD_HOME")
            .map(PathBuf::from)
            .or_else(|_| dirs::home_dir().context("Home dir not found").map(|h| h.join(".koad-os")))
    }
    pub fn get_path() -> Result<PathBuf> { Ok(Self::get_home()?.join("koad.json")) }
    pub fn get_db_path() -> Result<PathBuf> { Ok(Self::get_home()?.join("koad.db")) }
    pub fn get_log_path() -> Result<PathBuf> { Ok(Self::get_home()?.join("SESSION_LOG.md")) }

    pub fn load() -> Result<Self> {
        let path = Self::get_path()?;
        if !path.exists() { return Ok(Self::default_initial()); }
        let content = std::fs::read_to_string(path)?;
        let mut cfg: Self = serde_json::from_str(&content).context("Failed to parse koad.json")?;
        if let Ok(val) = env::var("KOAD_NAME") { cfg.identity.name = val; }
        if let Ok(val) = env::var("KOAD_ROLE") { cfg.identity.role = val; }
        if let Ok(val) = env::var("KOAD_BIO") { cfg.identity.bio = val; }
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path()?;
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content).context("Failed to write koad.json")
    }

    pub fn default_initial() -> Self {
        Self {
            version: "2.4".to_string(),
            identity: Identity {
                name: env::var("KOAD_NAME").unwrap_or_else(|_| "Koad".into()),
                role: env::var("KOAD_ROLE").unwrap_or_else(|_| "Admin".into()),
                bio: env::var("KOAD_BIO").unwrap_or_else(|_| "Principal Systems & Operations Engineer; Agnostic AI framework.".into()),
            },
            preferences: Preferences {
                languages: vec!["Rust".into(), "Node.js".into(), "Python".into()],
                style: "programmatic-first".to_string(),
                principles: vec![
                    "Simplicity first".into(), 
                    "Plan before build".into(),
                    "Sanctuary Rule".into()
                ],
            },
            drivers: HashMap::new(),
            notion: NotionConfig::default(),
        }
    }
}

struct KoadDB {
    conn: Connection,
}

impl KoadDB {
    fn init() -> Result<Self> {
        let path = KoadConfig::get_db_path()?;
        let conn = Connection::open(path)?;
        conn.execute("CREATE TABLE IF NOT EXISTS knowledge (id INTEGER PRIMARY KEY, category TEXT NOT NULL, content TEXT NOT NULL, tags TEXT, timestamp TEXT NOT NULL, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS projects (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, path TEXT NOT NULL, role TEXT, stack TEXT, last_boot TEXT, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY, command TEXT NOT NULL, args TEXT, timestamp TEXT NOT NULL, status TEXT)", [])?;
        Ok(Self { conn })
    }

    fn remember(&self, category: &str, content: &str, tags: Option<String>) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute("INSERT INTO knowledge (category, content, tags, timestamp) VALUES (?1, ?2, ?3, ?4)", params![category, content, tags, timestamp])?;
        Ok(())
    }

    fn get_ponderings(&self, limit: usize) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare("SELECT content FROM knowledge WHERE category = 'pondering' AND active = 1 ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit], |row| Ok(row.get(0)?))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    fn log_execution(&self, command: &str, args: &str, status: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute("INSERT INTO executions (command, args, timestamp, status) VALUES (?1, ?2, ?3, ?4)", params![command, args, timestamp, status])?;
        Ok(())
    }

    fn get_recent_executions(&self, hours: i64) -> Result<Vec<(String, String)>> {
        let cutoff = (Local::now() - Duration::hours(hours)).format("%Y-%m-%d %H:%M:%S").to_string();
        let mut stmt = self.conn.prepare("SELECT command, args FROM executions WHERE timestamp > ?1 AND status = 'success' GROUP BY command, args")?;
        let rows = stmt.query_map(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    fn register_project(&self, name: &str, path: &str) -> Result<()> {
        let last_boot = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute("INSERT INTO projects (name, path, last_boot) VALUES (?1, ?2, ?3) ON CONFLICT(name) DO UPDATE SET last_boot=?3, path=?2", params![name, path, last_boot])?;
        Ok(())
    }

    fn retire(&self, id: i64) -> Result<()> {
        self.conn.execute("UPDATE knowledge SET active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn query(&self, term: &str) -> Result<Vec<(i64, String, String, String)>> {
        let mut stmt = self.conn.prepare("SELECT id, category, content, timestamp FROM knowledge WHERE active = 1 AND (content LIKE ?1 OR tags LIKE ?1) ORDER BY timestamp DESC LIMIT 20")?;
        let rows = stmt.query_map(params![format!("%{}%", term)], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    fn get_contextual(&self, limit: usize, tags: Vec<String>) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();
        if !tags.is_empty() {
            let mut query = String::from("SELECT category, content FROM knowledge WHERE active = 1 AND category != 'pondering' AND (");
            for (i, _) in tags.iter().enumerate() {
                if i > 0 { query.push_str(" OR "); }
                query.push_str("tags LIKE ?"); query.push_str(&(i + 1).to_string());
                query.push_str(" OR content LIKE ?"); query.push_str(&(i + 1).to_string());
            }
            query.push_str(") ORDER BY timestamp DESC LIMIT ?");
            query.push_str(&(tags.len() + 1).to_string());
            let mut stmt = self.conn.prepare(&query)?;
            let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
            for t in &tags { params_vec.push(format!("%{}%", t).into()); }
            params_vec.push(((limit / 2) as i64).into());
            let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| Ok((row.get(0)?, row.get(1)?)))?;
            for row in rows { results.push(row?); }
        }
        let mut stmt = self.conn.prepare("SELECT category, content FROM knowledge WHERE active = 1 AND category != 'pondering' ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit - results.len()], |row| Ok((row.get(0)?, row.get(1)?)))?;
        for row in rows { 
            let item = row?;
            if !results.contains(&item) { results.push(item); }
        }
        Ok(results)
    }

    fn get_active_project(&self, current_path: &str) -> Result<Option<(String, Option<String>, Option<String>)>> {
        let mut stmt = self.conn.prepare("SELECT name, role, stack FROM projects WHERE path = ?1 AND active = 1")?;
        let mut rows = stmt.query(params![current_path])?;
        if let Some(row) = rows.next()? { Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?))) } else { Ok(None) }
    }
}

fn get_gh_pat_for_path(path: &Path) -> (&'static str, &'static str) {
    let path_str = path.to_string_lossy();
    if path_str.contains("skylinks") { ("GITHUB_SKYLINKS_PAT", "Work") } else { ("GITHUB_PERSONAL_PAT", "Personal") }
}

fn get_gdrive_token_for_path(path: &Path) -> (&'static str, &'static str) {
    let path_str = path.to_string_lossy();
    if path_str.contains("skylinks") { ("GDRIVE_SKYLINKS_TOKEN", "Work") } else { ("GDRIVE_PERSONAL_TOKEN", "Personal") }
}

fn detect_context_tags(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("skylinks") { tags.push("skylinks".into()); }
    if path_str.contains("ttrpg") { tags.push("ttrpg".into()); }
    if path_str.contains("rust") || path.join("Cargo.toml").exists() { tags.push("rust".into()); }
    if path_str.contains("node") || path.join("package.json").exists() { tags.push("node".into()); }
    tags
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = KoadConfig::load()?;
    let db = KoadDB::init()?;
    let role = cli.role.to_lowercase();
    let is_admin = role == "admin";
    let is_pm = role == "pm";
    let has_privileged_access = is_admin || is_pm;

    match cli.command {
        Commands::Boot { agent: _, project, task: _, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let current_path_str = current_dir.to_string_lossy().to_string();
            let (pat_var, _) = get_gh_pat_for_path(&current_dir);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);

            if compact {
                println!("I:{}|R:{}|G:{}|D:{}|T:{}", config.identity.name, config.identity.role, pat_var, drive_var, tags.join(","));
            } else {
                println!("<koad_boot>");
                println!("Identity: {} ({})", config.identity.name, config.identity.role);
                println!("Auth: GH={} | GD={}", pat_var, drive_var);
                
                if let Some((p_name, _, _)) = db.get_active_project(&current_path_str)? {
                    println!("\n[Active Project: {}]", p_name);
                }

                println!("\n[Persona Reflections]");
                let ponders = db.get_ponderings(3)?;
                if ponders.is_empty() { println!("- No active reflections."); }
                for p in ponders { println!("- {}", p); }

                println!("\n[Contextual Memory]");
                for (cat, content) in db.get_contextual(8, tags)? { println!("- [{}] {}", cat, content); }

                if project {
                    let progress_path = current_dir.join("PROJECT_PROGRESS.md");
                    if progress_path.exists() {
                        let p = std::fs::read_to_string(progress_path)?;
                        if let Some(s) = p.find("## Snapshot") { println!("\n[Project Progress]\n{}", p[s..].trim()); }
                    }
                }
                println!("</koad_boot>");
            }
        }
        Commands::Auth => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (p, _) = get_gh_pat_for_path(&current_dir);
            let (d, _) = get_gdrive_token_for_path(&current_dir);
            println!("GH:{} | GD:{}", p, d);
        }
        Commands::Query { term } => {
            for (id, cat, content, ts) in db.query(&term)? { println!("- ID:{} [{}] ({}) {}", id, cat, ts, content); }
        }
        Commands::Remember { category } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            match category {
                MemoryCategory::Fact { text, tags } => db.remember("fact", &text, tags)?,
                MemoryCategory::Learning { text, tags } => db.remember("learning", &text, tags)?,
            }
            println!("Memory updated.");
        }
        Commands::Ponder { text, tags } => {
            let p_tags = format!("persona-journal,{}", tags.unwrap_or_default());
            db.remember("pondering", &text, Some(p_tags))?;
            println!("Reflection recorded in persona journal.");
        }
        Commands::Skill { action } => {
             let skills_dir = KoadConfig::get_home()?.join("skills");
             match action {
                 SkillAction::List => {
                     for entry in std::fs::read_dir(&skills_dir)? {
                         let entry = entry?;
                         if entry.path().is_dir() {
                             let cat = entry.file_name().to_string_lossy().to_string();
                             for s in std::fs::read_dir(entry.path())? { println!("- {}/{}", cat, s?.file_name().to_string_lossy()); }
                         }
                     }
                 },
                 SkillAction::Run { name, args } => {
                     let mut child = Command::new(skills_dir.join(&name)).args(&args).spawn()?;
                     let status = child.wait()?;
                     db.log_execution("skill run", &name, if status.success() { "success" } else { "failed" })?;
                 }
             }
        }
        Commands::Template { action } => {
            let t_dir = KoadConfig::get_home()?.join("templates/project_flow");
            match action {
                TemplateAction::List => { if t_dir.exists() { for e in std::fs::read_dir(t_dir)? { println!("- {}", e?.file_name().to_string_lossy()); } } }
                TemplateAction::Use { name, out } => {
                    let s = t_dir.join(&name);
                    let d = out.unwrap_or_else(|| PathBuf::from(&name));
                    std::fs::copy(s, d)?; println!("Template applied.");
                }
            }
        }
        Commands::Init { force } => {
            if !is_admin { anyhow::bail!("Admin only."); }
            let p = KoadConfig::get_path()?;
            if p.exists() && !force { anyhow::bail!("Exists."); }
            KoadConfig::default_initial().save()?; println!("Initialized.");
        }
        Commands::Harvest { path, git } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            if git { println!("Scanning git history..."); }
            if let Some(p) = path {
                let r = BufReader::new(std::fs::File::open(&p)?);
                let mut d = false;
                for l in r.lines() {
                    let l = l?;
                    if l.starts_with("## Discoveries") { d = true; continue; }
                    if l.starts_with("## ") && d { break; }
                    if d && l.trim().starts_with("- ") { db.remember("learning", &l.trim()[2..], None)?; }
                }
            }
        }
        Commands::Sync { source } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            match source {
                SyncSource::Airtable { schema_only, base_id } => {
                    let mut args = vec!["skill".to_string(), "run".to_string(), "global/airtable_sync.py".to_string(), "--".to_string()];
                    if schema_only { args.push("--schema-only".to_string()); }
                    if let Some(id) = base_id { args.push("--base-id".to_string()); args.push(id); }
                    Command::new(env::current_exe()?).args(args).spawn()?.wait()?;
                }
                SyncSource::Notion { page_id, db_id } => {
                    let mut args = vec!["skill".to_string(), "run".to_string(), "global/notion_sync.py".to_string(), "--".to_string()];
                    if let Some(id) = page_id { args.push("--page-id".to_string()); args.push(id); }
                    if let Some(id) = db_id { args.push("--db-id".to_string()); args.push(id); }
                    Command::new(env::current_exe()?).args(args).spawn()?.wait()?;
                }
                SyncSource::Named { name } => {
                    let id = config.notion.index.get(&name).ok_or_else(|| anyhow::anyhow!("Not found"))?;
                    Command::new(env::current_exe()?).args(vec!["skill", "run", "global/notion_sync.py", "--", "--page-id", id]).spawn()?.wait()?;
                }
            }
        }
        Commands::Stream { action } => {
            let s_path = KoadConfig::get_home()?.join("skills/global/notion_stream.py");
            match action {
                StreamAction::Post { topic, message, msg_type } => {
                    Command::new(s_path).arg("post").arg(topic).arg(message).arg("--type").arg(msg_type).spawn()?.wait()?;
                }
                StreamAction::List { limit } => {
                    Command::new(s_path).arg("list").arg("--limit").arg(limit.to_string()).spawn()?.wait()?;
                }
            }
        }
        Commands::Gcloud { action } => {
            let mut args = vec!["skill".to_string(), "run".to_string(), "global/gcloud_ops.py".to_string(), "--".to_string()];
            match action {
                GcloudAction::List { ref resource } => { args.push("list".to_string()); args.push("--resource".to_string()); args.push(resource.clone()); }
                GcloudAction::Deploy { ref name } => { if !is_admin { anyhow::bail!("Restricted."); } args.push("deploy".to_string()); args.push("--name".to_string()); args.push(name.clone()); }
                GcloudAction::Logs { ref name, limit } => { args.push("logs".to_string()); args.push("--name".to_string()); args.push(name.clone()); args.push("--limit".to_string()); args.push(limit.to_string()); }
                GcloudAction::Audit { ref project } => { if !is_admin { anyhow::bail!("Restricted."); } args.push("audit".to_string()); args.push("--project".to_string()); args.push(project.clone()); }
            }
            Command::new(env::current_exe()?).args(args).spawn()?.wait()?;
        }
        Commands::Drive { action } => {
            let mut args = vec!["skill".to_string(), "run".to_string(), "global/gdrive_ops.py".to_string(), "--".to_string()];
            match action {
                DriveAction::List { shared } => { args.push("list".to_string()); if shared { args.push("--shared".to_string()); } }
                DriveAction::Download { id, dest } => { args.push("download".to_string()); args.push("--id".to_string()); args.push(id); if let Some(d) = dest { args.push("--dest".to_string()); args.push(d.to_string_lossy().to_string()); } }
                DriveAction::Sync => { if !has_privileged_access { anyhow::bail!("Restricted."); } args.push("sync".to_string()); }
            }
            Command::new(env::current_exe()?).args(args).spawn()?.wait()?;
        }
        Commands::Retire { id } => { if !is_admin { anyhow::bail!("Restricted."); } db.retire(id)?; }
        Commands::Saveup { summary, scope, facts, auto } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            if let Some(f) = facts { for item in f.split(',') { db.remember("fact", item.trim(), Some(scope.clone()))?; } }
            if auto {
                let recent = db.get_recent_executions(4)?;
                for (cmd, args) in recent { db.remember("fact", &format!("Verified {} with: {}", cmd, args), Some(format!("auto,{}", scope)))?; }
            }
            let log_entry = format!("\n## {} - {}\n- Scope: {}\n- Project: {}\n", Local::now().format("%Y-%m-%d"), summary, scope, db.get_active_project(&env::current_dir()?.to_string_lossy())?.map(|(n,_,_)| n).unwrap_or("General".into()));
            std::fs::OpenOptions::new().append(true).create(true).open(KoadConfig::get_log_path()?)?.write_all(log_entry.as_bytes())?;
            println!("Saveup complete.");
        }
        Commands::Scan { path } => {
            let t = path.unwrap_or_else(|| env::current_dir().unwrap_or(PathBuf::from(".")));
            if t.join(".koad").exists() {
                db.register_project(&t.file_name().unwrap().to_string_lossy(), &t.to_string_lossy())?;
                println!("Project registered.");
            }
        }
        Commands::Publish { message } => {
            if !is_admin { anyhow::bail!("Admin only."); }
            let h = KoadConfig::get_home()?;
            let m = message.unwrap_or_else(|| format!("KoadOS Sync - {}", Local::now().format("%Y-%m-%d %H:%M")));
            Command::new("git").arg("-C").arg(&h).arg("add").arg(".").spawn()?.wait()?;
            Command::new("git").arg("-C").arg(&h).arg("commit").arg("-m").arg(&m).spawn()?.wait()?;
            Command::new("git").arg("-C").arg(&h).arg("push").arg("origin").arg("main").spawn()?.wait()?;
            println!("Published.");
        }
    }
    Ok(())
}
