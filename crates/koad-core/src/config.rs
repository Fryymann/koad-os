use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct KoadConfig {
    pub home: PathBuf,
    pub redis_socket: PathBuf,
    pub spine_socket: PathBuf,
    pub spine_grpc_addr: String,
    pub gateway_addr: String,
    pub github_project_number: u32,
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
        })
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
