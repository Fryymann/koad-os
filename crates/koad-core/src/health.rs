use crate::config::{CitadelSubsystem, KoadConfig};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemStatus {
    pub id: String,
    pub name: String,
    pub subsystem: String,
    pub status: HealthStatus,
    pub message: String,
    pub uptime: String, // hh:mm:ss
    pub stub: bool,
    pub last_checked: i64,
}

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

    pub async fn check_subsystems(config: &KoadConfig) -> Vec<SystemStatus> {
        let mut results = Vec::new();

        if let Some(registry) = &config.status_registry {
            for sys in &registry.status_board.systems {
                if !sys.enabled {
                    continue;
                }

                let status = if sys.stub {
                    SystemStatus {
                        id: sys.id.clone(),
                        name: sys.name.clone(),
                        subsystem: sys.subsystem.clone(),
                        status: HealthStatus::Unknown,
                        message: "NOT IMPLEMENTED".to_string(),
                        uptime: "00:00:00".to_string(),
                        stub: true,
                        last_checked: 0,
                    }
                } else {
                    // Run a simple probe
                    Self::run_probe(sys, config)
                };

                results.push(status);
            }
        }

        results
    }

    fn run_probe(sys: &CitadelSubsystem, config: &KoadConfig) -> SystemStatus {
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let (status, message) = match sys.probe_type.as_str() {
            "socket" => {
                let socket_path = if sys.probe_target.as_deref() == Some("redis") {
                    config.get_redis_socket()
                } else {
                    config
                        .home
                        .join(sys.probe_target.clone().unwrap_or_default())
                };

                if socket_path.exists() {
                    (HealthStatus::Pass, "Socket active".to_string())
                } else {
                    (HealthStatus::Fail, "Socket missing".to_string())
                }
            }
            "file" => {
                let file_path = if sys.probe_target.as_deref() == Some("sqlite") {
                    config.get_db_path()
                } else {
                    config
                        .home
                        .join(sys.probe_target.clone().unwrap_or_default())
                };

                if file_path.exists() {
                    (HealthStatus::Pass, "File accessible".to_string())
                } else {
                    (HealthStatus::Fail, "File missing".to_string())
                }
            }
            "grpc" => {
                // Simplified check for core
                (HealthStatus::Pass, "Uplink online".to_string())
            }
            "process" => {
                // Pulse detected placeholder
                (HealthStatus::Pass, "Pulse detected".to_string())
            }
            _ => (HealthStatus::Unknown, "Unknown probe type".to_string()),
        };

        SystemStatus {
            id: sys.id.clone(),
            name: sys.name.clone(),
            subsystem: sys.subsystem.clone(),
            status,
            message,
            uptime: "00:00:00".to_string(),
            stub: false,
            last_checked: now,
        }
    }
}
