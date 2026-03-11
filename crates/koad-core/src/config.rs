use anyhow::{Context, Result};
use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadConfig {
    pub home: PathBuf,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub github: GithubConfig,
    pub sessions: SessionsConfig,
    pub watchdog: WatchdogConfig,
    #[serde(default)]
    pub projects: HashMap<String, ProjectConfig>,
    #[serde(default)]
    pub identities: HashMap<String, IdentityConfig>,
    #[serde(default)]
    pub extra: HashMap<String, String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GithubConfig {
    pub default_owner: String,
    pub default_repo: String,
    pub default_project_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionsConfig {
    pub deadman_timeout_secs: u64,
    pub dark_timeout_secs: u64,
    pub purge_timeout_secs: u64,
    pub lease_duration_secs: u64,
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
    #[serde(default)]
    pub preferences: Option<PreferenceConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceConfig {
    pub languages: Vec<String>,
    pub style: String,
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
            // Load integrations
            .add_source(File::from(config_dir.join("integrations/github.toml")).required(false))
            // Load registry
            .add_source(File::from(config_dir.join("registry.toml")).required(false));

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

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize config: {}", e))
    }

    pub fn resolve_gh_token(&self, project: Option<&ProjectConfig>) -> Result<String> {
        if let Some(p) = project {
            if let Some(key) = &p.credential_key {
                if let Ok(token) = env::var(key) {
                    return Ok(token);
                }
            }
        }

        env::var("GITHUB_PAT")
            .or_else(|_| env::var("GITHUB_ADMIN_PAT"))
            .or_else(|_| env::var("GITHUB_PERSONAL_PAT"))
            .context(
                "No GitHub PAT found in environment (tried project key, GITHUB_PAT, GITHUB_ADMIN_PAT, GITHUB_PERSONAL_PAT)",
            )
    }

    pub fn get_github_owner(&self, project: Option<&ProjectConfig>) -> String {
        project
            .and_then(|p| p.github_owner.clone())
            .unwrap_or_else(|| self.github.default_owner.clone())
    }

    pub fn get_github_repo(&self, project: Option<&ProjectConfig>) -> String {
        project
            .and_then(|p| p.github_repo.clone())
            .unwrap_or_else(|| self.github.default_repo.clone())
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        // This test requires actual files to exist or mock data.
        // For now, we just verify the struct can be instantiated.
    }
}
