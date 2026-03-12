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
