use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "koad")]
#[command(about = "The KoadOS Control Plane", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Set the role for the session (admin, pm, officer, crew)
    #[arg(short, long, default_value = "admin")]
    pub role: String,

    /// Skip pre-flight health checks
    #[arg(long)]
    pub no_check: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Wake a KAI and initialize a session
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
    /// Core system management and orchestration
    System {
        #[command(subcommand)]
        action: SystemAction,
    },
    /// Intellectual and memory operations
    Intel {
        #[command(subcommand)]
        action: IntelAction,
    },
    /// Fleet and project management
    Fleet {
        #[command(subcommand)]
        action: FleetAction,
    },
    /// Bridge and integration services
    Bridge {
        #[command(subcommand)]
        action: BridgeAction,
    },
    /// Check system telemetry and neural link status
    Status {
        #[arg(short, long)]
        json: bool,
        #[arg(short, long)]
        full: bool,
    },
    /// Display current persona and bio
    Whoami,
    /// Launch the TUI dashboard
    Dash,
}

#[derive(Subcommand)]
pub enum SystemAction {
    /// Initialize KoadOS in the current directory
    Init {
        #[arg(short, long)]
        force: bool,
    },
    /// Authenticate and verify credentials
    Auth,
    /// Show current system configuration
    Config {
        #[arg(short, long)]
        json: bool,
    },
    /// Refresh and rebuild the KoadOS environment
    Refresh {
        #[arg(short, long)]
        restart: bool,
    },
    /// Execute the Sovereign Save protocol
    Save {
        #[arg(short, long)]
        full: bool,
    },
    /// Apply an atomic patch to a file
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
pub enum IntelAction {
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
pub enum FleetAction {
    Board {
        #[command(subcommand)]
        action: BoardAction,
    },
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    Issue {
        #[command(subcommand)]
        action: IssueAction,
    },
}

#[derive(Subcommand)]
pub enum BridgeAction {
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
    Publish {
        #[arg(short, long)]
        message: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum MemoryCategory {
    Fact {
        text: String,
        #[arg(short, long)]
        tags: Option<String>,
    },
    Learning {
        text: String,
        #[arg(short, long)]
        tags: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum GcloudAction {
    List,
    Deploy { name: String },
}

#[derive(Subcommand)]
pub enum AirtableAction {
    Sync,
    List,
}

#[derive(Subcommand)]
pub enum SyncSource {
    Notion,
    Airtable,
    All,
}

#[derive(Subcommand)]
pub enum DriveAction {
    List,
    Download { id: String },
    Upload { path: PathBuf },
}

#[derive(Subcommand)]
pub enum StreamAction {
    Logs {
        #[arg(short, long)]
        topic: Option<String>,
    },
    Post {
        topic: String,
        message: String,
        #[arg(short, long, default_value = "INFO")]
        msg_type: String,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    List,
    Run {
        name: String,
        #[arg(last = true)]
        args: Vec<String>,
    },
}

#[derive(Subcommand)]
pub enum ProjectAction {
    List,
    Register {
        name: String,
        path: Option<PathBuf>,
    },
    Sync {
        id: Option<i32>,
    },
    Retire {
        id: i32,
    },
    Info {
        id: i32,
    },
}

#[derive(Subcommand)]
pub enum IssueAction {
    Track { number: i32, description: String },
    Move { number: i32, step: i32 },
    Approve { number: i32 },
    Close { number: i32 },
    Status { number: i32 },
}

#[derive(Subcommand)]
pub enum BoardAction {
    Status {
        #[arg(short, long)]
        active: bool,
    },
    Sync,
    Sdr,
    Done {
        id: i32,
    },
    Todo {
        id: i32,
    },
    Verify {
        id: i32,
    },
}

#[derive(Subcommand)]
pub enum MindAction {
    Status,
    Snapshot,
    Learn {
        domain: String,
        summary: String,
        #[arg(short, long)]
        detail: Option<String>,
    },
}
