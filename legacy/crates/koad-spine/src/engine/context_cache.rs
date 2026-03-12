use koad_core::config::KoadConfig;
use koad_core::utils::redis::RedisClient;
use fred::interfaces::{HashesInterface, KeysInterface};
use std::sync::Arc;
use tracing::{info, warn};

pub struct KoadContextCache {
    redis: Arc<RedisClient>,
    config: Arc<KoadConfig>,
}

impl KoadContextCache {
    pub fn new(redis: Arc<RedisClient>, config: Arc<KoadConfig>) -> Self {
        Self { redis, config }
    }

    pub async fn get_snippet(
        &self,
        path: &str,
        start_line: usize,
        end_line: usize,
        bypass_cache: bool,
        session: Option<&koad_core::session::AgentSession>,
    ) -> anyhow::Result<(String, usize, String)> {
        // 0. Path Validation (Sanctuary Rule)
        let full_path = self.config.validate_path(path, session)?;
        
        let cache_key = format!("koad:cache:file:{}", full_path.to_string_lossy());
        info!(
            "Snippet Request: path={:?}, range={}-{}",
            full_path, start_line, end_line
        );

        // 1. Check Cache
        if !bypass_cache {
            match self
                .redis
                .pool
                .hget::<Option<String>, _, _>(&cache_key, "content")
                .await
            {
                Ok(Some(cached_content)) => {
                    info!("Cache Hit: serving snippet from memory for {:?}", full_path);
                    return self.extract_range(&cached_content, start_line, end_line, "cache");
                }
                Ok(None) => info!("Cache Miss: key {} not found.", cache_key),
                Err(e) => warn!("Cache Error: {}", e),
            }
        }

        // 2. Read from Disk
        info!("Reading from disk: {:?}", full_path);
        if !full_path.exists() {
            anyhow::bail!("File not found: {:?}", full_path);
        }
        let full_content = std::fs::read_to_string(&full_path)?;

        // 3. Update Cache
        let _: () = self
            .redis
            .pool
            .hset(&cache_key, ("content", &full_content))
            .await?;
        let _: () = self.redis.pool.expire(&cache_key, 600).await?;

        self.extract_range(&full_content, start_line, end_line, "disk")
    }

    fn extract_range(
        &self,
        content: &str,
        start: usize,
        end: usize,
        source: &str,
    ) -> anyhow::Result<(String, usize, String)> {
        let lines: Vec<&str> = content.lines().collect();
        let total = lines.len();

        if total == 0 {
            return Ok(("".to_string(), 0, source.to_string()));
        }

        let start_idx = if start > 0 { start - 1 } else { 0 };
        let end_idx = if end > total { total } else { end };

        if start_idx >= total {
            return Ok(("".to_string(), total, source.to_string()));
        }

        let snippet = lines[start_idx..end_idx].join(
            "
",
        );
        Ok((snippet, total, source.to_string()))
    }
}
