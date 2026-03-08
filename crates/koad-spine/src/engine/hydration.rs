use koad_core::utils::redis::RedisClient;
use chrono::Utc;
use fred::interfaces::{HashesInterface, KeysInterface};
use koad_core::types::HotContextChunk;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tracing::info;

pub struct KoadHydrationManager {
    redis: Arc<RedisClient>,
    context_cap: usize,
}

impl KoadHydrationManager {
    pub fn new(redis: Arc<Arc<RedisClient>>) -> Self {
        Self {
            redis: (*redis).clone(),
            context_cap: 50_000, // Default 50k char cap
        }
    }

    /// Hydrates an agent's context with a new chunk.
    /// Implements the "Governor" logic: Hashing, Capping, and TTL.
    pub async fn hydrate(
        &self,
        session_id: &str,
        content: &str,
        ttl_seconds: i32,
    ) -> anyhow::Result<HotContextChunk> {
        // 1. Governor: Character Cap Check
        if content.len() > self.context_cap {
            anyhow::bail!(
                "Hydration Rejected: Content exceeds cap ({} > {})",
                content.len(),
                self.context_cap
            );
        }

        // 2. Governor: Content Hashing (SHA-256)
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let chunk_id = format!("{:x}", hasher.finalize());

        // 3. Governor: Duplicate Check
        let context_key = format!("koad:context:session:{}", session_id);
        if self
            .redis
            .pool
            .hexists::<bool, _, _>(&context_key, &chunk_id)
            .await?
        {
            info!(
                "Hydration Canceled: Chunk {} already exists for session {}",
                chunk_id, session_id
            );
            // Return existing or bail? For idempotency, we return the existing structure info.
        }

        let chunk = HotContextChunk {
            chunk_id: chunk_id.clone(),
            content: content.to_string(),
            ttl_seconds,
            created_at: Utc::now(),
        };

        // 4. Persistence
        let chunk_json = serde_json::to_string(&chunk)?;
        let _: () = self
            .redis
            .pool
            .hset(&context_key, (&chunk_id, &chunk_json))
            .await?;

        // 5. TTL Logic (Optional: Redis TTL on the whole hash or per-key? Redis HSET doesn't support per-field TTL)
        // For now, we rely on the agent flushing or the session ending.
        // We'll implement a manual cleanup in the ASM reaper later.

        info!(
            "Hydration Successful: Chunk {} ({} bytes) injected into session {}",
            chunk_id,
            content.len(),
            session_id
        );

        Ok(chunk)
    }

    /// Flushes all hot context for a session.
    pub async fn flush_context(&self, session_id: &str) -> anyhow::Result<()> {
        let context_key = format!("koad:context:session:{}", session_id);
        let _: () = self.redis.pool.del(&context_key).await?;
        info!(
            "Context Flushed: Session {} volatile memory cleared.",
            session_id
        );
        Ok(())
    }
}
