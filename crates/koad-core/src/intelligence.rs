use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A distilled unit of high-signal knowledge extracted from session context.
/// FactCards are durable (L3) and searchable across agent sessions.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FactCard {
    pub id: uuid::Uuid,
    /// The source agent who extracted the fact.
    pub source_agent: String,
    /// The session ID where the fact was born.
    pub session_id: String,
    /// High-level domain (e.g., "architecture", "user_preference", "bug_fix").
    pub domain: String,
    /// The distilled knowledge content.
    pub content: String,
    /// Reliability score (0.0 to 1.0).
    pub confidence: f32,
    /// Associated tags for hybrid search.
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    /// Time-to-live (0 = permanent).
    pub ttl_seconds: i32,
}

/// A living summary of a session's history used for L2 token compression.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContextSummary {
    pub session_id: String,
    /// The compressed narrative of the session so far.
    pub summary: String,
    /// Number of turns represented in this summary.
    pub turn_count: usize,
    /// The last message ID incorporated into the summary.
    pub last_message_id: String,
    pub updated_at: DateTime<Utc>,
}

impl FactCard {
    pub fn new(agent: &str, session: &str, domain: &str, content: &str) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            source_agent: agent.to_string(),
            session_id: session.to_string(),
            domain: domain.to_string(),
            content: content.to_string(),
            confidence: 1.0,
            tags: Vec::new(),
            created_at: Utc::now(),
            ttl_seconds: 0,
        }
    }
}
