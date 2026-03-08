use anyhow::Result;
use async_trait::async_trait;
use tracing::{info, warn, error};

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
    ) -> Result<Option<Self>> {
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
    pub async fn release(mut self) -> Result<()> {
        if self.active {
            if self.lock_client.unlock(&self.sector, &self.agent_name).await? {
                info!("SectorLock: Released '{}'", self.sector);
                self.active = false;
            } else {
                error!("SectorLock: Failed to release '{}' (ownership lost?)", self.sector);
            }
        }
        Ok(())
    }
}

// Note: Automatic release on Drop requires a synchronous drop, but distributed 
// unlocking is usually asynchronous. We recommend using explicit .release()
// or wrapping in a high-level macro that handles the async cleanup.

/// Helper macro for scoped locking.
/// Returns anyhow::Result<T>
#[macro_export]
macro_rules! with_sector_lock {
    ($client:expr, $sector:expr, $agent:expr, $ttl:expr, $body:block) => {
        async {
            let guard = $crate::utils::lock::SectorLockGuard::try_acquire($client, $sector, $agent, $ttl).await?;
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
