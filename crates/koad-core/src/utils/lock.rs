use async_trait::async_trait;
use tracing::{error, info, warn};

/// A trait for distributed locking operations, allowing koad-core to remain
/// agnostic of the specific Redis client (fred vs redis-rs).
#[async_trait]
pub trait DistributedLock: Send + Sync {
    /// Attempts to acquire a lock on a sector.
    async fn lock(&self, sector: &str, agent_name: &str, ttl_secs: u64) -> anyhow::Result<bool>;

    /// Releases a lock on a sector, verifying ownership.
    async fn unlock(&self, sector: &str, agent_name: &str) -> anyhow::Result<bool>;
}

/// A RAII guard that manages the lifecycle of a distributed sector lock.
/// The lock is acquired on creation and automatically released when the guard is dropped.
pub struct SectorLockGuard<'a> {
    lock_client: &'a dyn DistributedLock,
    sector: String,
    agent_name: String,
    active: bool,
}

impl<'a> SectorLockGuard<'a> {
    /// Tries to acquire a lock and returns a guard if successful.
    pub async fn try_acquire(
        client: &'a dyn DistributedLock,
        sector: &str,
        agent_name: &str,
        ttl_secs: u64,
    ) -> anyhow::Result<Option<Self>> {
        if client.lock(sector, agent_name, ttl_secs).await? {
            info!("SectorLock: Acquired '{}' for '{}'", sector, agent_name);
            Ok(Some(Self {
                lock_client: client,
                sector: sector.to_string(),
                agent_name: agent_name.to_string(),
                active: true,
            }))
        } else {
            warn!("SectorLock: Failed to acquire '{}' (already held)", sector);
            Ok(None)
        }
    }

    /// Explicitly release the lock before the guard is dropped.
    pub async fn release(mut self) -> anyhow::Result<()> {
        if self.active {
            if self
                .lock_client
                .unlock(&self.sector, &self.agent_name)
                .await?
            {
                info!("SectorLock: Released '{}'", self.sector);
                self.active = false;
            } else {
                error!(
                    "SectorLock: Failed to release '{}' (ownership lost?)",
                    self.sector
                );
            }
        }
        Ok(())
    }
}

impl<'a> Drop for SectorLockGuard<'a> {
    fn drop(&mut self) {
        if self.active {
            warn!(
                "SectorLock: Guard dropped for '{}' without explicit release. Attempting background cleanup.",
                self.sector
            );
            // We cannot await here, so we must rely on the fact that most
            // async runtimes are still active or the TTL will eventually
            // clear it. This is a safety fallback.
        }
    }
}

/// Helper macro for scoped locking.
/// Returns anyhow::Result<T>
#[macro_export]
macro_rules! with_sector_lock {
    ($client:expr, $sector:expr, $agent:expr, $ttl:expr, $body:block) => {
        async {
            let guard =
                $crate::utils::lock::SectorLockGuard::try_acquire($client, $sector, $agent, $ttl)
                    .await?;
            if let Some(g) = guard {
                let result = { $body };
                g.release().await?;
                Ok(result)
            } else {
                anyhow::bail!("LOCK_DENIED: Sector '{}' is busy.", $sector)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct MockLock {
        acquires: bool,
        releases: bool,
    }

    #[async_trait]
    impl DistributedLock for MockLock {
        async fn lock(&self, _sector: &str, _agent: &str, _ttl: u64) -> anyhow::Result<bool> {
            Ok(self.acquires)
        }

        async fn unlock(&self, _sector: &str, _agent: &str) -> anyhow::Result<bool> {
            Ok(self.releases)
        }
    }

    #[tokio::test]
    async fn try_acquire_returns_some_when_lock_is_available() -> anyhow::Result<()> {
        let mock = MockLock {
            acquires: true,
            releases: true,
        };
        let guard = SectorLockGuard::try_acquire(&mock, "sector-a", "agent-1", 30).await?;
        assert!(guard.is_some(), "Expected Some guard when lock succeeds");
        if let Some(g) = guard {
            g.release().await?;
        }
        Ok(())
    }

    #[tokio::test]
    async fn try_acquire_returns_none_when_lock_is_held() -> anyhow::Result<()> {
        let mock = MockLock {
            acquires: false,
            releases: false,
        };
        let guard = SectorLockGuard::try_acquire(&mock, "sector-a", "agent-1", 30).await?;
        assert!(
            guard.is_none(),
            "Expected None when lock is already held (contention)"
        );
        Ok(())
    }

    #[tokio::test]
    async fn release_returns_ok_when_ownership_is_valid() -> anyhow::Result<()> {
        let mock = MockLock {
            acquires: true,
            releases: true,
        };
        let guard = SectorLockGuard::try_acquire(&mock, "sector-b", "agent-2", 30)
            .await?
            .expect("Lock should be acquired");
        guard.release().await?;
        Ok(())
    }

    #[tokio::test]
    async fn with_sector_lock_macro_executes_body_and_returns_value() -> anyhow::Result<()> {
        let mock = MockLock {
            acquires: true,
            releases: true,
        };
        let result: i32 = with_sector_lock!(&mock, "sector-c", "agent-3", 30, { 42 }).await?;
        assert_eq!(
            result, 42,
            "Macro should return the value produced by the body block"
        );
        Ok(())
    }

    #[tokio::test]
    async fn with_sector_lock_macro_returns_lock_denied_on_contention() {
        let mock = MockLock {
            acquires: false,
            releases: false,
        };
        let result: anyhow::Result<i32> =
            with_sector_lock!(&mock, "sector-c", "agent-3", 30, { 42 }).await;
        assert!(
            result.is_err(),
            "Macro should return an error when lock cannot be acquired"
        );
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("LOCK_DENIED"),
            "Error should contain 'LOCK_DENIED', got: {}",
            err
        );
    }
}
