use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a unit of work within the KoadOS ecosystem.
/// Every Ticket MUST have a corresponding GitHub Issue for external tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ticket {
    /// Unique internal identifier.
    pub id: uuid::Uuid,
    /// Associated GitHub issue number.
    pub github_issue: Option<u32>,
    /// Concise summary of the objective.
    pub title: String,
    /// Detailed description of the requirement or bug.
    pub problem: String,
    /// The high-level technical strategy for resolution.
    pub solution: String,
    /// Step-by-step implementation roadmap.
    pub implementation_plan: Vec<String>,
    /// Current lifecycle stage of the ticket.
    pub status: TicketStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TicketStatus {
    /// Initial draft state.
    Draft,
    /// Verified and ready for assignment.
    Open,
    /// Currently being implemented by an agent.
    InProgress,
    /// Implementation complete, awaiting verification.
    Testing,
    /// Successfully verified and merged.
    Resolved,
    /// Formally archived.
    Closed,
}

/// The environment in which a session or task is executing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EnvironmentType {
    Wsl,
    Windows,
    Remote,
    Unspecified,
}

/// A standardized log entry for system-wide telemetry and event tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogEntry {
    /// The UTC timestamp of the event.
    pub timestamp: DateTime<Utc>,
    /// The crate or module name that generated the log.
    pub source: String,
    /// The message content.
    pub message: String,
    /// The severity level (e.g., INFO, WARN, ERROR).
    pub level: String,
}

/// A chunk of transient context injected into an agent's memory.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HotContextChunk {
    /// Content hash or unique label to prevent duplicates.
    pub chunk_id: String,
    /// The actual context content.
    pub content: String,
    /// Optional file path for reference-based hydration.
    pub file_path: Option<String>,
    /// Time-to-live in seconds (0 = session-persistent).
    pub ttl_seconds: i32,
    /// Importance for context ranking (0.0 to 1.0).
    pub significance_score: f32,
    /// Searchable metadata tags.
    pub tags: Vec<String>,
    /// Creation timestamp.
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_entry_serialization() {
        let log = LogEntry {
            timestamp: Utc::now(),
            source: "koad-core".to_string(),
            message: "Testing log entry".to_string(),
            level: "INFO".to_string(),
        };
        let serialized = serde_json::to_string(&log).unwrap();
        let deserialized: LogEntry = serde_json::from_str(&serialized).unwrap();
        assert_eq!(log, deserialized);
    }
}
