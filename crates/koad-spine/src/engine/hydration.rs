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
    /// Implements the \"Governor\" logic: Hashing, Capping, and TTL.
    pub async fn hydrate(
        &self,
        session_id: &str,
        content: &str,
        file_path: Option<String>,
        ttl_seconds: i32,
    ) -> anyhow::Result<HotContextChunk> {
        let mut final_content = content.to_string();
        let mut tags = vec!["manual_hydration".to_string()];

        // 1. Reference Hydration Logic
        if let Some(ref path) = file_path {
            tags.push("file_reference".to_string());
            if final_content.is_empty() {
                let path_buf = std::path::PathBuf::from(path);
                if path_buf.exists() && path_buf.is_file() {
                    let meta = std::fs::metadata(&path_buf)?;
                    if meta.len() > self.context_cap as u64 {
                        info!("Sentinel: File {} too large for full hydration. Generating Virtual Chunk.", path);
                        final_content = format!("[VIRTUAL CHUNK] Reference: {}. Size: {} bytes. Type: Codebase Reference.", path, meta.len());
                        tags.push("virtual_chunk".to_string());
                    } else {
                        final_content = std::fs::read_to_string(&path_buf)?;
                    }
                }
            }
        }

        // 2. Governor: Character Cap Check (on final content)
        if final_content.len() > self.context_cap {
            anyhow::bail!(
                "Hydration Rejected: Content exceeds cap ({} > {})",
                final_content.len(),
                self.context_cap
            );
        }

        // 3. Governor: Content Hashing (SHA-256)
        let mut hasher = Sha256::new();
        hasher.update(final_content.as_bytes());
        let chunk_id = format!("{:x}", hasher.finalize());

        // 4. Governor: Duplicate Check
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
        }

        let chunk = HotContextChunk {
            chunk_id: chunk_id.clone(),
            content: final_content,
            file_path,
            ttl_seconds,
            significance_score: 1.0,
            tags,
            created_at: Utc::now(),
        };

        // 5. Persistence
        let chunk_json = serde_json::to_string(&chunk)?;
        let _: () = self
            .redis
            .pool
            .hset(&context_key, (&chunk_id, &chunk_json))
            .await?;

        info!(
            "Hydration Successful: Chunk {} injected into session {}",
            chunk_id, session_id
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
