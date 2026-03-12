use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use std::env;
use std::path::Path;
use tonic::transport::Channel;

pub enum PreFlightStatus {
    Optimal,
    Degraded(String),
    Critical(String),
}

pub async fn get_spine_client(config: &KoadConfig) -> Result<SpineServiceClient<Channel>> {
    SpineServiceClient::connect(config.network.spine_grpc_addr.clone())
        .await
        .context("Failed to connect to Koad Spine gRPC")
}

pub fn pre_flight(config: &KoadConfig) -> PreFlightStatus {
    if !config.get_redis_socket().exists() {
        return PreFlightStatus::Critical("Neural Bus (Redis) offline.".to_string());
    }
    PreFlightStatus::Optimal
}

pub fn detect_context_tags(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();
    if path.join("Cargo.toml").exists() {
        tags.push("rust".to_string());
    }
    if path.join("package.json").exists() {
        tags.push("node".to_string());
    }
    if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() {
        tags.push("python".to_string());
    }
    tags
}

pub fn get_gh_pat_for_path(_path: &Path, role: &str, _config: &KoadConfig) -> (String, String) {
    let pat = match role {
        "admin" => {
            env::var("GITHUB_ADMIN_PAT").unwrap_or_else(|_| "GITHUB_PERSONAL_PAT".to_string())
        }
        _ => "GITHUB_PERSONAL_PAT".to_string(),
    };
    (pat.clone(), pat)
}

pub fn get_gdrive_token_for_path(_path: &Path) -> (String, String) {
    let token = "GDRIVE_PERSONAL_TOKEN".to_string();
    (token.clone(), token)
}

pub fn detect_model_tier() -> i32 {
    if env::var("GEMINI_CLI").is_ok()
        || env::var("CLAUDE_CODE").is_ok()
        || env::var("CODEX_CLI").is_ok()
    {
        1
    } else {
        3
    }
}

pub fn feature_gate(feature: &str, min_tier: Option<i32>) {
    println!(
        "\x1b[33m[LOCKED]\x1b[0m Feature '{}' requires Tier {} cognitive clearance.",
        feature,
        min_tier.unwrap_or(1)
    );
}
