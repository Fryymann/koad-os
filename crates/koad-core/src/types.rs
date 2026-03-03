use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents a unit of work within the KoadOS ecosystem.
/// Every Ticket MUST have a corresponding GitHub Issue for external tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ticket {
    pub id: uuid::Uuid,
    pub github_issue: Option<u32>,
    pub title: String,
    pub problem: String,
    pub solution: String,
    pub implementation_plan: Vec<String>,
    pub status: TicketStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TicketStatus {
    Draft,
    Open,
    InProgress,
    Testing,
    Resolved,
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
