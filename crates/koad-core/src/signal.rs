//! Signal Corps: Streams
//!
//! Provides an interface for broadcasting and receiving inter-agent signals
//! using Redis Streams.

use anyhow::{Context, Result};
use fred::interfaces::StreamsInterface;
use crate::utils::redis::RedisClient;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// Redis Streams-based Signal Corps for inter-agent messaging.
#[derive(Clone)]
pub struct SignalCorps {
    redis: Arc<RedisClient>,
    stream_prefix: String,
    max_stream_len: i64,
}

impl SignalCorps {
    /// Creates a new `SignalCorps` instance.
    pub fn new(redis: Arc<RedisClient>, stream_prefix: &str, max_stream_len: i64) -> Self {
        Self {
            redis,
            stream_prefix: stream_prefix.to_string(),
            max_stream_len,
        }
    }

    fn stream_key(&self, topic: &str) -> String {
        format!("{}{}", self.stream_prefix, topic)
    }

    fn consumer_group(&self, agent_name: &str) -> String {
        format!("koad:cg:{}", agent_name)
    }

    pub async fn broadcast(
        &self,
        topic: &str,
        payload: &str,
        trace_id: &str,
        actor: &str,
    ) -> Result<String> {
        let key = self.stream_key(topic);
        let fields: Vec<(&str, &str)> = vec![("payload", payload), ("trace_id", trace_id), ("actor", actor)];
        let entry_id: String = self.redis.pool.xadd(&key, false, ("MAXLEN", "~", self.max_stream_len), "*", fields).await
            .with_context(|| format!("Failed to broadcast signal to topic '''{}'''", topic))?;
        Ok(entry_id)
    }

    pub async fn ensure_consumer_groups(&self, agent_name: &str, topics: &[String]) -> Result<()> {
        let group = self.consumer_group(agent_name);
        for topic in topics {
            let key = self.stream_key(topic);
            let result: Result<(), _> = self.redis.pool.xgroup_create(&key, &group, "$", true).await;
            if let Err(e) = result {
                if !e.to_string().contains("BUSYGROUP") { return Err(e).context("XGROUP CREATE failed"); }
            }
        }
        Ok(())
    }

    pub async fn read_messages(&self, agent_name: &str, topics: &[String], count: Option<u64>, block_ms: Option<u64>) -> Result<Vec<(String, String, HashMap<String, String>)>> {
        let group = self.consumer_group(agent_name);
        let keys: Vec<String> = topics.iter().map(|t| self.stream_key(t)).collect();
        let ids: Vec<&str> = vec![">"; keys.len()];
        let results: fred::types::XReadResponse<String, String, String, String> = self.redis.pool.xreadgroup_map(&group, agent_name, count, block_ms, false, keys, ids).await
            .context("XREADGROUP failed")?;
        let mut messages = Vec::new();
        for (stream_key, entries) in results {
            let topic = stream_key.strip_prefix(&self.stream_prefix).unwrap_or(&stream_key).to_string();
            for (entry_id, fields) in entries {
                let field_map: HashMap<String, String> = fields.into_iter().collect();
                messages.push((topic.clone(), entry_id, field_map));
            }
        }
        Ok(messages)
    }

    pub async fn ack(&self, agent_name: &str, topic: &str, entry_ids: &[String]) -> Result<()> {
        let key = self.stream_key(topic);
        let group = self.consumer_group(agent_name);
        let _: i64 = self.redis.pool.xack(&key, &group, entry_ids.to_vec()).await.context("XACK failed")?;
        Ok(())
    }
}
