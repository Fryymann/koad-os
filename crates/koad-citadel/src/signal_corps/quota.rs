//! Signal Corps: Quota Validator
//!
//! Provides a sliding-window rate limiter for agent signal broadcasts using
//! Redis sorted sets.

use anyhow::{Context, Result};
use fred::interfaces::{KeysInterface, SortedSetsInterface};
use koad_core::utils::redis::RedisClient;
use std::sync::Arc;
use tonic::Status;
use tracing::warn;

/// Rate limiter for Signal Corps broadcasts.
#[derive(Clone)]
pub struct QuotaValidator {
    redis: Arc<RedisClient>,
    /// Maximum signals per agent per window.
    max_signals: u64,
    /// Window duration in seconds.
    window_secs: u64,
}

impl QuotaValidator {
    /// Creates a new `QuotaValidator`.
    pub fn new(redis: Arc<RedisClient>, max_signals: u64, window_secs: u64) -> Self {
        Self {
            redis,
            max_signals,
            window_secs,
        }
    }

    /// Generates the Redis key for an agent's quota tracking set.
    fn quota_key(&self, agent_name: &str) -> String {
        format!("koad:quota:{}", agent_name)
    }

    /// Check if the agent is within their signal quota.
    ///
    /// # Errors
    /// Returns `ResourceExhausted` if the agent has exceeded their broadcast quota.
    /// Returns `Internal` if Redis operations fail.
    pub async fn check_and_record(&self, agent_name: &str, signal_id: &str) -> Result<(), Status> {
        let key = self.quota_key(agent_name);
        let now = chrono::Utc::now().timestamp() as f64;
        let window_start = now - self.window_secs as f64;

        // Remove entries outside the window.
        // Pass f64 values directly so fred converts them to ZRangeBound::Score,
        // which is required for the BYSCORE validation in fred v9.
        let _: i64 = self
            .redis
            .pool
            .zremrangebyscore(&key, f64::NEG_INFINITY, window_start)
            .await
            .map_err(|e| Status::internal(format!("Quota check failed: {}", e)))?;

        // Count current entries in window
        let count: i64 = self
            .redis
            .pool
            .zcard(&key)
            .await
            .map_err(|e| Status::internal(format!("Quota count failed: {}", e)))?;

        if count as u64 >= self.max_signals {
            warn!(
                "QuotaValidator: Agent '{}' exceeded signal quota ({}/{})",
                agent_name, count, self.max_signals
            );
            return Err(Status::resource_exhausted(format!(
                "Signal quota exceeded: {}/{} signals in {}s window",
                count, self.max_signals, self.window_secs
            )));
        }

        // Record this signal
        let _: i64 = self
            .redis
            .pool
            .zadd(
                &key,
                None::<fred::types::SetOptions>,
                None::<fred::types::Ordering>,
                false,
                false,
                (now, signal_id.to_string()),
            )
            .await
            .map_err(|e| Status::internal(format!("Quota record failed: {}", e)))?;

        // Set TTL on the quota key so it auto-cleans
        let _: bool = self
            .redis
            .pool
            .expire(&key, self.window_secs as i64 * 2)
            .await
            .map_err(|e| Status::internal(format!("Quota TTL failed: {}", e)))?;

        Ok(())
    }

    /// Get current quota utilization for an agent.
    ///
    /// # Errors
    /// Returns an error if Redis operations fail.
    pub async fn get_utilization(&self, agent_name: &str) -> Result<(u64, u64)> {
        let key = self.quota_key(agent_name);
        let now = chrono::Utc::now().timestamp() as f64;
        let window_start = now - self.window_secs as f64;

        // Remove expired entries (f64 values required for fred v9 BYSCORE validation).
        let _: i64 = self
            .redis
            .pool
            .zremrangebyscore(&key, f64::NEG_INFINITY, window_start)
            .await
            .context("Failed to clear expired quota entries")?;

        let count: i64 = self
            .redis
            .pool
            .zcard(&key)
            .await
            .context("Failed to get quota card")?;
        Ok((count as u64, self.max_signals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tempfile::tempdir;

    async fn make_validator(
        dir: &tempfile::TempDir,
        max_signals: u64,
        window_secs: u64,
    ) -> anyhow::Result<QuotaValidator> {
        let client =
            koad_core::utils::redis::RedisClient::new(dir.path().to_str().unwrap(), true).await?;
        Ok(QuotaValidator::new(
            Arc::new(client),
            max_signals,
            window_secs,
        ))
    }

    #[tokio::test]
    async fn test_within_quota_allows_signals() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let validator = make_validator(&dir, 3, 60).await?;

        validator.check_and_record("agent-alpha", "sig-1").await?;
        validator.check_and_record("agent-alpha", "sig-2").await?;
        validator.check_and_record("agent-alpha", "sig-3").await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_quota_exhausted_returns_resource_exhausted() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let validator = make_validator(&dir, 3, 60).await?;

        validator.check_and_record("agent-beta", "sig-1").await?;
        validator.check_and_record("agent-beta", "sig-2").await?;
        validator.check_and_record("agent-beta", "sig-3").await?;

        let result = validator.check_and_record("agent-beta", "sig-4").await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err().code(), tonic::Code::ResourceExhausted);
        Ok(())
    }

    #[tokio::test]
    async fn test_get_utilization_reflects_recorded_signals() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let validator = make_validator(&dir, 3, 60).await?;

        validator.check_and_record("agent-gamma", "sig-1").await?;
        validator.check_and_record("agent-gamma", "sig-2").await?;

        let (used, max) = validator.get_utilization("agent-gamma").await?;

        assert_eq!(used, 2);
        assert_eq!(max, 3);
        Ok(())
    }
}
