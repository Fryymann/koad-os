use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadLegacyConfig {
    pub version: String,
    pub identity: KoadIdentity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KoadIdentity {
    pub name: String,
    pub role: String,
    pub bio: String,
}

impl KoadLegacyConfig {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: KoadLegacyConfig = serde_json::from_str(&content)?;
        Ok(config)
    }
}
