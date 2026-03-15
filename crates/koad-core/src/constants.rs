/// Default address for the KoadOS Web Deck (Gateway)
pub const DEFAULT_GATEWAY_PORT: u32 = 3000;
pub const DEFAULT_GATEWAY_ADDR: &str = "0.0.0.0:3000";

pub const DEFAULT_GITHUB_OWNER: &str = "Fryymann";
pub const DEFAULT_GITHUB_REPO: &str = "koad-os";
pub const DEFAULT_GITHUB_PROJECT_NUMBER: u32 = 2;

/// Session & Reaper Defaults
pub const DEFAULT_REAPER_INTERVAL_SECS: u64 = 10;
pub const DEFAULT_LEASE_DURATION_SECS: u64 = 90;
pub const DEFAULT_DARK_TIMEOUT_SECS: u64 = 60;
pub const DEFAULT_PURGE_TIMEOUT_SECS: u64 = 300;
pub const DEFAULT_DEADMAN_TIMEOUT_SECS: u64 = 45;

/// Backup Paths (Relative to KOAD_HOME)
pub const BACKUP_DIR_SQLITE: &str = "backups/sqlite";
pub const BACKUP_DIR_REDIS: &str = "backups/redis";
pub const BACKUP_DIR_SYSTEM: &str = "backups/system";

/// Default address for the KoadOS Spine gRPC service
pub const DEFAULT_SPINE_GRPC_PORT: u32 = 50051;
pub const DEFAULT_SPINE_GRPC_ADDR: &str = "http://127.0.0.1:50051";

/// API Base URLs
pub const GITHUB_API_BASE: &str = "https://api.github.com";
pub const NOTION_API_BASE: &str = "https://api.notion.com/v1";
pub const DND_BEYOND_CHAR_SERVICE: &str =
    "https://character-service.dndbeyond.com/character/v5/character";

/// Default Redis socket filename
pub const DEFAULT_REDIS_SOCK: &str = "koad.sock";

/// Default Spine socket filename
pub const DEFAULT_SPINE_SOCK: &str = "kspine.sock";

/// Default Admin socket filename
pub const DEFAULT_ADMIN_SOCK: &str = "kadmin.sock";

/// Default PID filenames
pub const DEFAULT_SPINE_PID: &str = "kspine.pid";
pub const DEFAULT_GATEWAY_PID: &str = "kgateway.pid";

/// Redis Channels
pub const REDIS_CHANNEL_TELEMETRY: &str = "koad:telemetry";
pub const REDIS_CHANNEL_SESSIONS: &str = "koad:sessions";

/// Redis Keys
pub const REDIS_KEY_CONFIG: &str = "koad:config";
pub const REDIS_KEY_HEALTH_REGISTRY: &str = "koad:health_registry";
pub const REDIS_KEY_STATE: &str = "koad:state";
pub const REDIS_KEY_SYSTEM_STATS: &str = "system_stats";
pub const REDIS_KEY_CREW_MANIFEST: &str = "crew_manifest";
