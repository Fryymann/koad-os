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
    #[serde(default)]
    pub system: Option<SystemConfig>,
    #[serde(default = "default_network")]
    pub network: NetworkConfig,
    #[serde(default = "default_storage")]
    pub storage: StorageConfig,
    #[serde(default = "default_sessions")]
    pub sessions: SessionsConfig,
    #[serde(default = "default_watchdog")]
    pub watchdog: WatchdogConfig,
    #[serde(default)]
    pub integrations: IntegrationsConfig,
    #[serde(default)]
    pub filesystem: FilesystemConfig,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
    #[serde(default)]
    pub identities: HashMap<String, IdentityConfig>,
    #[serde(default)]
    pub interfaces: HashMap<String, InterfaceConfig>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

fn default_network() -> NetworkConfig {
    NetworkConfig {
        gateway_port: DEFAULT_GATEWAY_PORT,
        gateway_addr: DEFAULT_GATEWAY_ADDR.to_string(),
        spine_grpc_port: DEFAULT_SPINE_GRPC_PORT,
        spine_grpc_addr: DEFAULT_SPINE_GRPC_ADDR.to_string(),
        redis_socket: DEFAULT_REDIS_SOCK.to_string(),
        spine_socket: DEFAULT_SPINE_SOCK.to_string(),
    }
}

fn default_storage() -> StorageConfig {
    StorageConfig {
        db_name: "koad.db".to_string(),
        drain_interval_secs: 30,
    }
}

fn default_sessions() -> SessionsConfig {
    SessionsConfig {
        deadman_timeout_secs: DEFAULT_DEADMAN_TIMEOUT_SECS,
        dark_timeout_secs: DEFAULT_DARK_TIMEOUT_SECS,
        purge_timeout_secs: DEFAULT_PURGE_TIMEOUT_SECS,
        lease_duration_secs: DEFAULT_LEASE_DURATION_SECS,
        reaper_interval_secs: DEFAULT_REAPER_INTERVAL_SECS,
    }
}

fn default_watchdog() -> WatchdogConfig {
    WatchdogConfig {
        check_interval_secs: 10,
        max_failures: 3,
        monitor_asm: true,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    pub version: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct IntegrationsConfig {
    #[serde(default)]
    pub github: Option<GithubConfig>,
    #[serde(default)]
    pub notion: Option<NotionConfig>,
    #[serde(default)]
    pub airtable: Option<AirtableConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotionConfig {
    pub mcp: bool,
    pub index: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirtableConfig {
    pub index: HashMap<String, AirtableTableConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirtableTableConfig {
    pub base_id: String,
    pub table: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FilesystemConfig {
    #[serde(default)]
    pub mappings: HashMap<String, String>,
    #[serde(default)]
    pub workspace_symlink: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub gateway_port: u32,
    pub gateway_addr: String,
    pub spine_grpc_port: u32,
    pub spine_grpc_addr: String,
    pub redis_socket: String,
    pub spine_socket: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub db_name: String,
    pub drain_interval_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    #[serde(default = "default_github_owner")]
    pub default_owner: String,
    #[serde(default = "default_github_repo")]
    pub default_repo: String,
    #[serde(default = "default_github_project_number")]
    pub default_project_number: u32,
    #[serde(default = "default_github_api_base")]
    pub api_base: String,
}

fn default_github_owner() -> String {
    crate::constants::DEFAULT_GITHUB_OWNER.to_string()
}

fn default_github_repo() -> String {
    crate::constants::DEFAULT_GITHUB_REPO.to_string()
}

fn default_github_project_number() -> u32 {
    crate::constants::DEFAULT_GITHUB_PROJECT_NUMBER
}

fn default_github_api_base() -> String {
    crate::constants::GITHUB_API_BASE.to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsConfig {
    pub deadman_timeout_secs: u64,
    pub dark_timeout_secs: u64,
    pub purge_timeout_secs: u64,
    pub lease_duration_secs: u64,
    #[serde(default = "default_reaper_interval")]
    pub reaper_interval_secs: u64,
}

fn default_reaper_interval() -> u64 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    pub check_interval_secs: u64,
    pub max_failures: u32,
    pub monitor_asm: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub path: String,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub default_project: Option<u32>,
    pub credential_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    pub name: String,
    pub role: String,
    pub rank: String,
    pub bio: String,
    pub bootstrap: Option<String>,
    #[serde(default)]
    pub preferences: Option<PreferenceConfig>,
    #[serde(default)]
    pub session_policy: Option<SessionPolicy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceConfig {
    pub name: String,
    pub bootstrap: String,
    #[serde(default)]
    pub mcp_enabled: bool,
    #[serde(default)]
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionPolicy {
    pub deadman_timeout_secs: Option<u64>,
    pub dark_timeout_secs: Option<u64>,
    pub purge_timeout_secs: Option<u64>,
    pub lease_duration_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceConfig {
    pub languages: Vec<String>,
    pub style: String,
    #[serde(default)]
    pub access_keys: Vec<String>,
    pub principles: Vec<String>,
}

impl KoadConfig {
    pub fn load() -> Result<Self> {
        let home = env::var("KOAD_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_default();
                PathBuf::from(format!("{}/.koad-os", home))
            });

        let config_dir = home.join("config");

        let mut builder = Config::builder()
            .set_default("home", home.to_string_lossy().to_string())?
            // Load base kernel config
            .add_source(File::from(config_dir.join("kernel.toml")).required(false))
            // Load filesystem mappings
            .add_source(File::from(config_dir.join("filesystem.toml")).required(false))
            // Load registry
            .add_source(File::from(config_dir.join("registry.toml")).required(false));

        // Load all integrations
        let integrations_dir = config_dir.join("integrations");
        if integrations_dir.exists() {
            for entry in std::fs::read_dir(integrations_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    builder = builder.add_source(File::from(path).required(false));
                }
            }
        }

        // Load all identities
        let identity_dir = config_dir.join("identities");
        if identity_dir.exists() {
            for entry in std::fs::read_dir(identity_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    builder = builder.add_source(File::from(path).required(false));
                }
            }
        }

        // Load all interfaces
        let interface_dir = config_dir.join("interfaces");
        if interface_dir.exists() {
            for entry in std::fs::read_dir(interface_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("toml") {
                    builder = builder.add_source(File::from(path).required(false));
                }
            }
        }

        // Environment overrides
        builder = builder.add_source(config::Environment::with_prefix("KOAD").separator("__"));

        let settings = builder.build()?;
        let config: KoadConfig = settings.try_deserialize()?;

        Ok(config)
    }

    /// Resolve the active project name and config based on the current directory.
    pub fn resolve_project_context(&self, current_dir: &Path) -> Option<(String, ProjectConfig)> {
        for (name, project) in &self.projects {
            let project_path = self.resolve_path(&project.path);
            if current_dir.starts_with(&project_path) {
                return Some((name.clone(), project.clone()));
            }
        }
        None
    }

    fn resolve_path(&self, path: &str) -> PathBuf {
        if path.starts_with('~') {
            let home = env::var("HOME").unwrap_or_default();
            PathBuf::from(path.replace('~', &home))
        } else {
            PathBuf::from(path)
        }
    }

    /// Validates that a path is within authorized KoadOS boundaries.
    pub fn validate_path(
        &self,
        path: &str,
        session: Option<&crate::session::AgentSession>,
    ) -> Result<PathBuf> {
        // 0. System Bypass
        if let Some(s) = session {
            if s.context.allowed_paths.contains(&"all".to_string()) {
                let expanded_p = self.resolve_path(path);
                return std::fs::canonicalize(&expanded_p)
                    .context(format!("System Path Resolve Error: {}", path));
            }
        }

        let expanded_p = self.resolve_path(path);

        let canonical_p = std::fs::canonicalize(&expanded_p).context(format!(
            "Sanctuary Rule: Path does not exist or cannot be accessed: {}",
            path
        ))?;

        let mut allowed = false;

        // 1. Check KOAD_HOME
        if let Ok(canon_home) = std::fs::canonicalize(&self.home) {
            if canonical_p.starts_with(&canon_home) {
                allowed = true;
            }
        }

        // 2. Check registered projects
        if !allowed {
            for project in self.projects.values() {
                let project_path = self.resolve_path(&project.path);
                if let Ok(canon_project) = std::fs::canonicalize(project_path) {
                    if canonical_p.starts_with(&canon_project) {
                        allowed = true;
                        break;
                    }
                }
            }
        }

        if !allowed {
            anyhow::bail!(
                "Sanctuary Violation: Path '{:?}' is outside authorized KoadOS boundaries.",
                canonical_p
            );
        }

        Ok(canonical_p)
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize config: {}", e))
    }

    pub fn resolve_gh_token(
        &self,
        project: Option<&ProjectConfig>,
        identity: Option<&crate::identity::Identity>,
    ) -> Result<String> {
        if let Some(p) = project {
            if let Some(key) = &p.credential_key {
                if let Ok(token) = env::var(key) {
                    return Ok(token);
                }
            }
        }

        if let Some(id) = identity {
            for key in &id.access_keys {
                if let Ok(token) = env::var(key) {
                    return Ok(token);
                }
            }
        }

        env::var("GITHUB_PAT")
            .or_else(|_| env::var("GITHUB_ADMIN_PAT"))
            .or_else(|_| env::var("GITHUB_PERSONAL_PAT"))
            .context(
                "No GitHub PAT found in environment (tried project key, identity access_keys, GITHUB_PAT, GITHUB_ADMIN_PAT, GITHUB_PERSONAL_PAT)",
            )
    }

    pub fn get_github_owner(&self, project: Option<&ProjectConfig>) -> String {
        project
            .and_then(|p| p.github_owner.clone())
            .unwrap_or_else(|| {
                self.integrations
                    .github
                    .as_ref()
                    .map(|g| g.default_owner.clone())
                    .unwrap_or_else(|| "Fryymann".to_string())
            })
    }

    pub fn get_github_repo(&self, project: Option<&ProjectConfig>) -> String {
        project
            .and_then(|p| p.github_repo.clone())
            .unwrap_or_else(|| {
                self.integrations
                    .github
                    .as_ref()
                    .map(|g| g.default_repo.clone())
                    .unwrap_or_else(|| "koad-os".to_string())
            })
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.home.join(&self.storage.db_name)
    }

    pub fn get_log_path(&self, service: &str) -> PathBuf {
        self.home.join(format!("{}.log", service))
    }

    pub fn get_redis_socket(&self) -> PathBuf {
        self.home.join(&self.network.redis_socket)
    }

    pub fn get_spine_socket(&self) -> PathBuf {
        self.home.join(&self.network.spine_socket)
    }

    pub fn get_agent_name(&self) -> String {
        if let Ok(agent) = env::var("KOAD_AGENT") {
            return agent;
        }

        if self.identities.contains_key("Tyr") {
            return "Tyr".to_string();
        }

        self.identities
            .keys()
            .next()
            .cloned()
            .unwrap_or_else(|| "Koad".to_string())
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
