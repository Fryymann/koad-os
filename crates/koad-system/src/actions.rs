use clap::{Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone)]
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
        /// Restart Citadel services after successful build.
        #[arg(short, long)]
        restart: bool,
        /// Explicit confirmation to bypass the safety gate.
        #[arg(long)]
        confirm: bool,
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

    /// List all active distributed locks.
    Locks,

    /// Re-establish a neural link session after a disconnection or reboot.
    Reconnect {
        /// Explicit agent name to recover.
        #[arg(short, long)]
        agent: Option<String>,

        /// Perform a live re-sync using the active environment session ID.
        #[arg(short, long)]
        live: bool,
    },

    /// Trigger a manual backup of the system memory sectors.
    Backup {
        /// Sector to backup (all, sqlite, redis). [default: all]
        #[arg(short, long, default_value = "all")]
        source: String,
    },

    /// Tail or filter KoadOS log files.
    Logs {
        /// The service or component to filter by (e.g., citadel, cass, gateway).
        #[arg(short, long)]
        service: Option<String>,
        /// Number of lines to show from the end. [default: 50]
        #[arg(short, long, default_value_t = 50)]
        tail: usize,
        /// Follow log output in real-time.
        #[arg(short, long)]
        follow: bool,
    },

    /// Start the Citadel kernel and all dependent services (CASS).
    Start,

    /// Restart the Citadel kernel and all dependent services.
    Restart,

    /// Gracefully stop the Citadel kernel and all background services.
    Stop {
        /// Trigger a full state drain before stopping.
        #[arg(short, long)]
        drain: bool,
        /// Explicit confirmation to bypass the safety gate.
        #[arg(long)]
        confirm: bool,
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

    /// Prepare the Citadel for clean distribution — removes local state, logs, and databases.
    Scrub {
        /// Preview all targets without deleting anything.
        #[arg(long)]
        dry_run: bool,
        /// Skip the confirmation prompt (DANGER: immediately deletes targets).
        #[arg(long)]
        force: bool,
    },

    /// Manage and hydrate an agent's transient context.
    Context {
        #[command(subcommand)]
        action: ContextAction,
    },

    /// Force a 2-way synchronization between local reality (updates/log) and GitHub Project #6.
    BoardSync {
        /// Preview changes without modifying GitHub or CASS.
        #[arg(long)]
        dry_run: bool,
        /// Automatically create missing issues for local updates.
        #[arg(long)]
        auto_spawn: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
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
        /// Explicit confirmation to bypass the safety gate.
        #[arg(long)]
        confirm: bool,
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

#[derive(Subcommand, Debug, Clone)]
pub enum ConfigAction {
    /// Set a dynamic configuration value in Redis (Hot Config).
    Set { key: String, value: String },
    /// Get a specific configuration value.
    Get { key: String },
    /// List all extra configuration keys.
    List,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum ImportRoute {
    GithubIssues,
    Hydration,
    Knowledge,
}
