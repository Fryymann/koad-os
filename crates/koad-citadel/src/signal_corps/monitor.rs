//! Signal Corps: Stream Monitor
//!
//! Diagnostic harness for inspecting [`koad:stream:*`] Redis Streams in real-time.
//! Provides stream length queries and tail-reads for tracing inter-agent signals.

use anyhow::Result;
use fred::interfaces::StreamsInterface;
use koad_core::utils::redis::RedisClient;
use std::sync::Arc;

/// Diagnostic harness for monitoring Signal Corps Redis Streams (#163).
pub struct StreamMonitor {
    redis: Arc<RedisClient>,
    stream_prefix: String,
}

impl StreamMonitor {
    /// Creates a new `StreamMonitor` scoped to the given stream prefix.
    pub fn new(redis: Arc<RedisClient>, stream_prefix: &str) -> Self {
        Self {
            redis,
            stream_prefix: stream_prefix.to_string(),
        }
    }

    /// Get stream info (length, first/last entry) for a topic.
    pub async fn stream_info(&self, topic: &str) -> Result<StreamInfo> {
        let key = format!("{}{}", self.stream_prefix, topic);
        let len: i64 = self.redis.pool.xlen(&key).await?;

        Ok(StreamInfo {
            topic: topic.to_string(),
            length: len as u64,
            stream_key: key,
        })
    }

    /// Tail a stream from a given entry ID (or "0" for all).
    /// Returns up to `count` entries.
    pub async fn tail(&self, topic: &str, from_id: &str, count: u64) -> Result<Vec<StreamEntry>> {
        let key = format!("{}{}", self.stream_prefix, topic);

        let results: Vec<(String, Vec<(String, String)>)> = self
            .redis
            .pool
            .xrange(&key, from_id, "+", Some(count))
            .await?;

        let entries = results
            .into_iter()
            .map(|(entry_id, fields)| {
                let field_map: std::collections::HashMap<String, String> =
                    fields.into_iter().collect();
                StreamEntry {
                    entry_id,
                    payload: field_map.get("payload").cloned().unwrap_or_default(),
                    trace_id: field_map.get("trace_id").cloned().unwrap_or_default(),
                    actor: field_map.get("actor").cloned().unwrap_or_default(),
                }
            })
            .collect();

        Ok(entries)
    }
}

/// Metadata about a Redis Stream topic.
#[derive(Debug)]
pub struct StreamInfo {
    /// The topic name (without the stream prefix).
    pub topic: String,
    /// Number of entries currently in the stream.
    pub length: u64,
    /// Full Redis key for the stream.
    pub stream_key: String,
}

/// A single decoded entry read from a Redis Stream.
#[derive(Debug)]
pub struct StreamEntry {
    /// The Redis stream entry ID (e.g. `"1714000000000-0"`).
    pub entry_id: String,
    /// The signal payload string.
    pub payload: String,
    /// The trace ID propagated from the originating [`TraceContext`].
    pub trace_id: String,
    /// The name of the agent that broadcast this signal.
    pub actor: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal_corps::streams::SignalCorps;
    use std::sync::Arc;
    use tempfile::tempdir;

    async fn make_redis(
        dir: &tempfile::TempDir,
    ) -> anyhow::Result<Arc<koad_core::utils::redis::RedisClient>> {
        let client =
            koad_core::utils::redis::RedisClient::new(dir.path().to_str().unwrap(), true).await?;
        Ok(Arc::new(client))
    }

    #[tokio::test]
    async fn test_stream_info_returns_zero_for_empty_stream() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let redis = make_redis(&dir).await?;
        let monitor = StreamMonitor::new(redis, "koad:stream:");

        let info = monitor.stream_info("test:empty").await?;

        assert_eq!(info.length, 0);
        assert_eq!(info.topic, "test:empty");
        Ok(())
    }

    #[tokio::test]
    async fn test_tail_shows_broadcast_entry_with_trace_and_actor() -> anyhow::Result<()> {
        let dir = tempdir()?;
        let redis = make_redis(&dir).await?;
        let corps = SignalCorps::new(redis.clone(), "koad:stream:", 100);
        let monitor = StreamMonitor::new(redis, "koad:stream:");

        corps
            .broadcast("test:ac1", "hello-world", "trace-abc", "agent-alpha")
            .await?;

        let entries = monitor.tail("test:ac1", "0", 10).await?;

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].trace_id, "trace-abc");
        assert_eq!(entries[0].actor, "agent-alpha");
        assert_eq!(entries[0].payload, "hello-world");
        Ok(())
    }
}
