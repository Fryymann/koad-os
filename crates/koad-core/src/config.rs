use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadConfig {
    pub home: PathBuf,
    pub redis_socket: PathBuf,
    pub spine_socket: PathBuf,
    pub spine_grpc_addr: String,
    pub gateway_addr: String,
    pub github_project_number: u32,
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl KoadConfig {
    pub fn load() -> Result<Self> {
        let home = env::var("KOAD_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let home = env::var("HOME").unwrap_or_default();
                PathBuf::from(format!("{}/.koad-os", home))
            });

        let redis_socket = env::var("REDIS_SOCKET")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(crate::constants::DEFAULT_REDIS_SOCK));

        let spine_socket = env::var("SPINE_SOCKET")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join(crate::constants::DEFAULT_SPINE_SOCK));

        let spine_grpc_addr = env::var("SPINE_GRPC_ADDR")
            .unwrap_or_else(|_| crate::constants::DEFAULT_SPINE_GRPC_ADDR.to_string());

        let gateway_addr = env::var("GATEWAY_ADDR")
            .unwrap_or_else(|_| crate::constants::DEFAULT_GATEWAY_ADDR.to_string());

        let github_project_number = env::var("GITHUB_PROJECT_NUMBER")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2);

        Ok(Self {
            home,
            redis_socket,
            spine_socket,
            spine_grpc_addr,
            gateway_addr,
            github_project_number,
            extra: HashMap::new(),
        })
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .map_err(|e| anyhow::anyhow!("Failed to serialize config: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize config: {}", e))
    }

    pub fn resolve_gh_token(&self) -> Result<String> {
        env::var("GITHUB_ADMIN_PAT")
            .or_else(|_| env::var("GITHUB_PERSONAL_PAT"))
            .context(
                "No GitHub PAT found in environment (tried GITHUB_ADMIN_PAT, GITHUB_PERSONAL_PAT)",
            )
    }

    pub fn get_github_owner(&self) -> Result<String> {
        Ok(env::var("GITHUB_OWNER")
            .unwrap_or_else(|_| crate::constants::DEFAULT_GITHUB_OWNER.to_string()))
    }

    pub fn get_github_repo(&self) -> Result<String> {
        Ok(env::var("GITHUB_REPO")
            .unwrap_or_else(|_| crate::constants::DEFAULT_GITHUB_REPO.to_string()))
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.home.join("koad.db")
    }

    pub fn get_log_path(&self, service: &str) -> PathBuf {
        self.home.join(format!("{}.log", service))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let mut config = KoadConfig::load().unwrap();
        config
            .extra
            .insert("test_key".to_string(), "test_value".to_string());

        let json = config.to_json().unwrap();
        let deserialized = KoadConfig::from_json(&json).unwrap();

        assert_eq!(config.home, deserialized.home);
        assert_eq!(
            config.github_project_number,
            deserialized.github_project_number
        );
        assert_eq!(
            config.extra.get("test_key"),
            Some(&"test_value".to_string())
        );
    }
}
