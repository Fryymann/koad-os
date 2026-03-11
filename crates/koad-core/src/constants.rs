/// Default address for the KoadOS Web Deck (Gateway)
pub const DEFAULT_GATEWAY_PORT: u32 = 3000;
pub const DEFAULT_GATEWAY_ADDR: &str = "0.0.0.0:3000";

pub const DEFAULT_GITHUB_OWNER: &str = "DoodzCode";
pub const DEFAULT_GITHUB_REPO: &str = "koad-os";

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
