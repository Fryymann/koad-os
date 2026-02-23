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

/// The central configuration for KoadOS.
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
#[command(version = "2.3.3")]
#[command(about = "The KoadOS Control Plane")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Set the active role for the session (admin | pm | developer).
    #[arg(short, long, global = true, default_value = "admin")]
    role: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Boot koadOS.
    Boot {
        #[arg(short, long, default_value = "gemini")]
        agent: String,
        #[arg(short, long)]
        project: bool,
        /// Optional task ID (e.g., S1-01) to tune model selection.
        #[arg(short, long)]
        task: Option<String>,
        /// Output a token-efficient compact block.
        #[arg(short, long)]
        compact: bool,
    },
    Auth,
    Query { term: String },
    Remember {
        #[command(subcommand)]
        category: MemoryCategory,
    },
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// Manage project workflow templates.
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
    Init {
        #[arg(short, long)]
        force: bool,
    },
    /// Harvest discoveries from documentation or git history.
    Harvest {
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Scan git diff for changes.
        #[arg(short, long)]
        git: bool,
    },
    Sync {
        #[command(subcommand)]
        source: SyncSource,
    },
    /// Forum-style communication stream with Notion/Noti.
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
    /// Native Session Saveup (Token Efficient).
    Saveup {
        /// Short summary of work.
        summary: String,
        /// Project scope.
        #[arg(short, long, default_value = "General")]
        scope: String,
        /// Optional facts to add (comma-separated).
        #[arg(short, long)]
        facts: Option<String>,
        /// Automatically harvest facts from recent successful executions.
        #[arg(short, long)]
        auto: bool,
    },
    /// Native project scanner (database-aware).
    Scan {
        path: Option<PathBuf>,
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
    /// Run an IAM audit on a project.
    Audit { #[arg(short, long, default_value = "ops")] project: String },
}

#[derive(Subcommand)]
enum SyncSource {
    Airtable { #[arg(short, long)] schema_only: bool, #[arg(short, long)] base_id: Option<String> },
    Notion { #[arg(short, long)] page_id: Option<String>, #[arg(short, long)] db_id: Option<String> },
    /// Sync a page or database by name from the Koad index.
    Named { name: String },
}

#[derive(Subcommand)]
enum StreamAction {
    /// Post a new message to the stream.
    Post {
        topic: String,
        message: String,
        #[arg(short, long, default_value = "Log")]
        msg_type: String,
    },
    /// List recent messages.
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
    /// List available workflow templates.
    List,
    /// Copy a template to the current directory.
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
            version: "2.3".to_string(),
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
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT,
                timestamp TEXT NOT NULL,
                active INTEGER DEFAULT 1
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                path TEXT NOT NULL,
                role TEXT,
                stack TEXT,
                last_boot TEXT,
                active INTEGER DEFAULT 1
            )",
            [],
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS executions (
                id INTEGER PRIMARY KEY,
                command TEXT NOT NULL,
                args TEXT,
                timestamp TEXT NOT NULL,
                status TEXT
            )",
            [],
        )?;

        Ok(Self { conn })
    }

    fn remember(&self, category: &str, content: &str, tags: Option<String>) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute(
            "INSERT INTO knowledge (category, content, tags, timestamp) VALUES (?1, ?2, ?3, ?4)",
            params![category, content, tags, timestamp],
        )?;
        Ok(())
    }

    fn log_execution(&self, command: &str, args: &str, status: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute(
            "INSERT INTO executions (command, args, timestamp, status) VALUES (?1, ?2, ?3, ?4)",
            params![command, args, timestamp, status],
        )?;
        Ok(())
    }

    fn get_recent_executions(&self, hours: i64) -> Result<Vec<(String, String)>> {
        let cutoff = (Local::now() - Duration::hours(hours)).format("%Y-%m-%d %H:%M:%S").to_string();
        let mut stmt = self.conn.prepare(
            "SELECT command, args FROM executions 
             WHERE timestamp > ?1 AND status = 'success' 
             GROUP BY command, args"
        )?;
        let rows = stmt.query_map(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    fn register_project(&self, name: &str, path: &str) -> Result<()> {
        let last_boot = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.conn.execute(
            "INSERT INTO projects (name, path, last_boot) VALUES (?1, ?2, ?3)
             ON CONFLICT(name) DO UPDATE SET last_boot=?3, path=?2",
            params![name, path, last_boot],
        )?;
        Ok(())
    }

    fn retire(&self, id: i64) -> Result<()> {
        self.conn.execute("UPDATE knowledge SET active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    fn query(&self, term: &str) -> Result<Vec<(i64, String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, category, content, timestamp FROM knowledge 
             WHERE active = 1 AND (content LIKE ?1 OR tags LIKE ?1) 
             ORDER BY timestamp DESC LIMIT 20"
        )?;
        let rows = stmt.query_map(params![format!("%{}%", term)], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    fn get_contextual(&self, limit: usize, tags: Vec<String>) -> Result<Vec<(String, String)>> {
        let mut results = Vec::new();
        if !tags.is_empty() {
            let mut query = String::from("SELECT category, content FROM knowledge WHERE active = 1 AND (");
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
        let mut stmt = self.conn.prepare("SELECT category, content FROM knowledge WHERE active = 1 ORDER BY timestamp DESC LIMIT ?1")?;
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
        if let Some(row) = rows.next()? {
            Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?)))
        } else {
            Ok(None)
        }
    }
}

fn get_gh_pat_for_path(path: &Path) -> (&'static str, &'static str) {
    let path_str = path.to_string_lossy();
    if path_str.contains("skylinks") { ("GITHUB_SKYLINKS_PAT", "Work") } 
    else { ("GITHUB_PERSONAL_PAT", "Personal") }
}

fn get_gdrive_token_for_path(path: &Path) -> (&'static str, &'static str) {
    let path_str = path.to_string_lossy();
    if path_str.contains("skylinks") { ("GDRIVE_SKYLINKS_TOKEN", "Work") } 
    else { ("GDRIVE_PERSONAL_TOKEN", "Personal") }
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

fn get_task_recommendation(task_id: &str) -> Option<(String, HashMap<String, String>)> {
    let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let possible_paths = vec![
        current_dir.join(format!("{}.md", task_id)),
        current_dir.join("tasks").join(format!("{}.md", task_id)),
        current_dir.join("docs").join("tasks").join(format!("{}.md", task_id)),
    ];

    for path in possible_paths {
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                let mut ratings = HashMap::new();
                let mut in_ratings = false;
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.to_lowercase().contains("ratings") { in_ratings = true; continue; }
                    if in_ratings && trimmed.starts_with("##") { break; }
                    if in_ratings && trimmed.starts_with("- ") && trimmed.contains(':') {
                        let parts: Vec<&str> = trimmed[2..].split(':').collect();
                        if parts.len() == 2 {
                            ratings.insert(parts[0].trim().to_string(), parts[1].trim().to_string());
                        }
                    }
                }
                
                let reasoning = ratings.get("Reasoning").or_else(|| ratings.get("reasoning")).cloned().unwrap_or_default();
                let deep_thinking = ratings.get("Deep Thinking").or_else(|| ratings.get("deep_thinking")).cloned().unwrap_or_default();
                
                let is_high = reasoning.starts_with('4') || reasoning.starts_with('5') || 
                             deep_thinking.starts_with('4') || deep_thinking.starts_with('5');
                
                let model = if is_high {
                    "gemini-2.0-flash-thinking-exp-01-21"
                } else {
                    "gemini-2.0-flash"
                };

                return Some((model.to_string(), ratings));
            }
        }
    }
    None
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
        Commands::Boot { agent: _, project, task, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let current_path_str = current_dir.to_string_lossy().to_string();
            let (pat_var, pat_desc) = get_gh_pat_for_path(&current_dir);
            let (drive_var, drive_desc) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);
            let task_info = task.as_ref().and_then(|t| get_task_recommendation(t));

            if compact {
                print!("I:{}|R:{}|", config.identity.name, config.identity.role);
                print!("G:{}|D:{}|", pat_var, drive_var);
                print!("T:{}|", tags.join(","));
                if let Some((model, _)) = task_info { print!("M:{}|", model); }
                println!("\n[M]");
                for (cat, content) in db.get_contextual(8, tags)? {
                    println!("{}:{}", if cat == "fact" { "F" } else { "L" }, content);
                }
            } else {
                println!("<koad_boot>");
                println!("Identity: {} ({})", config.identity.name, config.identity.role);
                println!("Auth: GH={} ({}) | GD={} ({})", pat_var, pat_desc, drive_var, drive_desc);
                println!("Context Tags: {}", tags.join(", "));
                
                if let Some((model, ratings)) = task_info {
                    println!("\n[Task Context: {}]", task.unwrap());
                    println!("Recommended Model: {}", model);
                    for (k, v) in ratings { println!("- {}: {}", k, v); }
                }

                if let Some((p_name, p_role, p_stack)) = db.get_active_project(&current_path_str)? {
                    println!("\n[Active Project: {}]", p_name);
                    if let Some(r) = p_role { println!("Role: {}", r); }
                    if let Some(s) = p_stack { println!("Stack: {}", s); }
                }

                println!("\n[Contextual Memory]");
                for (cat, content) in db.get_contextual(12, tags)? {
                    println!("- [{}] {}", cat, content);
                }
                if project {
                    let progress_path = current_dir.join("PROJECT_PROGRESS.md");
                    if progress_path.exists() {
                        let progress = std::fs::read_to_string(progress_path)?;
                        if let Some(start) = progress.find("## Snapshot") {
                            let end = progress.find("## Roadmap Alignment").unwrap_or(progress.len());
                            println!("\n[Project Progress (Legacy)]\n{}", progress[start..end].trim());
                        }
                    }
                }
                println!("</koad_boot>");
            }
        }
        Commands::Auth => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (pat_var, pat_desc) = get_gh_pat_for_path(&current_dir);
            let (drive_var, drive_desc) = get_gdrive_token_for_path(&current_dir);
            println!("GH:{} ({}) | GD:{} ({})", pat_var, pat_desc, drive_var, drive_desc);
        }
        Commands::Query { term } => {
            for (id, cat, content, ts) in db.query(&term)? {
                println!("- ID:{} [{}] ({}) {}", id, cat, ts, content);
            }
        }
        Commands::Remember { category } => {
            if !has_privileged_access { anyhow::bail!("Access Denied: Sanctuary Rule."); }
            match category {
                MemoryCategory::Fact { text, tags } => db.remember("fact", &text, tags)?,
                MemoryCategory::Learning { text, tags } => db.remember("learning", &text, tags)?,
            }
            println!("Memory updated.");
        }
        Commands::Skill { action } => {
             let base = KoadConfig::get_home()?;
             let skills_dir = base.join("skills");
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
                     db.log_execution("skill run", &format!("{}: {:?}", name, args), "pending")?;
                     let mut child = Command::new(skills_dir.join(&name)).args(&args).spawn()?;
                     let status = child.wait()?;
                     db.log_execution("skill run", &name, if status.success() { "success" } else { "failed" })?;
                 }
             }
        }
        Commands::Template { action } => {
            let base = KoadConfig::get_home()?;
            let template_dir = base.join("templates/project_flow");
            match action {
                TemplateAction::List => {
                    println!("Available Workflow Templates:");
                    if template_dir.exists() {
                        for entry in std::fs::read_dir(template_dir)? {
                            let entry = entry?;
                            if entry.path().is_file() {
                                println!("- {}", entry.file_name().to_string_lossy());
                            }
                        }
                    }
                }
                TemplateAction::Use { name, out } => {
                    let source = template_dir.join(&name);
                    if !source.exists() {
                        let source_md = template_dir.join(format!("{}.md", name));
                        if source_md.exists() {
                            let dest = out.unwrap_or_else(|| PathBuf::from(format!("{}.md", name)));
                            std::fs::copy(source_md, dest)?;
                            println!("Template {} applied.", name);
                        } else {
                            anyhow::bail!("Template {} not found.", name);
                        }
                    } else {
                        let dest = out.unwrap_or_else(|| PathBuf::from(&name));
                        std::fs::copy(source, dest)?;
                        println!("Template {} applied.", name);
                    }
                }
            }
        }
        Commands::Init { force } => {
            if !is_admin { anyhow::bail!("Admin only."); }
            let path = KoadConfig::get_path()?;
            if path.exists() && !force { anyhow::bail!("Exists."); }
            KoadConfig::default_initial().save()?; println!("Initialized.");
        }
        Commands::Harvest { path, git } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            if git { println!("Scanning git history for discoveries..."); }
            if let Some(p) = path {
                let file = std::fs::File::open(&p)?;
                let reader = BufReader::new(file);
                let mut in_discovery = false;
                let mut count = 0;
                for line in reader.lines() {
                    let line = line?;
                    if line.starts_with("## Discoveries") || line.starts_with("## Learnings") { in_discovery = true; continue; }
                    if line.starts_with("## ") && in_discovery { break; }
                    if in_discovery && line.trim().starts_with("- ") {
                        db.remember("learning", &line.trim()[2..], None)?;
                        count += 1;
                    }
                }
                if count > 0 { println!("Harvested {} learnings.", count); }
            }
        }
        Commands::Sync { source } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            match source {
                SyncSource::Airtable { schema_only, base_id } => {
                    let mut cmd_args = vec!["skill".to_string(), "run".to_string(), "global/airtable_sync.py".to_string(), "--".to_string()];
                    if schema_only { cmd_args.push("--schema-only".to_string()); }
                    if let Some(id) = base_id { cmd_args.push("--base-id".to_string()); cmd_args.push(id); }
                    let mut child = Command::new(env::current_exe()?).args(cmd_args).spawn()?; child.wait()?;
                }
                SyncSource::Notion { page_id, db_id } => {
                    let mut cmd_args = vec!["skill".to_string(), "run".to_string(), "global/notion_sync.py".to_string(), "--".to_string()];
                    if let Some(id) = page_id { cmd_args.push("--page-id".to_string()); cmd_args.push(id); }
                    if let Some(id) = db_id { cmd_args.push("--db-id".to_string()); cmd_args.push(id); }
                    let mut child = Command::new(env::current_exe()?).args(cmd_args).spawn()?; child.wait()?;
                }
                SyncSource::Named { name } => {
                    let id = config.notion.index.get(&name).ok_or_else(|| anyhow::anyhow!("Name '{}' not found in Notion index", name))?;
                    let cmd_args = vec!["skill".to_string(), "run".to_string(), "global/notion_sync.py".to_string(), "--".to_string(), "--page-id".to_string(), id.clone()];
                    let mut child = Command::new(env::current_exe()?).args(cmd_args).spawn()?; child.wait()?;
                }
            }
        }
        Commands::Stream { action } => {
            let base = KoadConfig::get_home()?;
            let skill_path = base.join("skills/global/notion_stream.py");
            match action {
                StreamAction::Post { topic, message, msg_type } => {
                    db.log_execution("stream post", &topic, "pending")?;
                    let mut child = Command::new(skill_path).arg("post").arg(topic).arg(message).arg("--type").arg(msg_type).spawn()?;
                    let status = child.wait()?;
                    db.log_execution("stream post", "", if status.success() { "success" } else { "failed" })?;
                }
                StreamAction::List { limit } => {
                    let mut child = Command::new(skill_path).arg("list").arg("--limit").arg(limit.to_string()).spawn()?;
                    child.wait()?;
                }
            }
        }
        Commands::Gcloud { action } => {
            let mut cmd_args = vec!["skill".to_string(), "run".to_string(), "global/gcloud_ops.py".to_string(), "--".to_string()];
            let cmd_name = match action {
                GcloudAction::List { ref resource } => { cmd_args.push("list".to_string()); cmd_args.push("--resource".to_string()); cmd_args.push(resource.clone()); "gcloud list" }
                GcloudAction::Deploy { ref name } => { if !is_admin { anyhow::bail!("Deploy restricted."); } cmd_args.push("deploy".to_string()); cmd_args.push("--name".to_string()); cmd_args.push(name.clone()); "gcloud deploy" }
                GcloudAction::Logs { ref name, limit } => { cmd_args.push("logs".to_string()); cmd_args.push("--name".to_string()); cmd_args.push(name.clone()); cmd_args.push("--limit".to_string()); cmd_args.push(limit.to_string()); "gcloud logs" }
                GcloudAction::Audit { ref project } => { if !is_admin { anyhow::bail!("Audit restricted."); } cmd_args.push("audit".to_string()); cmd_args.push("--project".to_string()); cmd_args.push(project.clone()); "gcloud audit" }
            };
            db.log_execution(cmd_name, "", "pending")?;
            let mut child = Command::new(env::current_exe()?).args(cmd_args).spawn()?; 
            let status = child.wait()?;
            db.log_execution(cmd_name, "", if status.success() { "success" } else { "failed" })?;
        }
        Commands::Drive { action } => {
            let mut cmd_args = vec!["skill".to_string(), "run".to_string(), "global/gdrive_ops.py".to_string(), "--".to_string()];
            match action {
                DriveAction::List { shared } => { cmd_args.push("list".to_string()); if shared { cmd_args.push("--shared".to_string()); } }
                DriveAction::Download { id, dest } => { cmd_args.push("download".to_string()); cmd_args.push("--id".to_string()); cmd_args.push(id); if let Some(d) = dest { cmd_args.push("--dest".to_string()); cmd_args.push(d.to_string_lossy().to_string()); } }
                DriveAction::Sync => { if !has_privileged_access { anyhow::bail!("Sync restricted."); } cmd_args.push("sync".to_string()); }
            }
            let mut child = Command::new(env::current_exe()?).args(cmd_args).spawn()?; child.wait()?;
        }
        Commands::Retire { id } => {
            if !is_admin { anyhow::bail!("Admin only."); }
            db.retire(id)?; println!("Knowledge entry {} retired.", id);
        }
        Commands::Saveup { summary, scope, facts, auto } => {
            if !has_privileged_access { anyhow::bail!("Access Denied: Sanctuary Rule."); }
            
            // 1. Process Manual Facts
            if let Some(f_str) = facts {
                for f in f_str.split(',') { 
                    db.remember("fact", f.trim(), Some(scope.clone()))?; 
                    println!("Fact remembered: {}", f.trim());
                }
            }

            // 2. Process Auto-Harvested Facts
            if auto {
                println!("Auto-harvesting facts from recent executions...");
                let recent = db.get_recent_executions(4)?;
                for (cmd, args) in recent {
                    let fact_text = if args.is_empty() {
                        format!("Verified successful execution of {}", cmd)
                    } else {
                        format!("Verified successful {} with: {}", cmd, args)
                    };
                    db.remember("fact", &fact_text, Some(format!("auto,{}", scope)))?;
                    println!("Auto-fact remembered: {}", fact_text);
                }
            }

            // 3. Append to SESSION_LOG.md
            let log_path = KoadConfig::get_log_path()?;
            let date_str = Local::now().format("%Y-%m-%d").to_string();
            
            // Try to find active project for better logging
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let p_info = db.get_active_project(&current_dir.to_string_lossy())?.map(|(n, _, _)| n).unwrap_or_else(|| "General".to_string());

            let log_entry = format!("\n## {} - {} (Project: {})\n- Scope: {}\n- Completed via harvest-aware saveup.\n", date_str, summary, p_info, scope);
            
            let mut file = std::fs::OpenOptions::new().append(true).create(true).open(log_path)?;
            file.write_all(log_entry.as_bytes())?;
            
            println!("Saveup complete. Contextual memory and session logs updated.");
        }
        Commands::Scan { path } => {
            let target = path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));
            let koad_dir = target.join(".koad");
            if koad_dir.exists() {
                let p_name = target.file_name().unwrap_or_default().to_string_lossy().to_string();
                let p_path = target.to_string_lossy().to_string();
                db.register_project(&p_name, &p_path)?;
                println!("Project '{}' registered in database.", p_name);
                db.remember("fact", &format!("Verified Koad project at {:?}", target), Some("scan".to_string()))?;
            } else {
                println!("No koad project found at {:?}", target);
            }
        }
    }

    Ok(())
}
