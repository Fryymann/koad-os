use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::env;
use std::process::{Command, Stdio};
use chrono::{Local, Duration};
use std::io::{BufRead, BufReader, Write};
use rusqlite::params;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State, Path as AxumPath},
    routing::get,
    Router,
};
use tokio::sync::broadcast;
use tower_http::services::ServeDir;
use std::sync::Arc;

#[derive(Clone)]
struct AppState {
    // A map of topic -> sender
    channels: Arc<std::sync::Mutex<HashMap<String, broadcast::Sender<String>>>>,
}

impl AppState {
    fn get_or_create_channel(&self, topic: &str) -> broadcast::Sender<String> {
        let mut channels = self.channels.lock().unwrap();
        if let Some(tx) = channels.get(topic) {
            tx.clone()
        } else {
            let (tx, _) = broadcast::channel(100);
            channels.insert(topic.to_string(), tx.clone());
            tx
        }
    }
}

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
#[command(version = "2.4.1")]
#[command(about = "The KoadOS Control Plane")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true, default_value = "admin")]
    role: String,
}

#[derive(Subcommand)]
enum Commands {
    /// [AGENT] Initialize agent context for the current session.
    /// (Optimized for AI ingestion: loads identity, auth, and contextual memory)
    Boot {
        /// Select the agent driver to use (e.g., 'gemini').
        #[arg(short, long, default_value = "gemini")]
        agent: String,
        /// Ingest project-specific context (PROJECT_PROGRESS.md).
        #[arg(short, long)]
        project: bool,
        /// (Optional) Focus on a specific task ID.
        #[arg(short, long)]
        task: Option<String>,
        /// Output a compact pipe-delimited string for scripting.
        #[arg(short, long)]
        compact: bool,
    },
    /// [AGENT/HUMAN] Display active authentication tokens for GitHub and Google Drive.
    Auth,
    /// [HUMAN] Search the Koad Knowledge Base for a specific term or tag.
    Query { 
        /// Search term or tag.
        term: String,
        /// Maximum results to return (Token Efficiency).
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        /// (Optional) Filter by specific comma-separated tags.
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// [AGENT/ADMIN] Commit a new fact or learning to the persistent SQLite database.
    Remember {
        #[command(subcommand)]
        category: MemoryCategory,
    },
    /// [AGENT] Record a personal reflection or interpretation (Persona Journaling).
    /// Used for agent self-alignment and long-term reasoning storage.
    Ponder {
        /// Reflection text.
        text: String,
        /// (Optional) Comma-separated tags.
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// [AGENT/HUMAN] Manage and execute specialized KoadOS Skills (Python/JS/Rust).
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// [HUMAN/AGENT] Scaffold new files or projects using standardized KoadOS templates.
    Template {
        #[command(subcommand)]
        action: TemplateAction,
    },
    /// [ADMIN] Initialize a new KoadOS root configuration (~/.koad-os/koad.json).
    Init {
        /// Overwrite existing configuration.
        #[arg(short, long)]
        force: bool,
    },
    /// [AGENT/ADMIN] Harvest discoveries from external files or git history into contextual memory.
    Harvest {
        /// Path to the file to harvest discoveries from.
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Harvest learnings from recent git commits.
        #[arg(short, long)]
        git: bool,
    },
    /// [AGENT/ADMIN] Synchronize state between local SQLite and cloud sources (Airtable/Notion).
    Sync {
        #[command(subcommand)]
        source: SyncSource,
    },
    /// [AGENT/ADMIN] Manage Airtable bases and records directly.
    Airtable {
        #[command(subcommand)]
        action: AirtableAction,
    },
    /// [ADMIN] Start the Koad background service (Cognitive Booster).
    Serve {
        /// Stop the background daemon.
        #[arg(short, long)]
        stop: bool,
    },
    /// [AGENT] Interact with the Koad Stream (Notion-backed communication channel).
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },
    /// [AGENT/ADMIN] Execute Google Cloud Platform operations (Deployments, Logs, Audits).
    Gcloud {
        #[command(subcommand)]
        action: GcloudAction,
    },
    /// [AGENT/ADMIN] Manage files and synchronization with Google Drive.
    Drive {
        #[command(subcommand)]
        action: DriveAction,
    },
    /// [ADMIN] Mark a knowledge entry as inactive (soft delete).
    Retire {
        /// ID of the knowledge record.
        id: i64,
    },
    /// [AGENT/ADMIN] Archive the current session and capture learnings.
    /// Updates SESSION_LOG.md and harvests verified actions from history.
    Saveup {
        /// Short summary of the work completed.
        summary: String,
        /// Scope of work (e.g., project name).
        #[arg(short, long, default_value = "General")]
        scope: String,
        /// Comma-separated list of specific facts to remember.
        #[arg(short, long)]
        facts: Option<String>,
        /// Automatically harvest verified command executions from the last 4 hours.
        #[arg(short, long)]
        auto: bool,
    },
    /// [ADMIN] Register the current directory as a project in the Koad ecosystem.
    /// Requires a .koad directory to be present.
    Scan {
        /// Path to scan (defaults to CWD).
        path: Option<PathBuf>,
    },
    /// [HUMAN] Save a quick note or idea from the user to the agent.
    Note {
        /// Note content.
        text: String,
    },
    /// [HUMAN] Record a brainstorm or rant to the personal ledger.
    Brainstorm {
        /// Brainstorm or rant text.
        text: String,
        /// Mark as a 'rant' for categorical filtering.
        #[arg(short, long)]
        rant: bool,
    },
    /// [HUMAN/AGENT] List user notes to the agent.
    Notes {
        /// Maximum notes to return.
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    /// [HUMAN/AGENT] List recent brainstorms and rants.
    Brainstorms {
        /// Maximum brainstorms to return.
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    /// [AGENT] Dispatch a command to the Koad daemon for background execution.
    Dispatch {
        /// The command to execute.
        command: String,
        /// (Optional) Arguments for the command.
        #[arg(short, long)]
        args: Option<String>,
    },
    /// [ADMIN] Publish all local KoadOS changes to the remote origin (git push).
    Publish {
        /// Custom commit message.
        #[arg(short, long)]
        message: Option<String>,
    },
    /// [HUMAN] Display information about the current KoadOS identity and environment.
    Whoami,
    /// [HUMAN] Run a real-time TUI dashboard of the KoadOS state.
    Dash,
    /// [HUMAN/ADMIN] Run a deep health check of the KoadOS platform.
    Doctor,
    /// [HUMAN/ADMIN] Display live system metrics and platform state.
    Stat {
        /// Output as JSON.
        #[arg(short, long)]
        json: bool,
    },
    /// [ADMIN] Stream the live KoadOS event bus to the terminal.
    Tail {
        /// Topic to tail (e.g. 'events', 'metrics', 'agents').
        #[arg(short, long, default_value = "events")]
        topic: String,
    },
    /// [ADMIN] Manage system backups, snapshots, and disaster recovery.
    Vault {
        #[command(subcommand)]
        action: VaultAction,
    },
    /// [HUMAN/AGENT] Track or update the current active task specification.
    Spec {
        #[command(subcommand)]
        action: SpecAction,
    },
    /// [HUMAN/ADMIN] Run a self-diagnostic check of the KoadOS environment.
    Diagnostic {
        /// Perform a full system check including skills and remote access.
        #[arg(short, long)]
        full: bool,
    },
    /// [HUMAN] Display KoadOS developer and onboarding guides.
    Guide {
        /// Guide name (onboarding, development, architecture).
        topic: Option<String>,
    },
    /// [AGENT/ADMIN] Log out the current session and release role slots.
    Logout {
        /// Session ID to close (defaults to current process session if found).
        session_id: Option<String>,
    },
    /// [AGENT/HUMAN] Host a local web server for testing front-ends or the Koad Dashboard.
    Host {
        /// Port to bind the server to.
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        /// Directory to serve (defaults to KoadOS dashboard if omitted).
        #[arg(short, long)]
        dir: Option<PathBuf>,
        /// Stop the background host.
        #[arg(short, long)]
        stop: bool,
    },
    /// [ADMIN] Manage and apply KoadOS v3 Blueprints for automated scaffolding.
    Blueprint {
        #[command(subcommand)]
        action: BlueprintAction,
    },
}

#[derive(Subcommand)]
enum BlueprintAction {
    /// List all registered blueprints.
    List,
    /// Apply a blueprint to the current environment.
    Use {
        /// Blueprint ID (e.g. 'koad-skill').
        blueprint: String,
        /// Variables in key=val format.
        #[arg(short, long)]
        var: Vec<String>,
    }
}

#[derive(Subcommand)]
enum VaultAction {
    /// Create a new atomic snapshot of the platform state.
    Snapshot,
    /// List all available snapshots in the vault.
    List,
    /// Restore the platform state from a specific snapshot.
    Restore {
        /// Snapshot name (from 'koad vault list').
        name: String,
    }
}

#[derive(Subcommand)]
enum SpecAction {
    /// Update the current task specification.
    Set {
        /// Task title.
        title: String,
        /// (Optional) Detailed description or plan.
        #[arg(short, long)]
        desc: Option<String>,
        /// Task status (Active, Paused, Complete).
        #[arg(short, long, default_value = "Active")]
        status: String,
    },
    /// Display the current task specification.
    Read,
    /// Clear the current task specification.
    Clear,
}

#[derive(Subcommand)]
enum DriveAction {
    /// List files in the configured Google Drive directory.
    List { 
        /// Include shared drive files.
        #[arg(short, long)] 
        shared: bool 
    },
    /// Download a file by its Drive ID.
    Download { 
        /// Google Drive file ID.
        id: String, 
        /// Destination path.
        #[arg(short, long)] 
        dest: Option<PathBuf> 
    },
    /// Sync local directories with Google Drive based on project config.
    Sync,
}

#[derive(Subcommand)]
enum GcloudAction {
    /// List GCP resources.
    List { 
        /// Resource type (functions, buckets, etc.).
        #[arg(short, long, default_value = "functions")] 
        resource: String 
    },
    /// Trigger a Cloud Build deployment for a named function.
    Deploy { 
        /// Function name.
        name: String 
    },
    /// Tail or fetch logs for a specific function.
    Logs { 
        /// Function name.
        name: String, 
        /// Max log entries to return.
        #[arg(short, long, default_value_t = 20)] 
        limit: u32 
    },
    /// Run a security and spend audit on the project.
    Audit { 
        /// Project alias (ops, prod, etc.).
        #[arg(short, long, default_value = "ops")] 
        project: String 
    },
}

#[derive(Subcommand)]
enum SyncSource {
    /// Sync with Airtable bases.
    Airtable { 
        /// Only sync table schemas, not record data.
        #[arg(short, long)] 
        schema_only: bool, 
        /// (Optional) Specific Base ID.
        #[arg(short, long)] 
        base_id: Option<String> 
    },
    /// Sync with Notion pages or databases.
    Notion { 
        /// Specific Notion Page ID.
        #[arg(short, long)] 
        page_id: Option<String>, 
        /// Specific Notion Database ID.
        #[arg(short, long)] 
        db_id: Option<String> 
    },
    /// Sync a resource by its shortcut name defined in koad.json.
    Named { 
        /// Shortcut name (e.g., 'koad', 'stream', 'memories').
        name: String 
    },
}

#[derive(Subcommand)]
enum AirtableAction {
    /// List records from an Airtable table.
    List {
        /// The Airtable Base ID.
        base_id: String,
        /// The Table Name.
        table_name: String,
        /// (Optional) Filter formula.
        #[arg(short, long)]
        filter: Option<String>,
        /// (Optional) Max records to return.
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    /// Get a specific record by ID.
    Get {
        /// The Airtable Base ID.
        base_id: String,
        /// The Table Name.
        table_name: String,
        /// The Record ID.
        record_id: String,
    },
    /// Update a record.
    Update {
        /// The Airtable Base ID.
        base_id: String,
        /// The Table Name.
        table_name: String,
        /// The Record ID.
        record_id: String,
        /// JSON fields to update.
        fields: String,
    },
}

#[derive(Subcommand)]
enum StreamAction {
    /// Post a new message to the Koad Stream.
    Post {
        /// Message topic.
        topic: String,
        /// Message content.
        message: String,
        /// Message type (Log, Alert, Query, etc.).
        #[arg(short, long, default_value = "Log")]
        msg_type: String,
    },
    /// List recent messages from the Koad Stream.
    List {
        /// Number of messages to retrieve.
        #[arg(short, long, default_value_t = 5)]
        limit: usize,
    },
}

#[derive(Subcommand)]
enum MemoryCategory {
    /// Record an objective fact.
    Fact { 
        /// Fact text.
        text: String, 
        /// Comma-separated tags.
        #[arg(short, long)] 
        tags: Option<String> 
    },
    /// Record a lesson learned or process improvement.
    Learning { 
        /// Learning text.
        text: String, 
        /// Comma-separated tags.
        #[arg(short, long)] 
        tags: Option<String> 
    },
}

#[derive(Subcommand)]
enum SkillAction {
    /// List all available KoadOS Skills.
    List,
    /// Execute a skill by name.
    Run { 
        /// Skill name (path relative to skills/).
        name: String, 
        /// Arguments passed to the skill.
        #[arg(last = true)] 
        args: Vec<String> 
    },
}

#[derive(Subcommand)]
enum TemplateAction {
    /// List all available templates.
    List,
    /// Apply a template to the current directory.
    Use { 
        /// Template name.
        name: String, 
        /// Destination path (defaults to name).
        #[arg(short, long)] 
        out: Option<PathBuf> 
    },
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
                booster_enabled: false,
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

pub struct KoadDB {
    pub pool: Pool<SqliteConnectionManager>,
}

impl KoadDB {
    pub fn init() -> Result<Self> {
        let path = KoadConfig::get_db_path()?;
        let manager = SqliteConnectionManager::file(path);
        let pool = Pool::builder()
            .max_size(10)
            .build(manager)
            .context("Failed to create connection pool")?;
        
        let conn = pool.get().context("Failed to get connection from pool")?;
        conn.execute("CREATE TABLE IF NOT EXISTS knowledge (id INTEGER PRIMARY KEY, category TEXT NOT NULL, content TEXT NOT NULL, tags TEXT, timestamp TEXT NOT NULL, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS projects (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE, path TEXT NOT NULL, role TEXT, stack TEXT, last_boot TEXT, active INTEGER DEFAULT 1)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS executions (id INTEGER PRIMARY KEY, command TEXT NOT NULL, args TEXT, timestamp TEXT NOT NULL, status TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS notion_index (id TEXT PRIMARY KEY, name TEXT, type TEXT, last_sync TEXT, cloud_edited TEXT, url TEXT)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS active_spec (id INTEGER PRIMARY KEY, title TEXT NOT NULL, description TEXT, status TEXT, last_update TEXT NOT NULL)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS ian_notes (id INTEGER PRIMARY KEY, content TEXT NOT NULL, timestamp TEXT NOT NULL)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS brainstorms (id INTEGER PRIMARY KEY, content TEXT NOT NULL, category TEXT NOT NULL, timestamp TEXT NOT NULL)", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS project_state (
            id INTEGER PRIMARY KEY,
            path TEXT NOT NULL,
            event_type TEXT NOT NULL,
            timestamp TEXT NOT NULL,
            summary TEXT
        )", [])?;
        conn.execute("CREATE TABLE IF NOT EXISTS command_queue (
            id INTEGER PRIMARY KEY,
            command TEXT NOT NULL,
            args TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            output TEXT,
            pid INTEGER,
            created_at TEXT NOT NULL,
            started_at TEXT,
            finished_at TEXT
        )", [])?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                session_id TEXT PRIMARY KEY,
                agent TEXT NOT NULL,
                role TEXT NOT NULL,
                status TEXT NOT NULL,
                last_heartbeat TEXT NOT NULL,
                pid INTEGER,
                current_task_id INTEGER
            )",
            [],
        )?;
        Ok(Self { pool })
    }

    pub fn get_conn(&self) -> Result<r2d2::PooledConnection<SqliteConnectionManager>> {
        self.pool.get().context("Failed to get connection from pool")
    }

    pub fn set_spec(&self, title: &str, description: Option<String>, status: &str) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.get_conn()?.execute("INSERT INTO active_spec (id, title, description, status, last_update) VALUES (1, ?1, ?2, ?3, ?4) ON CONFLICT(id) DO UPDATE SET title=?1, description=?2, status=?3, last_update=?4", params![title, description, status, now])?;
        Ok(())
    }

    pub fn get_spec(&self) -> Result<Option<(String, Option<String>, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT title, description, status, last_update FROM active_spec WHERE id = 1")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? { Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))) } else { Ok(None) }
    }

    pub fn remember(&self, category: &str, content: &str, tags: Option<String>) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.get_conn()?.execute("INSERT INTO knowledge (category, content, tags, timestamp) VALUES (?1, ?2, ?3, ?4)", params![category, content, tags, timestamp])?;
        Ok(())
    }

    pub fn save_note(&self, content: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.get_conn()?.execute("INSERT INTO ian_notes (content, timestamp) VALUES (?1, ?2)", params![content, timestamp])?;
        Ok(())
    }

    pub fn save_brainstorm(&self, content: &str, is_rant: bool) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let category = if is_rant { "rant" } else { "brainstorm" };
        self.get_conn()?.execute("INSERT INTO brainstorms (content, category, timestamp) VALUES (?1, ?2, ?3)", params![content, category, timestamp])?;
        Ok(())
    }

    pub fn dispatch(&self, command: &str, args: Option<String>) -> Result<()> {
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.get_conn()?.execute("INSERT INTO command_queue (command, args, status, created_at) VALUES (?1, ?2, 'pending', ?3)", params![command, args, now])?;
        Ok(())
    }

    pub fn get_recent_brainstorms(&self, limit: usize) -> Result<Vec<(String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT content, category, timestamp FROM brainstorms ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn get_notes(&self, limit: usize) -> Result<Vec<(i64, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT id, content, timestamp FROM ian_notes ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn get_ponderings(&self, limit: usize) -> Result<Vec<String>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT content FROM knowledge WHERE category = 'pondering' AND active = 1 ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit], |row| Ok(row.get(0)?))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn log_execution(&self, command: &str, args: &str, status: &str) -> Result<()> {
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.get_conn()?.execute("INSERT INTO executions (command, args, timestamp, status) VALUES (?1, ?2, ?3, ?4)", params![command, args, timestamp, status])?;
        Ok(())
    }

    pub fn get_recent_executions(&self, hours: i64) -> Result<Vec<(String, String, String)>> {
        let cutoff = (Local::now() - Duration::hours(hours)).format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT command, args, status FROM executions WHERE timestamp > ?1 ORDER BY timestamp DESC LIMIT 15")?;
        let rows = stmt.query_map(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn register_project(&self, name: &str, path: &str) -> Result<()> {
        let last_boot = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        self.get_conn()?.execute("INSERT INTO projects (name, path, last_boot) VALUES (?1, ?2, ?3) ON CONFLICT(name) DO UPDATE SET last_boot=?3, path=?2", params![name, path, last_boot])?;
        Ok(())
    }

    pub fn retire(&self, id: i64) -> Result<()> {
        self.get_conn()?.execute("UPDATE knowledge SET active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn query(&self, term: &str, limit: usize, tags: Option<String>) -> Result<Vec<(i64, String, String, String)>> {
        let mut query_str = String::from("SELECT id, category, content, timestamp FROM knowledge WHERE active = 1");
        let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
        
        if !term.is_empty() {
            query_str.push_str(" AND (content LIKE ?1 OR tags LIKE ?1)");
            params_vec.push(format!("%{}%", term).into());
        }

        if let Some(t) = tags {
            for tag in t.split(',') {
                query_str.push_str(" AND tags LIKE ?");
                params_vec.push(format!("%{}%", tag.trim()).into());
                query_str.push_str(&(params_vec.len()).to_string());
            }
        }

        query_str.push_str(" ORDER BY timestamp DESC LIMIT ?");
        params_vec.push((limit as i64).into());
        query_str.push_str(&(params_vec.len()).to_string());

        let conn = self.get_conn()?;
        let mut stmt = conn.prepare(&query_str)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?;
        
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn get_contextual(&self, limit: usize, tags: Vec<String>) -> Result<Vec<(String, String)>> {
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
            
            let conn = self.get_conn()?;
            let mut stmt = conn.prepare(&query)?;
            let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
            for t in &tags { params_vec.push(format!("%{}%", t).into()); }
            params_vec.push(((limit / 2) as i64).into());
            let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| Ok((row.get(0)?, row.get(1)?)))?;
            for row in rows { results.push(row?); }
        }
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT category, content FROM knowledge WHERE active = 1 AND category != 'pondering' ORDER BY timestamp DESC LIMIT ?1")?;
        let rows = stmt.query_map(params![limit.saturating_sub(results.len())], |row| Ok((row.get(0)?, row.get(1)?)))?;
        for row in rows { 
            let item = row?;
            if !results.contains(&item) { results.push(item); }
        }
        Ok(results)
    }

    pub fn get_recent_deltas(&self, minutes: i64) -> Result<Vec<(String, String, String)>> {
        let cutoff = (Local::now() - Duration::minutes(minutes)).format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT path, event_type, timestamp FROM project_state WHERE timestamp > ?1 ORDER BY timestamp DESC LIMIT 10")?;
        let rows = stmt.query_map(params![cutoff], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn get_active_project(&self, current_path: &str) -> Result<Option<(String, Option<String>, Option<String>)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT name, role, stack FROM projects WHERE path = ?1 AND active = 1")?;
        let mut rows = stmt.query(params![current_path])?;
        if let Some(row) = rows.next()? { Ok(Some((row.get(0)?, row.get(1)?, row.get(2)?))) } else { Ok(None) }
    }

    pub fn get_workflows(&self, status: Option<String>, limit: usize) -> Result<Vec<(i64, String, String, Option<String>)>> {
        let conn = self.get_conn()?;
        let mut query = String::from("SELECT id, title, status, project FROM workflows");
        let mut params_vec: Vec<rusqlite::types::Value> = Vec::new();
        if let Some(s) = status {
            query.push_str(" WHERE status = ?1");
            params_vec.push(s.into());
        }
        query.push_str(" ORDER BY priority DESC, last_update DESC LIMIT ?");
        let limit_idx = params_vec.len() + 1;
        params_vec.push((limit as i64).into());
        
        let mut stmt = conn.prepare(&query)?;
        let rows = stmt.query_map(rusqlite::params_from_iter(params_vec), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
        })?;
        
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }

    pub fn get_notion_index(&self) -> Result<Vec<(String, String, String, String, String)>> {
        let conn = self.get_conn()?;
        let mut stmt = conn.prepare("SELECT name, type, last_sync, cloud_edited, id FROM notion_index ORDER BY name ASC")?;
        let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))?;
        let mut results = Vec::new();
        for row in rows { results.push(row?); }
        Ok(results)
    }
}

fn get_gh_pat_for_path(path: &Path) -> (&'static str, &'static str) {
    let path_str = path.to_string_lossy();
    if path_str.contains("/skylinks/") || path_str.contains("\\skylinks\\") {
        ("GITHUB_SKYLINKS_PAT", "Skylinks")
    } else {
        ("GITHUB_PERSONAL_PAT", "Personal")
    }
}

fn get_gdrive_token_for_path(path: &Path) -> (&'static str, &'static str) {
    let path_str = path.to_string_lossy();
    if path_str.contains("/skylinks/") || path_str.contains("\\skylinks\\") {
        ("GDRIVE_SKYLINKS_TOKEN", "Skylinks")
    } else {
        ("GDRIVE_PERSONAL_TOKEN", "Personal")
    }
}

fn detect_context_tags(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();
    let path_str = path.to_string_lossy().to_lowercase();
    if path_str.contains("rust") || path.join("Cargo.toml").exists() { tags.push("rust".into()); }
    if path_str.contains("node") || path.join("package.json").exists() { tags.push("node".into()); }
    tags
}

fn run_diagnostic(full: bool, config: &KoadConfig) -> Result<()> {
    let home = KoadConfig::get_home()?;
    println!("--- KoadOS Diagnostic ---\nKoad Home: {}", home.display());

    let mut files_missing = false;
    for file in ["koad.json", "koad.db", "SESSION_LOG.md"] {
        if home.join(file).exists() { println!("[PASS] {} exists", file); } 
        else { println!("[FAIL] {} is missing", file); files_missing = true; }
    }

    for var in ["KOAD_HOME", "KOAD_NAME", "KOAD_ROLE", "KOAD_BIO"] {
        match env::var(var) {
            Ok(val) => println!("[PASS] Env {} = {}", var, val),
            Err(_) => println!("[INFO] Env {} is not set (using defaults)", var),
        }
    }

    let db_status = match KoadDB::init() {
        Ok(db) => match db.get_conn()?.query_row("SELECT count(*) FROM knowledge", [], |r| Ok(r.get::<_, i64>(0)?)) {
            Ok(c) => format!("[PASS] Database connected (Knowledge entries: {})", c),
            Err(e) => format!("[FAIL] Database query failed: {}", e),
        },
        Err(e) => format!("[FAIL] Database connection failed: {}", e),
    };
    println!("{}", db_status);

    if full {
        println!("\n--- Full Integration Check ---");
        // Bridge Skills (Core if agent matches)
        for skill in ["gemini/remember.py", "gemini/harvest.py", "gemini/search.py"] {
            let status = if home.join("skills").join(skill).exists() { "[PASS]" } else { "[FAIL] (Core Skill)" };
            println!("{} Bridge skill {} found", status, skill);
        }

        // Notion (Optional)
        if config.notion.mcp {
            println!("[PASS] Notion Integration: Enabled (MCP)");
            if env::var("NOTION_TOKEN").is_err() { println!("[WARN] NOTION_TOKEN is not set. Notion sync will fail."); }
            if env::var("KOAD_STREAM_DB_ID").is_err() { println!("[WARN] KOAD_STREAM_DB_ID is not set. Stream posts will fail."); }
        } else {
            println!("[INFO] Notion Integration: Not Enabled (Optional)");
        }

        // Airtable (Optional)
        if env::var("AIRTABLE_KOAD_PAT").is_ok() {
            println!("[PASS] Airtable Integration: Auth set");
        } else {
            println!("[INFO] Airtable Integration: AIRTABLE_KOAD_PAT not set (Optional)");
        }

        let current_dir = env::current_dir()?;
        let (pat_var, pat_label) = get_gh_pat_for_path(&current_dir);
        let auth_status = if env::var(pat_var).is_ok() { "[PASS]" } else { "[FAIL] (Core Auth)" };
        println!("{} Auth token {} ({}) is set", auth_status, pat_var, pat_label);

        println!("\n--- Tool Availability ---");
        // Core Tools
        for tool in ["git"] {
            let status = if Command::new(tool).arg("--version").output().is_ok() { "[PASS]" } else { "[FAIL] (CORE)" };
            println!("{} {} is available", status, tool);
        }

        // Stack Tools (Conditionally Core based on preferences)
        for lang in &config.preferences.languages {
            let tools = match lang.as_str() {
                "Python" => vec!["python3"],
                "Node.js" => vec!["node", "npm"],
                "Rust" => vec!["cargo"],
                _ => vec![],
            };
            for t in tools {
                let status = if Command::new(t).arg("--version").output().is_ok() { "[PASS]" } else { "[FAIL] (STACK)" };
                println!("{} {} is available", status, t);
            }
        }

        // Optional Tools
        for tool in ["gh"] {
            if Command::new(tool).arg("--version").output().is_ok() {
                println!("[PASS] {} is available (Optional)", tool);
            } else {
                println!("[INFO] {} is not installed (Optional)", tool);
            }
        }

        // Expansion: Cognitive Booster
        println!("\n--- Expansions ---");
        if config.preferences.booster_enabled {
            let daemon_path = home.join("bin/koad-daemon");
            if daemon_path.exists() {
                println!("[PASS] Cognitive Booster: Enabled & Installed");
                let is_running = Command::new("pgrep").arg("-f").arg("koad-daemon").output().map(|o| o.status.success()).unwrap_or(false);
                if is_running { println!("[PASS] koad-daemon is running"); }
                else { println!("[INFO] koad-daemon is NOT running (Use 'koad serve' to start)"); }
            } else {
                println!("[FAIL] Cognitive Booster: Enabled in config but binary missing at {}", daemon_path.display());
            }
        } else {
            println!("[INFO] Cognitive Booster: Disabled (Optional Expansion)");
        }
    }

    println!("\nStatus: {}", if files_missing { "DEGRADED" } else { "OPERATIONAL" });
    Ok(())
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
        Commands::Dash => tui::run_dash(&db)?,
        Commands::Blueprint { action } => {
            if !is_admin { anyhow::bail!("Admin Access Required."); }
            let s_path = KoadConfig::get_home()?.join("skills/admin/blueprint_engine.py");
            match action {
                BlueprintAction::List => {
                    Command::new(s_path).arg("list").spawn()?.wait()?;
                }
                BlueprintAction::Use { blueprint, var } => {
                    let mut cmd = Command::new(s_path);
                    cmd.arg("use").arg(blueprint);
                    for v in var {
                        cmd.arg("--var").arg(v);
                    }
                    cmd.spawn()?.wait()?;
                }
            }
        }
        Commands::Doctor => {
            println!("--- KoadOS v3 Doctor ---");
            let res = reqwest::blocking::get("http://localhost:8080/health");
            match res {
                Ok(resp) => {
                    let health: Value = resp.json()?;
                    println!("[PASS] Kernel: Online (Uptime: {}s)", health["uptime"]);
                    println!("[PASS] Database: {}", health["database"]);
                    println!("[PASS] Event Bus: {}", health["event_bus"]);
                }
                Err(_) => println!("[FAIL] Kernel: Offline. Run 'kspine start' to fix."),
            }
            
            let db_path = KoadConfig::get_home()?.join("koad.db");
            if db_path.exists() { println!("[PASS] Storage: Database found."); }
            else { println!("[FAIL] Storage: Database missing!"); }

            let booster_pid = KoadConfig::get_home()?.join("kbooster.pid");
            if booster_pid.exists() { println!("[INFO] Booster: Active."); }
            else { println!("[INFO] Booster: Idle/Not running."); }
        }
        Commands::Stat { json } => {
            let mut cmd = Command::new("kspine");
            cmd.arg("status");
            if json { cmd.arg("--json"); }
            cmd.spawn()?.wait()?;
        }
        Commands::Tail { topic } => {
            println!(">>> Tailing KoadOS topic: '{}' (Ctrl+C to stop)", topic);
            let url = format!("ws://localhost:8080/ws/{}", topic);
            let (mut socket, _) = tungstenite::connect(url)?;
            loop {
                let msg = socket.read()?;
                if msg.is_text() || msg.is_binary() {
                    println!("{}", msg.into_text()?);
                }
            }
        }
        Commands::Vault { action } => {
            if !is_admin { anyhow::bail!("Admin Access Required."); }
            let s_path = KoadConfig::get_home()?.join("skills/admin/vault.py");
            match action {
                VaultAction::Snapshot => {
                    Command::new(s_path).arg("snapshot").spawn()?.wait()?;
                }
                VaultAction::List => {
                    Command::new(s_path).arg("list").spawn()?.wait()?;
                }
                VaultAction::Restore { name } => {
                    println!("WARNING: Restore will overwrite current state. Continue? [y/N]");
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input)?;
                    if input.trim().to_lowercase() == "y" {
                        Command::new(s_path).arg("restore").arg("--name").arg(name).spawn()?.wait()?;
                    }
                }
            }
        }
        Commands::Spec { action } => {
            match action {
                SpecAction::Set { title, desc, status } => db.set_spec(&title, desc, &status)?,
                SpecAction::Read => {
                    if let Some((t, d, s, ts)) = db.get_spec()? {
                        println!("--- Active Spec [{}] ---", ts);
                        println!("Title:   {}", t);
                        println!("Status:  {}", s);
                        if let Some(desc) = d { println!("Detail:  {}", desc); }
                    } else { println!("No active task spec."); }
                },
                SpecAction::Clear => {
                    db.get_conn()?.execute("DELETE FROM active_spec WHERE id = 1", [])?;
                    println!("Spec cleared.");
                }
            }
        },
        Commands::Diagnostic { full } => run_diagnostic(full, &config)?,
        Commands::Logout { session_id } => {
            let conn = db.get_conn()?;
            match session_id {
                Some(sid) => {
                    conn.execute("UPDATE sessions SET status = 'closed' WHERE session_id = ?1", [sid])?;
                    println!("Session closed.");
                }
                None => {
                    conn.execute("UPDATE sessions SET status = 'closed' WHERE pid = ?1 AND status = 'active'", [std::process::id()])?;
                    println!("Active process sessions closed.");
                }
            }
        }
        Commands::Host { port, dir, stop } => {
            if stop {
                println!("DEPRECATED: Please use 'kspine stop'");
                let _ = Command::new("kspine").arg("stop").status();
                return Ok(());
            }

            println!("DEPRECATED: Please use 'kspine start --port {}'", port);
            let mut cmd = Command::new("kspine");
            cmd.arg("start").arg("--port").arg(port.to_string());
            let _ = cmd.status();
            std::process::exit(0);
        }
        Commands::Guide { topic } => {
            let docs_dir = KoadConfig::get_home()?.join("docs");
            if let Some(t) = topic {
                // Try to find the file regardless of case
                let mut found = false;
                if docs_dir.exists() {
                    for entry in std::fs::read_dir(&docs_dir)? {
                        let entry = entry?;
                        let name = entry.path().file_stem().unwrap_or_default().to_string_lossy().to_lowercase();
                        if name == t.to_lowercase() {
                            let content = std::fs::read_to_string(entry.path())?;
                            println!("{}", content);
                            found = true;
                            break;
                        }
                    }
                }
                if !found {
                    println!("Guide for '{}' not found in {}.", t, docs_dir.display());
                }
            } else {
                println!("--- KoadOS Developer & Onboarding Guides ---");
                if docs_dir.exists() {
                    for entry in std::fs::read_dir(docs_dir)? {
                        let entry = entry?;
                        if entry.path().extension().map_or(false, |e| e == "md") {
                            println!("- {}", entry.path().file_stem().unwrap().to_string_lossy().to_lowercase());
                        }
                    }
                } else {
                    println!("No guides found.");
                }
                println!("\nUsage: koad guide <topic>");
            }
        }
        Commands::Boot { agent, project, task: _, compact } => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let current_path_str = current_dir.to_string_lossy().to_string();
            let (pat_var, _) = get_gh_pat_for_path(&current_dir);
            let (drive_var, _) = get_gdrive_token_for_path(&current_dir);
            let tags = detect_context_tags(&current_dir);

            let mut session_id = "BOOT".to_string();

            if !compact {
                // 1. Role Arbitration
                let conn = db.get_conn()?;
                let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                session_id = uuid::Uuid::new_v4().to_string();

                if role.to_lowercase() == "admin" {
                    // Check for active admin sessions in the last 2 minutes
                    let cutoff = (Local::now() - chrono::Duration::minutes(2)).format("%Y-%m-%d %H:%M:%S").to_string();
                    let mut stmt = conn.prepare("SELECT session_id, agent FROM sessions WHERE role = 'admin' AND last_heartbeat > ?1 AND status = 'active'")?;
                    let active_admin = stmt.query_row([cutoff], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)));

                    if let Ok((old_sid, old_agent)) = active_admin {
                        if old_agent != agent {
                            anyhow::bail!("Admin role is currently occupied by {} (Session: {}). Only one Admin allowed.", old_agent, old_sid);
                        } else {
                            // Reclaiming own session
                            session_id = old_sid;
                        }
                    }
                }

                // 2. Register/Refresh Session
                conn.execute(
                    "INSERT INTO sessions (session_id, agent, role, status, last_heartbeat, pid)
                     VALUES (?1, ?2, ?3, 'active', ?4, ?5)
                     ON CONFLICT(session_id) DO UPDATE SET last_heartbeat = excluded.last_heartbeat, status = 'active'",
                    params![session_id, agent, role, now, std::process::id()],
                )?;

                // 3. Launch Cognitive Booster v3 Sidecar
                if config.preferences.booster_enabled {
                    let booster_path = KoadConfig::get_home()?.join("bin/kbooster");
                    if booster_path.exists() {
                         let log_path = KoadConfig::get_home()?.join(format!("booster_{}.log", config.identity.name.to_lowercase()));
                         let log_file = std::fs::OpenOptions::new().append(true).create(true).open(&log_path)?;
                         let _ = Command::new(booster_path)
                            .arg("--agent-id").arg(&config.identity.name)
                            .arg("--role").arg(&config.identity.role)
                            .stdin(Stdio::null())
                            .stdout(Stdio::from(log_file.try_clone()?))
                            .stderr(Stdio::from(log_file))
                            .spawn();
                    }
                }
            }

            if compact {
                println!("I:{}|R:{}|G:{}|D:{}|T:{}|S:{}", config.identity.name, config.identity.role, pat_var, drive_var, tags.join(","), session_id);
            } else {
                println!("<koad_boot>");
                println!("Session:  {}", session_id);
                println!("Identity: {} ({})", config.identity.name, config.identity.role);
                println!("Auth: GH={} | GD={}", pat_var, drive_var);
                
                if let Some(driver) = config.drivers.get(&agent) {
                    let b_path = driver.bootstrap.replace("~", &env::var("HOME").unwrap_or_default());
                    if let Ok(content) = std::fs::read_to_string(b_path) {
                        println!("\n[Bootstrap: {}]\n{}", agent, content);
                    }
                }

                if let Some((p_name, _, _)) = db.get_active_project(&current_path_str)? {
                    println!("\n[Active Project: {}]", p_name);
                }

                println!("\n[Persona Reflections]");
                let ponders = db.get_ponderings(3)?;
                if ponders.is_empty() { println!("- No active reflections."); }
                for p in ponders { println!("- {}", p); }

                println!("\n[Contextual Memory]");
                for (cat, content) in db.get_contextual(8, tags)? { println!("- [{}] {}", cat, content); }

                println!("\n[Ian's Notes]");
                let notes = db.get_notes(5)?;
                if notes.is_empty() { println!("- No new notes from Ian."); }
                for (_, content, ts) in notes { println!("- ({}) {}", ts, content); }

                if config.preferences.booster_enabled {
                    println!("\n[Spine Intelligence Deltas]");
                    // Query knowledge for delta tags
                    let deltas = db.query("", 5, Some("delta".to_string()))?;
                    if deltas.is_empty() {
                        println!("- Spine is quiet. No background events recorded.");
                    } else {
                        for (_, _, content, ts) in deltas {
                            println!("- ({}) {}", ts, content);
                        }
                    }

                    let file_deltas = db.get_recent_deltas(60)?;
                    if !file_deltas.is_empty() {
                        println!("\n[Booster File Deltas (Last 60m)]");
                        for (path, event, ts) in file_deltas {
                            let p = PathBuf::from(path);
                            println!("- {}: {} ({})", event, p.file_name().unwrap_or_default().to_string_lossy(), ts);
                        }
                    }
                }

                println!("\n[Notion Index Summary]");
                let index = db.get_notion_index()?;
                if index.is_empty() { println!("- No resources indexed."); }
                for (name, r_type, local, _, _) in index { println!("- {}: [{}] (Synced: {})", name, r_type, local); }

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
        Commands::Query { term, limit, tags } => {
            let url = format!("http://localhost:8080/knowledge?term={}&limit={}&tags={}", 
                url::form_urlencoded::byte_serialize(term.as_bytes()).collect::<String>(),
                limit,
                tags.unwrap_or_default());
            let res = reqwest::blocking::get(url)?.json::<Vec<Value>>()?;
            for item in res {
                println!("- ID:{} [{}] ({}) {}", 
                    item["id"], item["category"], item["timestamp"], item["content"]);
            }
        }
        Commands::Remember { category } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            let (cat_str, text, tags) = match category {
                MemoryCategory::Fact { text, tags } => ("fact", text, tags),
                MemoryCategory::Learning { text, tags } => ("learning", text, tags),
            };
            
            let client = reqwest::blocking::Client::new();
            client.post("http://localhost:8080/knowledge")
                .json(&serde_json::json!({
                    "category": cat_str,
                    "content": text,
                    "tags": tags
                }))
                .send()?;
            
            println!("Memory updated via Kernel.");
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
            if p.exists() && !force { anyhow::bail!("Configuration already exists at {}. Use --force to overwrite.", p.display()); }
            
            // If file exists and we are forcing, we might want to preserve some things?
            // Actually, for a clean install, overwriting is fine. 
            // The issue is koad-setup.sh generates a koad.json then calls init --force.
            // Let's make koad-setup.sh skip generating if it's going to call init anyway,
            // OR make init load existing if it exists.
            
            let config_to_save = if p.exists() {
                let mut existing = KoadConfig::load()?;
                // Ensure version is updated
                existing.version = "2.4".into();
                existing
            } else {
                KoadConfig::default_initial()
            };
            
            config_to_save.save()?; 
            println!("Initialized.");
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
        Commands::Airtable { action } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            let pat = env::var("AIRTABLE_KOAD_PAT").context("AIRTABLE_KOAD_PAT not set")?;
            let client = AirtableClient::new(pat);
            
            match action {
                AirtableAction::List { base_id, table_name, filter, limit } => {
                    let records = client.list_records(&base_id, &table_name, filter, Some(limit))?;
                    println!("--- {} Records found ---", records.len());
                    for r in records {
                        println!("ID: {} | Fields: {}", r.id, r.fields);
                    }
                },
                AirtableAction::Get { base_id, table_name, record_id } => {
                    let record = client.get_record(&base_id, &table_name, &record_id)?;
                    println!("ID: {} | Fields: {}", record.id, record.fields);
                },
                AirtableAction::Update { base_id, table_name, record_id, fields } => {
                    let json_fields: Value = serde_json::from_str(&fields)?;
                    let _record = client.update_record(&base_id, &table_name, &record_id, json_fields)?;
                    println!("Record {} updated successfully.", record_id);
                }
            }
        }
        Commands::Serve { stop } => {
            if stop {
                println!("DEPRECATED: Please use 'kspine stop'");
                let _ = Command::new("kspine").arg("stop").status();
            } else {
                println!("DEPRECATED: Please use 'kspine start'");
                let _ = Command::new("kspine").arg("start").status();
            }
            return Ok(());
        },
        Commands::Sync { source } => {
            if !has_privileged_access { anyhow::bail!("Access Denied."); }
            match source {
                SyncSource::Airtable { schema_only: _, base_id } => {
                    let _pat = env::var("AIRTABLE_KOAD_PAT").context("AIRTABLE_KOAD_PAT not set")?;
                    println!("Syncing Airtable (Native Rust Implementation)...");
                    if let Some(id) = base_id {
                        println!("Target Base: {}", id);
                        // Future: Ingest schema/records into koad.db
                    } else {
                        println!("No Base ID provided. Scanning configured projects for Airtable metadata...");
                    }
                }
                SyncSource::Notion { page_id, db_id } => {
                    if config.notion.mcp {
                        println!("Note: MCP is enabled. The sync skill will use MCP tools for the backend pass.");
                    }
                    let mut args = vec!["skill".to_string(), "run".to_string(), "global/notion_sync.py".to_string(), "--".to_string()];
                    if let Some(id) = page_id { args.push("--page-id".to_string()); args.push(id); }
                    if let Some(id) = db_id { args.push("--db-id".to_string()); args.push(id); }
                    Command::new(env::current_exe()?).args(args).spawn()?.wait()?;
                }
                SyncSource::Named { name } => {
                    let id = config.notion.index.get(&name).ok_or_else(|| anyhow::anyhow!("Not found"))?;
                    if config.notion.mcp && name == "koad" {
                         println!("DELEGATE: Use Notion MCP tool for named resource '{}' (ID: {}).", name, id);
                         return Ok(());
                    }
                    let mut args = vec!["skill".to_string(), "run".to_string(), "global/notion_sync.py".to_string(), "--".to_string()];
                    args.push("--page-id".to_string());
                    args.push(id.clone());
                    Command::new(env::current_exe()?).args(args).spawn()?.wait()?;
                }
            }
        }
        Commands::Stream { action } => {
            if config.notion.mcp {
                match action {
                    StreamAction::Post { topic, message, msg_type } => {
                        println!("DELEGATE: Use Notion MCP 'API-post-page' to post to Koad Stream (DB: {}). Topic: {}, Type: {}, Msg: {}", 
                                 config.notion.index.get("stream").unwrap_or(&"UNKNOWN".to_string()),
                                 topic, msg_type, message);
                    }
                    StreamAction::List { limit } => {
                        println!("DELEGATE: Use Notion MCP 'API-query-data-source' to list Koad Stream (DB: {}, Limit: {}).", 
                                 config.notion.index.get("stream").unwrap_or(&"UNKNOWN".to_string()),
                                 limit);
                    }
                }
                return Ok(());
            }
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
                for (cmd, args, _) in recent { db.remember("fact", &format!("Verified {} with: {}", cmd, args), Some(format!("auto,{}", scope)))?; }
            }
            let log_entry = format!("\n## {} - {}\n- Scope: {}\n- Project: {}\n", Local::now().format("%Y-%m-%d"), summary, scope, db.get_active_project(&env::current_dir().unwrap_or_default().to_string_lossy())?.map(|(n,_,_)| n).unwrap_or("General".into()));
            std::fs::OpenOptions::new().append(true).create(true).open(KoadConfig::get_log_path()?)?.write_all(log_entry.as_bytes())?;
            println!("Saveup complete.");
        }
        Commands::Scan { path } => {
            let t = path.unwrap_or_else(|| env::current_dir().unwrap_or(PathBuf::from(".")));
            println!("Scanning for Koad projects in {}...", t.display());
            
            // Use fd-find to locate all .koad directories quickly
            let output = Command::new("fdfind")
                .arg(".koad")
                .arg("--type").arg("d")
                .arg("--hidden")
                .arg("--absolute-path")
                .arg(&t)
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    let paths = String::from_utf8_lossy(&out.stdout);
                    let mut count = 0;
                    for line in paths.lines() {
                        let koad_path = PathBuf::from(line);
                        let project_root = koad_path.parent().unwrap_or(&koad_path);
                        let name = project_root.file_name().unwrap_or_default().to_string_lossy();
                        
                        if db.register_project(&name, &project_root.to_string_lossy()).is_ok() {
                            println!("[PASS] Registered: {} ({})", name, project_root.display());
                            count += 1;
                        }
                    }
                    println!("Scan complete. {} project(s) registered.", count);
                },
                _ => {
                    // Fallback to single directory check if fd fails or is missing
                    if t.join(".koad").exists() {
                        db.register_project(&t.file_name().unwrap_or_default().to_string_lossy(), &t.to_string_lossy())?;
                        println!("Project registered.");
                    } else {
                        println!("[INFO] No .koad directories found.");
                    }
                }
            }
        }
        Commands::Note { text } => {
            db.save_note(&text)?;
            println!("Note saved to KoadDB.");
        }
        Commands::Brainstorm { text, rant } => {
            db.save_brainstorm(&text, rant)?;
            println!("{} saved to KoadDB.", if rant { "Rant" } else { "Brainstorm" });
        }
        Commands::Notes { limit } => {
            let notes = db.get_notes(limit)?;
            println!("--- Ian's Notes ---");
            if notes.is_empty() { println!("No notes found."); }
            for (id, content, ts) in notes { println!("[ID:{}] ({}) {}", id, ts, content); }
        }
        Commands::Brainstorms { limit } => {
            let items = db.get_recent_brainstorms(limit)?;
            println!("--- Ian's Brainstorms & Rants ---");
            if items.is_empty() { println!("No brainstorms found."); }
            for (content, cat, ts) in items { println!("[{}] ({}) {}", cat.to_uppercase(), ts, content); }
        }
        Commands::Dispatch { command, args } => {
            if !config.preferences.booster_enabled {
                println!("[WARN] Cognitive Booster is disabled in koad.json. Commands will be queued but not executed until 'koad serve' is running.");
            }
            db.dispatch(&command, args)?;
            println!("Command dispatched to Koad Spine.");
        }
        Commands::Publish { message } => {
            if !is_admin { anyhow::bail!("Admin only."); }
            let h = KoadConfig::get_home()?;
            let m = message.unwrap_or_else(|| format!("KoadOS Sync - {}", Local::now().format("%Y-%m-%d %H:%M")));
            
            // Detect current branch
            let branch_output = Command::new("git").arg("-C").arg(&h).arg("rev-parse").arg("--abbrev-ref").arg("HEAD").output()?;
            let branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

            Command::new("git").arg("-C").arg(&h).arg("add").arg(".").spawn()?.wait()?;
            Command::new("git").arg("-C").arg(&h).arg("commit").arg("-m").arg(&m).spawn()?.wait()?;
            Command::new("git").arg("-C").arg(&h).arg("push").arg("origin").arg(&branch).spawn()?.wait()?;
            println!("Published to branch: {}", branch);
        }
        Commands::Whoami => {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let current_path_str = current_dir.to_string_lossy().to_string();
            let tags = detect_context_tags(&current_dir);
            let user = env::var("USER").unwrap_or_else(|_| "Partner".into());
            
            println!("--- KoadOS Partnership ---");
            println!("Partner:   {}", user);
            println!("Persona:   {} ({})", config.identity.name, config.identity.role);
            println!("Status:    Active / Booted");
            println!("Bio:       {}", config.identity.bio);
            
            if let Some((p_name, role, stack)) = db.get_active_project(&current_path_str)? {
                println!("\n[Current Project Context]");
                println!("Project:   {}", p_name);
                if let Some(r) = role { println!("Role:      {}", r); }
                if let Some(s) = stack { println!("Stack:     {}", s); }
            }
            
            println!("\n[Environment]");
            println!("Context:   {}", tags.join(", "));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_detect_context_tags() {
        let path = Path::new("/tmp/some-rust-project/Cargo.toml");
        let tags = detect_context_tags(path);
        assert!(tags.contains(&"rust".to_string()));
        
        let path = Path::new("/tmp/some-node-project/package.json");
        let tags = detect_context_tags(path);
        assert!(tags.contains(&"node".to_string()));
    }

    #[test]
    fn test_gh_pat_resolution() {
        let path = Path::new("/home/ideans/data/personal/project1");
        let (var, label) = get_gh_pat_for_path(path);
        assert_eq!(var, "GITHUB_PERSONAL_PAT");
        assert_eq!(label, "Personal");

        let path = Path::new("/home/ideans/data/skylinks/project2");
        let (var, label) = get_gh_pat_for_path(path);
        assert_eq!(var, "GITHUB_SKYLINKS_PAT");
        assert_eq!(label, "Skylinks");
    }

    #[test]
    fn test_gdrive_token_resolution() {
        let path = Path::new("/home/ideans/data/personal/project1");
        let (var, label) = get_gdrive_token_for_path(path);
        assert_eq!(var, "GDRIVE_PERSONAL_TOKEN");
        assert_eq!(label, "Personal");

        let path = Path::new("/home/ideans/data/skylinks/project2");
        let (var, label) = get_gdrive_token_for_path(path);
        assert_eq!(var, "GDRIVE_SKYLINKS_TOKEN");
        assert_eq!(label, "Skylinks");
    }
}
