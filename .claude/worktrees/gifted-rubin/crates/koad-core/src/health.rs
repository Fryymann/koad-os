use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HealthStatus {
    Pass,
    Warn,
    Fail,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthCheck {
    pub name: String,
    pub status: HealthStatus,
    pub message: String,
    pub last_checked: i64, // Unix timestamp
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct HealthRegistry {
    pub systems: Vec<HealthCheck>,
}

impl HealthRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, check: HealthCheck) {
        self.systems.push(check);
    }
}
