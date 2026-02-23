use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use std::process::Command;
use chrono::Local;
use std::io::{BufRead, BufReader};
use rusqlite::{params, Connection};

/// The central configuration for KoadOS.
#[derive(Debug, Serialize, Deserialize)]
pub struct KoadConfig {
    pub version: String,
    pub identity: Identity,
    pub preferences: Preferences,
    pub drivers: HashMap<String, DriverConfig>,
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
#[command(version = "2.1.0")]
#[command(about = "The KoadOS Control Plane")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Boot koadOS and output a contextual context block.
    Boot {
        #[arg(short, long, default_value = "gemini")]
        agent: String,
        #[arg(short, long)]
        project: bool,
    },
    Auth,
    Query {
        term: String,
    },
    Remember {
        #[command(subcommand)]
        category: MemoryCategory,
    },
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    Init {
        #[arg(short, long)]
        force: bool,
    },
    Harvest {
        path: PathBuf,
    },
    Sync {
        #[command(subcommand)]
        source: SyncSource,
    },
    Gcloud {
        #[command(subcommand)]
        action: GcloudAction,
    },
}

#[derive(Subcommand)]
enum GcloudAction {
    List { #[arg(short, long)] resource: String },
    Deploy { name: String },
    Logs { name: String, #[arg(short, long, default_value_t = 20)] limit: u32 },
}

#[derive(Subcommand)]
enum SyncSource {
    Airtable { #[arg(short, long)] schema_only: bool, #[arg(short, long)] base_id: Option<String> },
    Notion { #[arg(short, long)] page_id: Option<String>, #[arg(short, long)] db_id: Option<String> },
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

impl KoadConfig {
    pub fn get_home() -> PathBuf {
        env::var("KOAD_HOME").map(PathBuf::from).unwrap_or_else(|_| dirs::home_dir().unwrap().join(".koad-os"))
    }

    pub fn get_path() -> PathBuf { Self::get_home().join("koad.json") }
    pub fn get_db_path() -> PathBuf { Self::get_home().join("koad.db") }

    pub fn load() -> Result<Self> {
        let path = Self::get_path();
        if !path.exists() { return Ok(Self::default_initial()); }
        let content = std::fs::read_to_string(path)?;
        let mut cfg: Self = serde_json::from_str(&content).context("Failed to parse koad.json")?;
        if let Ok(val) = env::var("KOAD_NAME") { cfg.identity.name = val; }
        if let Ok(val) = env::var("KOAD_ROLE") { cfg.identity.role = val; }
        if let Ok(val) = env::var("KOAD_BIO") { cfg.identity.bio = val; }
        Ok(cfg)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::get_path();
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content).context("Failed to write koad.json")
    }

    pub fn default_initial() -> Self {
        Self {
            version: "2.1".to_string(),
            identity: Identity {
                name: env::var("KOAD_NAME").unwrap_or_else(|_| "Koad".into()),
                role: env::var("KOAD_ROLE").unwrap_or_else(|_| "AI Persona".into()),
                bio: env::var("KOAD_BIO").unwrap_or_else(|_| "Agnostic AI coding framework.".into()),
            },
            preferences: Preferences {
                languages: vec!["Rust".into(), "Node.js".into(), "Python".into()],
                style: "programmatic-first".to_string(),
                principles: vec!["Simplicity first".into(), "Plan before build".into(), "Sanctuary Rule".into()],
            },
            drivers: HashMap::new(),
        }
    }
}

struct KoadDB {
    conn: Connection,
}

impl KoadDB {
    fn init() -> Result<Self> {
        let path = KoadConfig::get_db_path();
        let conn = Connection::open(path)?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS knowledge (
                id INTEGER PRIMARY KEY,
                category TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT,
                timestamp TEXT NOT NULL
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

    fn query(&self, term: &str) -> Result<Vec<(String, String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, content, timestamp FROM knowledge 
             WHERE content LIKE ?1 OR tags LIKE ?1 
             ORDER BY timestamp DESC LIMIT 20"
        )?;
        let rows = stmt.query_map(params![format!("%{}%", term)], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    fn get_recent(&self, limit: usize) -> Result<Vec<(String, String)>> {
        let mut stmt = self.conn.prepare(
            "SELECT category, content FROM knowledge ORDER BY timestamp DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }
}

fn get_gh_pat_for_path(path: &Path) -> (&'static str, &'static str) {
    let path_str = path.to_string_lossy();
    if path_str.contains("skylinks") { ("GITHUB_SKYLINKS_PAT", "Work (Skylinks)") } 
    else { ("GITHUB_PERSONAL_PAT", "Personal") }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = KoadConfig::load()?;
    let db = KoadDB::init()?;

    match cli.command {
        Commands::Boot { agent: _, project } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (pat_var, pat_desc) = get_gh_pat_for_path(&current_dir);
            println!("<koad_boot>");
            println!("Identity: {} ({})", config.identity.name, config.identity.role);
            println!("Auth: {} ({})", pat_var, pat_desc);
            println!("\n[Recent Memory]");
            for (cat, content) in db.get_recent(10)? {
                println!("- [{}] {}", cat, content);
            }
            if project {
                let progress_path = current_dir.join("PROJECT_PROGRESS.md");
                if progress_path.exists() {
                    let progress = std::fs::read_to_string(progress_path)?;
                    if let Some(start) = progress.find("## Snapshot") {
                        let end = progress.find("## Roadmap Alignment").unwrap_or(progress.len());
                        println!("\n[Project Progress]\n{}", progress[start..end].trim());
                    }
                }
            }
            println!("</koad_boot>");
        }
        Commands::Auth => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let (pat_var, pat_desc) = get_gh_pat_for_path(&current_dir);
            println!("Context: {} | Env: {}", pat_desc, pat_var);
        }
        Commands::Query { term } => {
            for (cat, content, ts) in db.query(&term)? {
                println!("- [{}] ({}) {}", cat, ts, content);
            }
        }
        Commands::Remember { category } => {
            match category {
                MemoryCategory::Fact { text, tags } => db.remember("fact", &text, tags)?,
                MemoryCategory::Learning { text, tags } => db.remember("learning", &text, tags)?,
            }
            println!("Memory updated in DB.");
        }
        Commands::Skill { action } => {
             let skills_dir = KoadConfig::get_home().join("skills");
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
                     let mut child = Command::new(skills_dir.join(name)).args(args).spawn()?;
                     child.wait()?;
                 }
             }
        }
        Commands::Init { force } => {
            let path = KoadConfig::get_path();
            if path.exists() && !force { anyhow::bail!("Exists."); }
            KoadConfig::default_initial().save()?; println!("Initialized.");
        }
        Commands::Harvest { path } => {
            let file = std::fs::File::open(&path)?;
            let reader = BufReader::new(file);
            let mut in_discovery = false;
            let mut count = 0;
            for line in reader.lines() {
                let line = line?;
                if line.starts_with("## Discoveries") { in_discovery = true; continue; }
                if line.starts_with("## ") && in_discovery { break; }
                if in_discovery && line.trim().starts_with("- ") {
                    db.remember("learning", &line.trim()[2..], None)?;
                    count += 1;
                }
            }
            if count > 0 { println!("Harvested {} learnings to DB.", count); }
        }
        Commands::Sync { source } => match source {
            SyncSource::Airtable { schema_only, base_id } => {
                let mut cmd_args = vec!["run".to_string(), "global/airtable_sync.py".to_string(), "--".to_string()];
                if schema_only { cmd_args.push("--schema-only".to_string()); }
                if let Some(id) = base_id { cmd_args.push("--base-id".to_string()); cmd_args.push(id); }
                let mut child = Command::new(env::current_exe()?).args(cmd_args).spawn()?; child.wait()?;
            }
            SyncSource::Notion { page_id, db_id } => {
                let mut cmd_args = vec!["run".to_string(), "global/notion_sync.py".to_string(), "--".to_string()];
                if let Some(id) = page_id { cmd_args.push("--page-id".to_string()); cmd_args.push(id); }
                if let Some(id) = db_id { cmd_args.push("--db-id".to_string()); cmd_args.push(id); }
                let mut child = Command::new(env::current_exe()?).args(cmd_args).spawn()?; child.wait()?;
            }
        }
        Commands::Gcloud { action } => {
            let mut cmd_args = vec!["run".to_string(), "global/gcloud_ops.py".to_string(), "--".to_string()];
            match action {
                GcloudAction::List { resource } => { cmd_args.push("list".to_string()); cmd_args.push("--resource".to_string()); cmd_args.push(resource); }
                GcloudAction::Deploy { name } => { cmd_args.push("deploy".to_string()); cmd_args.push("--name".to_string()); cmd_args.push(name); }
                GcloudAction::Logs { name, limit } => { cmd_args.push("logs".to_string()); cmd_args.push("--name".to_string()); cmd_args.push(name); cmd_args.push("--limit".to_string()); cmd_args.push(limit.to_string()); }
            }
            let mut child = Command::new(env::current_exe()?).args(cmd_args).spawn()?; child.wait()?;
        }
    }

    Ok(())
}
