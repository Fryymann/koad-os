use async_trait::async_trait;
use serde_json::Value;

/// The StorageBridge is the unified interface for KoadOS state.
/// It coordinates between the high-speed "Hot Path" (Redis) and
/// the durable "Cold Path" (SQLite).
#[async_trait]
pub trait StorageBridge: Send + Sync {
    /// Save a state object. This updates Redis immediately and
    /// marks it for eventual drain to SQLite.
    /// `caller_tier` is used for Cognitive Protection (CIP).
    async fn set_state(
        &self,
        key: &str,
        value: Value,
        caller_tier: Option<i32>,
    ) -> anyhow::Result<()>;

    /// Retrieve state. This checks Redis first, then falls back to
    /// SQLite if necessary (Hydration on demand).
    async fn get_state(&self, key: &str) -> anyhow::Result<Option<Value>>;

    /// Perform a "Full Drain": Synchronize all volatile Redis state into SQLite.
    async fn drain_all(&self) -> anyhow::Result<()>;

    /// Perform a "Full Hydration": Load critical persistent state from SQLite into Redis.
    async fn hydrate_all(&self) -> anyhow::Result<()>;
}

/// Metadata for state objects to track sync status.
pub struct StateMetadata {
    pub key: String,
    pub version: u64,
    pub is_volatile: bool,
    pub last_synced: Option<chrono::DateTime<chrono::Utc>>,
}
