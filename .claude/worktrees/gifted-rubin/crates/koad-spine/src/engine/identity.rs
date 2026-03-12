use koad_core::config::KoadConfig;
use crate::engine::storage_bridge::KoadStorageBridge;
use chrono::{DateTime, Duration, Utc};
use fred::interfaces::{HashesInterface, LuaInterface};
use koad_core::storage::StorageBridge;
use koad_proto::spine::v1::LeaseInfo;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KAILease {
    pub identity_name: String,
    pub session_id: String,
    pub driver_id: String,
    pub model_tier: i32,
    pub expires_at: DateTime<Utc>,
    pub is_sovereign: bool,
    /// Populated from schema v3.2.1+. Defaults to empty string for leases
    /// created before body_id was introduced (safe for heartbeat renewal).
    #[serde(default)]
    pub body_id: String,
}

pub struct KAILeaseManager {
    storage: Arc<KoadStorageBridge>,
    config: Arc<KoadConfig>,
}

impl KAILeaseManager {
    pub fn new(storage: Arc<KoadStorageBridge>, config: Arc<KoadConfig>) -> Self {
        Self { storage, config }
    }

    pub async fn acquire_lease(
        &self,
        kai_name: &str,
        session_id: &str,
        driver_id: &str,
        model_tier: i32,
        body_id: &str,
    ) -> anyhow::Result<LeaseInfo> {
        let key = format!("koad:kai:{}:lease", kai_name);
        
        // 1. Pre-check for existing lease (Rust side for detailed error reporting)
        let existing: Option<String> = self.storage.redis.pool.hget("koad:state", &key).await?;
        if let Some(data) = existing {
            let lease: KAILease = serde_json::from_str(&data)?;
            if lease.expires_at > Utc::now() && lease.session_id != session_id {
                anyhow::bail!(
                    "IDENTITY_LOCKED: KAI '{}' is already active in Body {} (Session: {}, Driver: {}). \
                     Use `koad logout --session {}` to release, or `koad boot --force` to take over.",
                    kai_name,
                    lease.body_id,
                    lease.session_id,
                    lease.driver_id,
                    lease.session_id
                );
            }
        }

        // 2. Sovereign Guardrail (Tyr/Koad/Ian)
        let is_sovereign =
            kai_name == "Tyr" || kai_name == "Koad" || kai_name == "Ian" || kai_name == "TestKoad";
        
        if is_sovereign && model_tier > 1 {
            anyhow::bail!("COGNITIVE_REJECTION: Sovereign KAI '{}' requires Tier 1 Admin driver. (Requested: Tier {})", kai_name, model_tier);
        }

        // 3. Atomic Commit
        let expiry_secs = self.config.sessions.lease_duration_secs;
        let expires_at = Utc::now() + Duration::seconds(expiry_secs as i64);

        let lease = KAILease {
            identity_name: kai_name.to_string(),
            session_id: session_id.to_string(),
            driver_id: driver_id.to_string(),
            model_tier,
            expires_at,
            is_sovereign,
            body_id: body_id.to_string(),
        };

        // TODO: In v4.2, use Lua script here for 100% atomic HSETNX or session check
        self.storage
            .set_state(&key, serde_json::to_value(&lease)?, Some(model_tier))
            .await?;

        info!(
            "KAI Lease Acquired: {} -> Session {} (Duration: {}s)",
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

    pub async fn heartbeat(&self, session_id: &str) -> anyhow::Result<()> {
        let expiry_secs = self.config.sessions.lease_duration_secs;
        
        let all_state: std::collections::HashMap<String, String> =
            self.storage.redis.pool.hgetall("koad:state").await?;
        for (key, val) in all_state {
            if key.starts_with("koad:kai:") && key.ends_with(":lease") {
                if let Ok(mut lease) = serde_json::from_str::<KAILease>(&val) {
                    if lease.session_id == session_id {
                        // Extend lease
                        lease.expires_at = Utc::now() + Duration::seconds(expiry_secs as i64);
                        self.storage
                            .set_state(&key, serde_json::to_value(&lease)?, Some(lease.model_tier))
                            .await?;
                        return Ok(());
                    }
                }
            }
        }
        Ok(())
    }
}
