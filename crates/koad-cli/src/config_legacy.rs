use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadLegacyConfig {
    pub version: String,
    pub identity: KoadIdentity,
    pub drivers: Option<HashMap<String, KoadDriverConfig>>,
    pub project_registry: Option<HashMap<String, ProjectConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadIdentity {
    pub name: String,
    pub role: String,
    pub bio: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadDriverConfig {
    pub bootstrap: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub path: String,
    pub github_owner: Option<String>,
    pub github_repo: Option<String>,
    pub default_project: Option<u32>,
    pub credential_key: Option<String>,
}

impl KoadLegacyConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: KoadLegacyConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}
