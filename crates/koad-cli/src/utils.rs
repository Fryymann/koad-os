use std::path::Path;
use std::env;
use koad_core::config::KoadConfig;
use sysinfo::System;

pub enum PreFlightStatus {
    Optimal,
    Degraded(String),
    Critical(String),
}

pub fn pre_flight(config: &KoadConfig) -> PreFlightStatus {
    if !config.redis_socket.exists() {
        return PreFlightStatus::Critical("Neural Bus (Redis) offline.".to_string());
    }
    PreFlightStatus::Optimal
}

pub fn find_ghosts(home: &Path) -> Vec<(u32, String)> {
    let mut sys = System::new_all();
    sys.refresh_all();
    let mut ghosts = Vec::new();
    let pid_file = home.join("redis.pid");
    if pid_file.exists() {
        if let Ok(pid_str) = std::fs::read_to_string(pid_file) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if !sys.processes().contains_key(&sysinfo::Pid::from(pid as usize)) {
                    ghosts.push((pid, "Stale Redis PID".to_string()));
                }
            }
        }
    }
    ghosts
}

pub fn detect_context_tags(path: &Path) -> Vec<String> {
    let mut tags = Vec::new();
    if path.join("Cargo.toml").exists() { tags.push("rust".to_string()); }
    if path.join("package.json").exists() { tags.push("node".to_string()); }
    if path.join("requirements.txt").exists() || path.join("pyproject.toml").exists() { tags.push("python".to_string()); }
    tags
}

pub fn get_gh_pat_for_path(_path: &Path, role: &str, _config: &KoadConfig) -> (String, String) {
    let pat = match role {
        "admin" => env::var("GITHUB_ADMIN_PAT").unwrap_or_else(|_| "GITHUB_PERSONAL_PAT".to_string()),
        _ => "GITHUB_PERSONAL_PAT".to_string(),
    };
    (pat.clone(), pat)
}

pub fn get_gdrive_token_for_path(_path: &Path) -> (String, String) {
    let token = "GDRIVE_PERSONAL_TOKEN".to_string();
    (token.clone(), token)
}

pub fn detect_model_tier() -> i32 {
    if env::var("GEMINI_CLI").is_ok() { 1 }
    else if env::var("CODEX_CLI").is_ok() { 2 }
    else { 3 }
}

pub fn feature_gate(feature: &str, min_tier: Option<i32>) {
    println!("\x1b[33m[LOCKED]\x1b[0m Feature '{}' requires Tier {} cognitive clearance.", feature, min_tier.unwrap_or(1));
}
