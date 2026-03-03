use std::path::{Path, PathBuf};
use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone)]
pub struct KoadConfig {
    pub home: PathBuf,
    pub redis_socket: PathBuf,
    pub spine_grpc_addr: String,
    pub gateway_addr: String,
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
            .unwrap_or_else(|_| home.join("koad.sock"));

        let spine_grpc_addr = env::var("SPINE_GRPC_ADDR")
            .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string());

        let gateway_addr = env::var("GATEWAY_ADDR")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string());

        Ok(Self {
            home,
            redis_socket,
            spine_grpc_addr,
            gateway_addr,
        })
    }

    pub fn resolve_gh_token(&self) -> Result<String> {
        env::var("GITHUB_ADMIN_PAT")
            .or_else(|_| env::var("GITHUB_PERSONAL_PAT"))
            .context("No GitHub PAT found in environment (tried GITHUB_ADMIN_PAT, GITHUB_PERSONAL_PAT)")
    }

    pub fn get_db_path(&self) -> PathBuf {
        self.home.join("koad.db")
    }

    pub fn get_log_path(&self, service: &str) -> PathBuf {
        self.home.join(format!("{}.log", service))
    }
}
