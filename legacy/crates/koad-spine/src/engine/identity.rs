use koad_core::config::KoadConfig;
use crate::engine::storage_bridge::KoadStorageBridge;
use chrono::{DateTime, Duration, Utc};
use fred::interfaces::{HashesInterface, LuaInterface};
use koad_proto::spine::v1::LeaseInfo;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use koad_core::identity::Rank;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KAILease {
    pub identity_name: String,
    pub session_id: String,
    pub driver_id: String,
    pub model_tier: i32,
    pub rank: Rank,
    pub expires_at: DateTime<Utc>,
    pub is_sovereign: bool,
    /// Populated from schema v3.2.1+. Defaults to empty string for leases
    /// created before body_id was introduced (safe for heartbeat renewal).
    #[serde(default)]
    pub body_id: String,
}

const ACQUIRE_LEASE_LUA: &str = r#"
    local lease_key = KEYS[1]
    local state_key = KEYS[2]
    local now = tonumber(ARGV[1])
    local lease_data = ARGV[2]
    local force = ARGV[3] == "true"

    local existing = redis.call("HGET", state_key, lease_key)
    if existing then
        local lease = cjson.decode(existing)
        local expires_at = lease["expires_at"]
        -- Convert ISO8601-ish to timestamp for comparison or rely on logic
        -- For simplicity in Lua, we check if it's currently active based on Spine's pre-check
        -- but the real safety is in the atomic HSET.
        if not force then
            return {err = "IDENTITY_LOCKED"}
        end
    end

    redis.call("HSET", state_key, lease_key, lease_data)
    return "OK"
"#;

const HEARTBEAT_LUA: &str = r#"
    local lease_key = KEYS[1]
    local state_key = KEYS[2]
    local session_id = ARGV[1]
    local expiry_secs = tonumber(ARGV[2])
    local now_iso = ARGV[3]

    local data = redis.call("HGET", state_key, lease_key)
    if data then
        local lease = cjson.decode(data)
        if lease["session_id"] == session_id then
            -- Update expiry and return
            lease["expires_at"] = now_iso
            redis.call("HSET", state_key, lease_key, cjson.encode(lease))
            return "OK"
        end
    end
    return {err = "LEASE_MISMATCH"}
"#;

pub struct KAILeaseManager {
    storage: Arc<KoadStorageBridge>,
    config: Arc<KoadConfig>,
}

impl KAILeaseManager {
    pub fn new(storage: Arc<KoadStorageBridge>, config: Arc<KoadConfig>) -> Self {
        Self { storage, config }
    }

    pub fn get_config(&self) -> Arc<KoadConfig> {
        self.config.clone()
    }

    pub async fn acquire_lease(
        &self,
        kai_name: &str,
        session_id: &str,
        driver_id: &str,
        model_tier: i32,
        body_id: &str,
        force: bool,
        rank: Rank,
        is_sovereign: bool,
    ) -> anyhow::Result<LeaseInfo> {
        let key = format!("koad:kai:{}:lease", kai_name);
        
        let requires_t1 = is_sovereign || rank == Rank::Officer;
        
        if requires_t1 && model_tier > 1 {
            anyhow::bail!("COGNITIVE_REJECTION: Rank '{:?}' agent '{}' requires Tier 1 Admin driver. (Requested: Tier {})", rank, kai_name, model_tier);
        }

        // 2. Atomic Commit via Lua
        let expiry_secs = self.config.sessions.lease_duration_secs;
        let expires_at = Utc::now() + Duration::seconds(expiry_secs as i64);

        let lease = KAILease {
            identity_name: kai_name.to_string(),
            session_id: session_id.to_string(),
            driver_id: driver_id.to_string(),
            model_tier,
            rank,
            expires_at,
            is_sovereign,
            body_id: body_id.to_string(),
        };

        let lease_json = serde_json::to_string(&lease)?;
        let now_ts = Utc::now().timestamp();

        // Execute Lua Script for Atomic Lease
        let result: String = self.storage.redis.pool.eval(
            ACQUIRE_LEASE_LUA,
            vec![key.clone(), "koad:state".to_string()],
            vec![now_ts.to_string(), lease_json, force.to_string()]
        ).await?;

        if result != "OK" {
            anyhow::bail!("IDENTITY_LOCKED: Atomic lease acquisition failed for {}", kai_name);
        }

        info!(
            "KAI Lease Acquired (Atomic): {} -> Session {} (Duration: {}s)",
            kai_name, session_id, expiry_secs
        );

        Ok(LeaseInfo {
            lock_id: format!("{}:{}", kai_name, session_id),
            expires_at: Some(prost_types::Timestamp {
                seconds: expires_at.timestamp(),
                nanos: expires_at.timestamp_subsec_nanos() as i32,
            }),
            is_sovereign,
        })
    }

    pub async fn release_lease(&self, kai_name: &str, session_id: &str) -> anyhow::Result<()> {
        let key = format!("koad:kai:{}:lease", kai_name);
        let existing: Option<String> = self.storage.redis.pool.hget("koad:state", &key).await?;

        if let Some(data) = existing {
            let lease: KAILease = serde_json::from_str(&data)?;
            if lease.session_id == session_id {
                let _: () = self.storage.redis.pool.hdel("koad:state", &key).await?;
                info!("KAI Lease Released: {} (Session {})", kai_name, session_id);
            }
        }
        Ok(())
    }

    pub async fn heartbeat(&self, kai_name: &str, session_id: &str) -> anyhow::Result<()> {
        let key = format!("koad:kai:{}:lease", kai_name);
        let expiry_secs = self.config.sessions.lease_duration_secs;
        let expires_at = Utc::now() + Duration::seconds(expiry_secs as i64);
        let expires_iso = expires_at.to_rfc3339();

        let result: String = self.storage.redis.pool.eval(
            HEARTBEAT_LUA,
            vec![key, "koad:state".to_string()],
            vec![session_id.to_string(), expiry_secs.to_string(), expires_iso]
        ).await?;

        if result != "OK" {
            anyhow::bail!("HEARTBEAT_FAILED: No active lease found for {} with session {}", kai_name, session_id);
        }

        Ok(())
    }
}
