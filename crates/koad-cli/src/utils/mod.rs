pub mod errors;

use anyhow::Result;
use koad_core::config::KoadConfig;
use koad_proto::citadel::v5::citadel_session_client::CitadelSessionClient;
use std::env;
use std::path::Path;
use tonic::transport::Channel;

pub enum PreFlightStatus {
    Optimal,
    Degraded(String),
    Critical(String),
}

pub async fn get_citadel_client(config: &KoadConfig) -> Result<CitadelSessionClient<Channel>> {
    let addr = config.network.citadel_grpc_addr.clone();
    CitadelSessionClient::connect(addr.clone())
        .await
        .map_err(|e| errors::map_connect_err("KoadOS Citadel", &addr, e))
        .map_err(anyhow::Error::from)
}

use koad_proto::citadel::v5::TraceContext;

pub fn get_trace_context(actor: &str, level: i32) -> TraceContext {
    TraceContext {
        trace_id: format!("TRC-{}-{}", actor, &uuid::Uuid::new_v4().to_string()[..8]),
        origin: "CLI".to_string(),
        actor: actor.to_string(),
        timestamp: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        level,
    }
}

pub fn authenticated_request<T>(payload: T) -> tonic::Request<T> {
    let mut req = tonic::Request::new(payload);
    if let Ok(sid) = env::var("KOAD_SESSION_ID") {
        if let Ok(val) = sid.parse::<tonic::metadata::MetadataValue<tonic::metadata::Ascii>>() {
            req.metadata_mut().insert("x-session-id", val);
        }
    }
    req
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
        "admin" => env::var("KOADOS_PAT_GITHUB_ADMIN").unwrap_or_default(),
        _ => env::var("KOADOS_MAIN_GITHUB_PAT").unwrap_or_default(),
    };
    (pat.clone(), pat)
}

pub fn get_gdrive_token_for_path(_path: &Path) -> (String, String) {
    let token = "GDRIVE_PERSONAL_TOKEN".to_string();
    (token.clone(), token)
}

pub fn detect_model_tier() -> i32 {
    if let Ok(override_tier) = env::var("KOAD_TIER_OVERRIDE") {
        if let Ok(tier) = override_tier.parse::<i32>() {
            return tier;
        }
    }

    if env::var("GEMINI_CLI").is_ok()
        || env::var("CLAUDE_CODE").is_ok()
        || env::var("CODEX_CLI").is_ok()
        || env::var("GEMINI_MODEL").is_ok()
        || env::var("CLAUDE_MODEL").is_ok()
        || env::var("CODEX_MODEL").is_ok()
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
