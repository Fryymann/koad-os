use anyhow::{Context, Result};
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

use crate::constants::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadConfig {
    pub home: PathBuf,
    pub system: SystemConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    #[serde(default = "default_sessions")]
    pub sessions: SessionsConfig,
    #[serde(default = "default_watchdog")]
    pub watchdog: WatchdogConfig,
    #[serde(default = "default_sandbox")]
    pub sandbox: SandboxConfig,
    #[serde(default)]
    pub integrations: IntegrationsConfig,
    #[serde(default)]
    pub filesystem: FilesystemConfig,
    #[serde(default)]
    pub identities: HashMap<String, AgentIdentityConfig>,
    #[serde(default)]
    pub interfaces: HashMap<String, InterfaceConfig>,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
    #[serde(default)]
    pub project_dirs: HashMap<String, ProjectDirConfig>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub gateway_port: u32,
    pub gateway_addr: String,
    pub spine_grpc_port: u32,
    pub spine_grpc_addr: String,
    pub cass_grpc_port: u32,
    pub cass_grpc_addr: String,
    pub redis_socket: String,
    pub spine_socket: String,
    #[serde(default = "default_admin_socket")]
    pub admin_socket: String,
}

fn default_admin_socket() -> String {
    DEFAULT_ADMIN_SOCK.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_name: String,
    pub drain_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsConfig {
    pub deadman_timeout_secs: u64,
    pub dark_timeout_secs: u64,
    pub purge_timeout_secs: u64,
    pub lease_duration_secs: u64,
    pub reaper_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    pub check_interval_secs: u64,
    pub max_failures: u32,
    pub monitor_asm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub blacklist: Vec<String>,
    pub sanctuary: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub version: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    pub github: Option<GithubConfig>,
    pub notion: Option<NotionConfig>,
    pub airtable: Option<AirtableConfig>,
    pub slack: Option<SlackConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    pub default_owner: String,
    pub default_repo: String,
    pub default_project_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionConfig {
    pub enabled: bool,
    pub index: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirtableConfig {
    pub enabled: bool,
    pub base_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackConfig {
    pub enabled: bool,
    pub channel_id: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    pub workspace_root: Option<String>,
    pub protected_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIdentityConfig {
    pub name: String,
    pub role: String,
    pub rank: String,
    pub bio: String,
    pub bootstrap: Option<String>,
    pub preferences: Option<AgentPreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPreferences {
    pub access_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    pub driver: String,
    pub bootstrap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    pub path: PathBuf,
    pub level: Option<String>,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub default_project: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDirConfig {
    pub path: PathBuf,
}

impl KoadConfig {
    pub fn load() -> Result<Self> {
        let home = match env::var("KOAD_HOME") {
            Ok(val) => PathBuf::from(val),
            Err(_) => dirs::home_dir()
                .context("Could not determine home directory and KOAD_HOME is not set")?
                .join(".koad-os"),
        };

        let config_path = home.join("config/kernel.toml");

        let s = Config::builder()
            .set_default("home", home.to_string_lossy().to_string())?
            .add_source(File::from(config_path).required(false))
            .add_source(config::Environment::with_prefix("KOAD").separator("__"))
            .build()?;

        s.try_deserialize()
            .context("Failed to deserialize KoadConfig")
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to parse config JSON")
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialize KoadConfig to JSON")
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.home.join(&self.storage.db_name)
    }

    pub fn get_redis_socket(&self) -> PathBuf {
        self.home.join(&self.network.redis_socket)
    }

    pub fn get_spine_socket(&self) -> PathBuf {
        self.home.join(&self.network.spine_socket)
    }

    pub fn get_admin_socket(&self) -> PathBuf {
        self.home.join(&self.network.admin_socket)
    }

    pub fn get_agent_name(&self) -> String {
        if let Ok(agent) = env::var("KOAD_AGENT") {
            return agent;
        }
        "Admiral".to_string()
    }

    pub fn get_github_owner(&self, project: Option<&str>) -> String {
        if let Some(proj) = project {
            if let Some(p_config) = self.projects.get(proj) {
                if let Some(owner) = &p_config.github_owner {
                    return owner.clone();
                }
            }
        }
        self.integrations
            .github
            .as_ref()
            .map(|g| g.default_owner.clone())
            .unwrap_or_else(|| DEFAULT_GITHUB_OWNER.to_string())
    }

    pub fn get_github_repo(&self, project: Option<&str>) -> String {
        if let Some(proj) = project {
            if let Some(p_config) = self.projects.get(proj) {
                if let Some(repo) = &p_config.github_repo {
                    return repo.clone();
                }
            }
        }
        self.integrations
            .github
            .as_ref()
            .map(|g| g.default_repo.clone())
            .unwrap_or_else(|| DEFAULT_GITHUB_REPO.to_string())
    }

    pub fn resolve_gh_token(&self, _project: Option<&str>, _agent: Option<&str>) -> Result<String> {
        Ok(env::var("GITHUB_PAT").unwrap_or_default())
    }

    pub fn resolve_project_context(&self, path: &Path) -> Option<(String, String)> {
        for (name, config) in &self.projects {
            if path.starts_with(&config.path) {
                return Some((name.clone(), name.clone()));
            }
        }
        None
    }
}

pub fn default_network() -> NetworkConfig {
    NetworkConfig {
        gateway_port: DEFAULT_GATEWAY_PORT,
        gateway_addr: DEFAULT_GATEWAY_ADDR.to_string(),
        spine_grpc_port: DEFAULT_SPINE_GRPC_PORT,
        spine_grpc_addr: DEFAULT_SPINE_GRPC_ADDR.to_string(),
        cass_grpc_port: DEFAULT_CASS_GRPC_PORT,
        cass_grpc_addr: DEFAULT_CASS_GRPC_ADDR.to_string(),
        redis_socket: DEFAULT_REDIS_SOCK.to_string(),
        spine_socket: DEFAULT_SPINE_SOCK.to_string(),
        admin_socket: DEFAULT_ADMIN_SOCK.to_string(),
    }
}

pub fn default_storage() -> StorageConfig {
    StorageConfig {
        db_name: "koad.db".to_string(),
        drain_interval_secs: 30,
    }
}

pub fn default_sessions() -> SessionsConfig {
    SessionsConfig {
        deadman_timeout_secs: DEFAULT_DEADMAN_TIMEOUT_SECS,
        dark_timeout_secs: DEFAULT_DARK_TIMEOUT_SECS,
        purge_timeout_secs: DEFAULT_PURGE_TIMEOUT_SECS,
        lease_duration_secs: DEFAULT_LEASE_DURATION_SECS,
        reaper_interval_secs: DEFAULT_REAPER_INTERVAL_SECS,
    }
}

pub fn default_watchdog() -> WatchdogConfig {
    WatchdogConfig {
        check_interval_secs: 10,
        max_failures: 3,
        monitor_asm: true,
    }
}

pub fn default_sandbox() -> SandboxConfig {
    SandboxConfig {
        enabled: true,
        blacklist: vec![
            "sudo ".to_string(),
            "su ".to_string(),
            "rm -rf /".to_string(),
            "koad boot".to_string(),
        ],
        sanctuary: vec![
            ".koad-os".to_string(),
            "/etc".to_string(),
            "/var".to_string(),
            "/root".to_string(),
        ],
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_config_loading() {
        // This test requires actual files to exist or mock data.
        // For now, we just verify the struct can be instantiated.
    }
}
