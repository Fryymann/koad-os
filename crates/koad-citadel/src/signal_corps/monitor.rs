use anyhow::Result;
use fred::interfaces::StreamsInterface;
use koad_core::utils::redis::RedisClient;
use std::sync::Arc;
use tracing::info;

/// Diagnostic harness for monitoring Signal Corps Redis Streams (#163).
pub struct StreamMonitor {
    redis: Arc<RedisClient>,
    stream_prefix: String,
}

impl StreamMonitor {
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
    pub async fn tail(
        &self,
        topic: &str,
        from_id: &str,
        count: u64,
    ) -> Result<Vec<StreamEntry>> {
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

#[derive(Debug)]
pub struct StreamInfo {
    pub topic: String,
    pub length: u64,
    pub stream_key: String,
}

#[derive(Debug)]
pub struct StreamEntry {
    pub entry_id: String,
    pub payload: String,
    pub trace_id: String,
    pub actor: String,
}
