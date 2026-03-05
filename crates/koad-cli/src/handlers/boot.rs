use anyhow::{Context, Result};
use koad_core::config::KoadConfig;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use std::env;
use crate::utils::{detect_context_tags, detect_model_tier, get_gh_pat_for_path};
use crate::config_legacy::KoadLegacyConfig;

pub async fn handle_boot_command(
    agent: String,
    project: bool,
    _task: Option<String>,
    compact: bool,
    role: String,
    config: &KoadConfig,
    legacy_config: &KoadLegacyConfig,
) -> Result<()> {
    let model_tier = detect_model_tier();
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let tags = detect_context_tags(&current_dir);
    let (gh_pat, _) = get_gh_pat_for_path(&current_dir, &role, config);
    let gdrive_token = "GDRIVE_PERSONAL_TOKEN";

    let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone())
        .await
        .context("Failed to connect to Spine Backbone.")?;

    let resp = client.initialize_session(InitializeSessionRequest {
        agent_name: agent.clone(),
        agent_role: role.clone(),
        project_name: if project { "active" } else { "default" }.to_string(),
        environment: EnvironmentType::Wsl as i32,
        driver_id: if env::var("GEMINI_CLI").is_ok() { "gemini".to_string() } else if env::var("CODEX_CLI").is_ok() { "codex".to_string() } else { "cli".to_string() },
        model_tier,
        model_name: env::var("GEMINI_MODEL").or_else(|_| env::var("CODEX_MODEL")).unwrap_or_else(|_| "unknown".to_string()),
    }).await.map_err(|e| anyhow::anyhow!("Lease Denied: {}", e.message()))?;
    
    let package = resp.into_inner();
    let session_id = package.session_id;
    let mission_briefing = package.intelligence.map(|i| i.mission_briefing);

    if compact {
        println!("I:{}|R:{}|G:{}|D:{}|T:{}|S:{}|Q:1", agent, role, gh_pat, gdrive_token, tags.join(","), session_id);
    } else {
        println!("
\x1b[1m--- KoadOS Neural Link Established ---\x1b[0m");
        println!("Agent:    {}", agent);
        println!("Role:     {}", role);
        println!("Session:  {}", session_id);
        println!("Tags:     {}", tags.join(", "));
        
        if let Some(ref drivers) = legacy_config.drivers {
            if let Some(driver) = drivers.get(&agent) {
                let b_path = driver.bootstrap.replace("~", &env::var("HOME").unwrap_or_default());
                if let Ok(content) = std::fs::read_to_string(b_path) {
                    println!("
\x1b[1m[BOOTSTRAP: {}]\x1b[0m
{}", agent, content);
                }
            }
        }

        if let Some(briefing) = mission_briefing {
            println!("
\x1b[1mMission Briefing:\x1b[0m
{}", briefing);
        }
        println!("\x1b[1m---------------------------------------\x1b[0m
");
    }
    Ok(())
}
