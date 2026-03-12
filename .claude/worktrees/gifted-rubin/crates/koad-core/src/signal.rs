use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents an asynchronous message between KoadOS agents (KAI Officers).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GhostSignal {
    pub id: String,
    pub source_agent: String,
    pub target_agent: String,
    pub message: String,
    pub priority: SignalPriority,
    pub timestamp: DateTime<Utc>,
    pub metadata: HashMap<String, String>,
    pub status: SignalStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalPriority {
    Low,
    Standard,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SignalStatus {
    Pending,
    Read,
    Archived,
}

impl GhostSignal {
    pub fn new(source: String, target: String, message: String, priority: SignalPriority) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            source_agent: source,
            target_agent: target,
            message,
            priority,
            timestamp: Utc::now(),
            metadata: HashMap::new(),
            status: SignalStatus::Pending,
        }
    }
}
