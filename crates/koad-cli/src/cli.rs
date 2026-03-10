use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "koad")]
#[command(
    about = "The KoadOS Control Plane: Orchestrating the Neural Grid",
    long_about = "The primary interface for KoadOS agents and the Admiral. Manages session lifecycle, intellectual memory, and system-wide orchestration."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Set the session authorization tier (admiral, captain, officer, crew).
    /// If omitted, KoadOS will attempt to auto-resolve based on the agent name.
    #[arg(short, long, default_value = "admin")]
    pub role: String,

    /// Bypass pre-flight system integrity checks. Use only during recovery.
    #[arg(long)]
    pub no_check: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Wake a KAI and initialize a neural link session.
    Boot {
        /// The name of the agent to wake (e.g., Sky, Pippin).
        #[arg(short, long)]
        agent: String,

        /// Enable path-aware project context detection.
        #[arg(short, long)]
        project: bool,

        /// Target a specific task ID for the session.
        #[arg(short, long)]
        task: Option<String>,

        /// Output session metadata in a compact, pipe-friendly format.
        #[arg(short, long)]
        compact: bool,

        /// Force boot: take over an existing session for the same agent (Sovereign agents only).
        /// Use when a prior session is orphaned and cannot be cleanly logged out.
        #[arg(long)]
        force: bool,
    },

    /// Core system management, orchestration, and recovery.
    System {
        #[command(subcommand)]
        action: SystemAction,
    },

    /// Intellectual memory operations and knowledge retrieval.
    Intel {
        #[command(subcommand)]
        action: IntelAction,
    },

    /// Fleet-wide project coordination and board synchronization.
    Fleet {
        #[command(subcommand)]
        action: FleetAction,
    },

    /// Integration bridges for cloud ecosystems (GCP, Airtable, Notion).
    Bridge {
        #[command(subcommand)]
        action: BridgeAction,
    },

    /// Asynchronous agent-to-agent messaging (A2A-S).
    Signal {
        #[command(subcommand)]
        action: SignalAction,
    },

    /// Display real-time system telemetry and grid integrity.
    Status {
        /// Output telemetry data as JSON.
        #[arg(short, long)]
        json: bool,

        /// Perform an exhaustive diagnostic sweep (Ghost detection, Resource allocation).
        #[arg(short, long)]
        full: bool,
    },

    /// Perform a comprehensive system health check and self-healing sweep.
    Doctor {
        /// Attempt to fix any identified minor issues.
        #[arg(short, long)]
        fix: bool,
    },

    /// Manage and sync the GitHub Command Deck (Project Board).
    Board {
        #[command(subcommand)]
        action: BoardAction,
    },

    /// High-level project mapping and registration.
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },

    /// Reveal active agent persona, bio, and authorization rank.
    Whoami,

    /// Launch the terminal-native monitoring dashboard (TUI).
    Dash,

    /// Start or manage the Autonomic Watchdog.
    Watchdog {
        /// Run the watchdog as a background daemon.
        #[arg(short, long)]
        daemon: bool,
    },

    /// Perform a deep audit of the agent's internal cognitive layers.
    Cognitive,

    /// Gracefully logout and untether the current session.
    Logout {
        /// Explicit session ID to terminate.
        #[arg(short, long)]
        session: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SystemAction {
    /// Initialize a new KoadOS environment in the current path.
    Init {
        /// Force initialization, overwriting existing configs.
        #[arg(short, long)]
        force: bool,
    },

    /// Display active credentials and path-aware PAT mapping.
    Auth,

    /// Inspect or modify the global system configuration.
    Config {
        #[command(subcommand)]
        action: Option<ConfigAction>,

        /// Output configuration as JSON.
        #[arg(short, long)]
        json: bool,
    },

    /// Rebuild and redeploy the KoadOS core from source.
    Refresh {
        /// Restart services (Spine, Gateway) after successful build.
        #[arg(short, long)]
        restart: bool,
    },

    /// Execute the Sovereign Save Protocol (Total State Checkpoint).
    Save {
        /// Create a full durable backup (Database + Git commit).
        #[arg(short, long)]
        full: bool,
    },

    /// Apply an atomic, surgical patch to a file.
    Patch {
        /// Target file path.
        path: Option<PathBuf>,
        /// Regex search pattern.
        #[arg(short, long)]
        search: Option<String>,
        /// Replacement string.
        #[arg(short, long)]
        replace: Option<String>,
        /// JSON payload for bulk patching.
        #[arg(long)]
        payload: Option<String>,
        /// Enable fuzzy matching for complex diffs.
        #[arg(short, long)]
        fuzzy: bool,
        /// Show changes without modifying the file system.
        #[arg(long)]
        dry_run: bool,
    },

    /// Perform a 5-pass cognitive efficiency audit.
    Tokenaudit {
        /// Remove audit artifacts after completion.
        #[arg(short, long)]
        cleanup: bool,
    },

    /// Spawn a new GitHub issue using a system template.
    Spawn {
        /// Template name (e.g., bug, feature, research).
        #[arg(short, long, default_value = "feature")]
        template: String,
        /// Issue title.
        #[arg(short, long)]
        title: String,
        /// Complexity weight (trivial, standard, complex).
        #[arg(short, long, default_value = "standard")]
        weight: String,
        /// Describe the high-level goal or problem.
        #[arg(short, long)]
        objective: Option<String>,
        /// Define the specific architectural or functional scope.
        #[arg(short, long)]
        scope: Option<String>,
        /// Specific labels to apply.
        #[arg(short, long)]
        labels: Vec<String>,
    },

    /// Bulk import data (Markdown/CSV) into KoadOS subsystems.
    Import {
        /// Source file path.
        source: PathBuf,
        /// Data format (md, csv). [default: md]
        #[arg(short, long, default_value = "md")]
        format: String,
        /// Custom regex delimiter for chunking.
        #[arg(short, long)]
        delimiter: Option<String>,
        /// Destination route (github-issues, hydration).
        #[arg(short, long, default_value = "github-issues")]
        route: ImportRoute,
        /// Issue template to use (for github-issues route).
        #[arg(short, long)]
        template: Option<String>,
        /// Labels to apply to imported items.
        #[arg(short, long)]
        labels: Vec<String>,
        /// Preview changes without persisting.
        #[arg(long)]
        dry_run: bool,
    },

    /// Acquire a distributed lock on a specific system sector.
    Lock {
        /// The sector or resource name to lock.
        sector: String,
        /// TTL in seconds for the lock. [default: 300]
        #[arg(short, long, default_value_t = 300)]
        ttl: u64,
    },

    /// Release a distributed lock on a specific system sector.
    Unlock {
        /// The sector or resource name to unlock.
        sector: String,
    },

    /// Maintain an active neural link session via periodic heartbeats.
    Heartbeat {
        /// Run as a background daemon process.
        #[arg(short, long)]
        daemon: bool,
        /// Target session ID (omit to use environment).
        #[arg(short, long)]
        session: Option<String>,
    },

    /// Manage and hydrate an agent's transient context.
    Context {
        #[command(subcommand)]
        action: ContextAction,
    },
}

#[derive(Subcommand)]
pub enum ContextAction {
    /// Inject a file or raw text into an agent's hot context.
    Hydrate {
        /// Target session ID (omit for current).
        #[arg(short, long)]
        session: Option<String>,
        /// Path to a file to hydrate.
        #[arg(short, long)]
        path: Option<PathBuf>,
        /// Raw text to hydrate (if path is omitted).
        #[arg(short, long)]
        text: Option<String>,
        /// TTL in seconds (0 = session-persistent). [default: 0]
        #[arg(short = 'L', long, default_value_t = 0)]
        ttl: i32,
    },
    /// Purge all volatile context for a session.
    Flush {
        /// Target session ID (omit for current).
        #[arg(short, long)]
        session: Option<String>,
    },
    /// List available context quicksaves.
    List {
        /// Filter by agent name.
        #[arg(short, long)]
        agent: Option<String>,
    },
    /// Restore a session's hot context from a quicksave.
    Restore {
        /// Snapshot ID.
        id: String,
        /// Target session ID (omit for current).
        #[arg(short, long)]
        session: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a dynamic configuration value in Redis (Hot Config).
    Set { key: String, value: String },
    /// Get a specific configuration value.
    Get { key: String },
    /// List all extra configuration keys.
    List,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum ImportRoute {
    GithubIssues,
    Hydration,
    Knowledge,
}

#[derive(Subcommand)]
pub enum IntelAction {
    /// Query the collective knowledge bank.
    Query {
        /// Search term or regex pattern.
        term: String,
        /// Maximum results to return.
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        /// Filter by specific tags.
        #[arg(short, long)]
        tags: Option<String>,
        /// Filter by agent identity (Captain's Oversight).
        #[arg(short, long)]
        agent: Option<String>,
    },

    /// Commit a fact or learning to the durable memory bank.
    Remember {
        #[command(subcommand)]
        category: MemoryCategory,
    },

    /// Record a persona-specific reflection or architectural thought.
    Ponder {
        /// The reflection content.
        text: String,
        /// Optional classification tags.
        #[arg(short, long)]
        tags: Option<String>,
    },

    /// Access the KoadOS Field Guide for a specific topic.
    Guide { topic: Option<String> },

    /// Perform a deep recursive scan of the workspace for project roots.
    Scan { path: Option<PathBuf> },

    /// Introspect on cognitive health and learning status.
    Mind {
        #[command(subcommand)]
        action: MindAction,
    },

    /// Retrieve a precise line-range snippet from a file (Spine-cached).
    Snippet {
        /// Target file path.
        path: PathBuf,
        /// Start line (1-indexed).
        #[arg(short, long)]
        start: i32,
        /// End line (inclusive).
        #[arg(short, long)]
        end: i32,
        /// Force reload from disk, bypassing Spine cache.
        #[arg(short, long)]
        bypass: bool,
    },
}

#[derive(Subcommand)]
pub enum MemoryCategory {
    /// A persistent system fact.
    Fact {
        text: String,
        #[arg(short, long)]
        tags: Option<String>,
    },
    /// A technical insight or discovery.
    Learning {
        text: String,
        #[arg(short, long)]
        tags: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum FleetAction {
    /// Manage the high-level Command Deck.
    Board {
        #[command(subcommand)]
        action: BoardAction,
    },
    /// Low-level project mapping.
    Project {
        #[command(subcommand)]
        action: ProjectAction,
    },
    /// Atomic task tracking and state transitions.
    Issue {
        #[command(subcommand)]
        action: IssueAction,
    },
}

#[derive(Subcommand)]
pub enum BoardAction {
    /// Display current project board items.
    Status {
        /// Only show 'In Progress' and 'Todo' items.
        #[arg(short, long)]
        active: bool,
    },
    /// Perform a 2-way sync between GitHub and the Local Memory Bank.
    Sync {
        #[arg(long)]
        dry_run: bool,
    },
    /// Transition a node to 'Done' on the Command Deck.
    Done { id: i32 },
    /// Re-open a node or move to 'Todo'.
    Todo { id: i32 },
    /// Run a Strategic Design Review (SDR).
    Sdr,
    /// Verify a node's status against the Command Deck.
    Verify { id: i32 },
}

#[derive(Subcommand)]
pub enum ProjectAction {
    /// List all registered projects in the Master Map.
    List,
    /// Manually register a new project root.
    Register {
        /// Project identifier.
        name: String,
        /// Physical directory path.
        path: Option<PathBuf>,
    },
    /// Update project health and branch metadata.
    Sync { id: Option<i32> },
    /// Display detailed project diagnostics.
    Info { id: i32 },
    /// Mark a project as retired or inactive.
    Retire { id: i32 },
}

#[derive(Subcommand)]
pub enum IssueAction {
    /// Track an existing GitHub issue in the local task graph.
    Track { number: i32, description: String },
    /// Advance an issue through the KoadOS Canon steps (1-9).
    Move { number: i32, step: i32 },
    /// Authorize implementation or closure (Admin/Captain only).
    Approve { number: i32 },
    /// Close an issue locally and on GitHub.
    Close { number: i32 },
    /// Show detailed sovereignty status for an issue.
    Status { number: i32 },
}

#[derive(Subcommand)]
pub enum BridgeAction {
    /// Interface with Google Cloud Platform.
    Gcloud,
    /// Synchronize data with Airtable.
    Airtable,
    /// Interface with Notion (Optimized Native Bridge).
    Notion {
        #[command(subcommand)]
        action: NotionAction,
    },
    /// Execute a global cloud-to-local sync.
    Sync,
    /// Manage Google Drive file anchors.
    Drive,
    /// Post a high-priority event to the KoadStream.
    Stream {
        #[command(subcommand)]
        action: StreamAction,
    },
    /// Manage and execute specialized KoadOS Skills.
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
    /// Publish local changes to the remote grid (Git Push).
    Publish {
        /// Commit message.
        #[arg(short, long)]
        message: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum NotionAction {
    /// Read a page's content as surgically parsed Markdown.
    Read {
        /// The Notion Page ID.
        id: String,
    },
    /// Post a high-priority message to the KoadStream.
    Stream {
        /// The message to post.
        message: String,
        /// Target agent (e.g., Sky, Koad, Noti). [default: Koad]
        #[arg(short, long, default_value = "Koad")]
        target: String,
        /// Priority level (Low, Medium, High). [default: Medium]
        #[arg(short, long, default_value = "Medium")]
        priority: String,
    },
}

#[derive(Subcommand)]
pub enum StreamAction {
    /// Broadcast a message to the Neural Bus.
    Post {
        /// Topic or source identifier.
        topic: String,
        /// Event payload.
        message: String,
        /// Severity level (INFO, WARN, ERROR, CRITICAL).
        #[arg(short, long, default_value = "INFO")]
        msg_type: String,
    },
}

#[derive(Subcommand)]
pub enum SkillAction {
    /// List all currently available Skills.
    List,
    /// Execute a specific Skill by name.
    Run { name: String, args: Vec<String> },
}

#[derive(Subcommand)]
pub enum MindAction {
    /// Display cognitive health and learning metrics.
    Status,
    /// Capture a manual identity snapshot.
    Snapshot,
    /// Integrate a new structured insight into the Mind.
    Learn {
        /// Technical domain (e.g., rust, ops, architecture).
        domain: String,
        /// High-level summary of the insight.
        summary: String,
        /// Detailed technical breakdown.
        #[arg(short, long)]
        detail: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum SignalAction {
    /// Send a signal to another agent.
    Send {
        /// Target agent name.
        target: String,
        /// Message content.
        #[arg(short, long)]
        message: String,
        /// Priority (low, standard, high, critical).
        #[arg(short, long, default_value = "standard")]
        priority: String,
    },
    /// List pending signals for the current agent.
    List {
        /// Show all signals including read and archived.
        #[arg(short, long)]
        all: bool,
    },
    /// Read a specific signal.
    Read {
        /// Signal ID.
        id: String,
    },
    /// Archive a signal.
    Archive {
        /// Signal ID.
        id: String,
    },
}
