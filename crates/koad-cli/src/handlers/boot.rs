use crate::config_legacy::KoadLegacyConfig;
use crate::utils::{detect_context_tags, detect_model_tier, get_gh_pat_for_path};
use anyhow::{Context, Result};
use fred::interfaces::{HashesInterface, KeysInterface, PubsubInterface};
use koad_core::config::KoadConfig;
use koad_core::utils::redis::RedisClient;
use koad_proto::spine::v1::spine_service_client::SpineServiceClient;
use koad_proto::spine::v1::*;
use std::env;

pub async fn handle_boot_command(
    agent: String,
    project: bool,
    _task: Option<String>,
    compact: bool,
    role: String,
    config: &KoadConfig,
    legacy_config: &KoadLegacyConfig,
) -> Result<()> {
    // --- [Laws of Consciousness: Occupancy Check] ---
    if let Ok(existing_sid) = env::var("KOAD_SESSION_ID") {
        if !existing_sid.is_empty() {
            anyhow::bail!(
                "\x1b[31mCONSCIOUSNESS_COLLISION\x1b[0m: Session {} is already active in this Body. Aborting boot.",
                existing_sid
            );
        }
    }

    let model_tier = detect_model_tier();
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let tags = detect_context_tags(&current_dir);
    let (gh_pat, _) = get_gh_pat_for_path(&current_dir, &role, config);
    let gdrive_token = "GDRIVE_PERSONAL_TOKEN";

    let mut client = SpineServiceClient::connect(config.spine_grpc_addr.clone())
        .await
        .context("Failed to connect to Spine Backbone.")?;

    // --- [Sovereign Pruning] ---
    // Captains and Admirals don't auto-prune, so we clean up our own ghosts on boot.
    // We use an optimistic, direct Redis sweep to ensure no "Dark" sessions or leases block the link.
    if agent == "Tyr" || agent == "Koad" || agent == "Dood" {
        if !compact {
            println!(
                "\x1b[33m[Sovereign Override] Clearing existing links for {}...\x1b[0m",
                agent
            );
        }

        let client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
        let pool = &client.pool;

        // 1. Clear Lease
        let lease_key = format!("koad:kai:{}:lease", agent);
        let _: () = pool.del(lease_key).await?;

        // 2. Clear Session mapping (if any)
        let identity_key = format!("koad:identity:{}", agent);
        let _: () = pool.del(identity_key).await?;

        // 3. Scan and Prune from koad:state
        let all_state: std::collections::HashMap<String, String> =
            pool.hgetall("koad:state").await?;
        for (key, val) in all_state {
            if key.starts_with("koad:session:") && val.contains(&format!("\"name\":\"{}\"", agent))
            {
                let _: () = pool.hdel("koad:state", &key).await?;
                let sid = key.replace("koad:session:", "");
                let msg = serde_json::json!({ "type": "SESSION_PRUNED", "session_id": sid });
                let _: () = pool.next()
                    .publish("koad:sessions", msg.to_string())
                    .await?;
            }
        }
    }

    let resp = client
        .initialize_session(InitializeSessionRequest {
            agent_name: agent.clone(),
            agent_role: role.clone(),
            project_name: if project { "active" } else { "default" }.to_string(),
            environment: EnvironmentType::Wsl as i32,
            driver_id: if env::var("GEMINI_CLI").is_ok() {
                "gemini".to_string()
            } else if env::var("CODEX_CLI").is_ok() {
                "codex".to_string()
            } else {
                "cli".to_string()
            },
            model_tier,
            model_name: env::var("GEMINI_MODEL")
                .or_else(|_| env::var("CODEX_MODEL"))
                .unwrap_or_else(|_| "unknown".to_string()),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Lease Denied: {}", e.message()))?;

    let package = resp.into_inner();
    let session_id = package.session_id;
    let mission_briefing = package.intelligence.map(|i| i.mission_briefing);

    // --- [Autonomic Nervous System: Heartbeat Daemon] ---
    if let Ok(bin_path) = env::current_exe() {
        let _ = std::process::Command::new(bin_path)
            .arg("system")
            .arg("heartbeat")
            .arg("--daemon")
            .arg("--session")
            .arg(&session_id)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
    }

    // Extract true rank from identity_json
    let identity: koad_core::identity::Identity = serde_json::from_str(&package.identity_json)
        .context("Failed to deserialize identity from Spine")?;
    let rank_display = format!("{:?}", identity.rank);

    if compact {
        println!(
            "I:{}|R:{}|G:{}|D:{}|T:{}|S:{}|Q:1|X:export KOAD_SESSION_ID={}",
            agent,
            rank_display,
            gh_pat,
            gdrive_token,
            tags.join(","),
            session_id,
            session_id
        );
    } else {
        println!(
            "
\x1b[1m--- KoadOS Neural Link Established ---\x1b[0m"
        );
        println!("Agent:    {}", agent);
        println!("Rank:     {}", rank_display);
        println!("Session:  {}", session_id);
        println!("Tags:     {}", tags.join(", "));
        println!("Lifeforce: Tethered via KOAD_SESSION_ID.");

        if let Some(ref drivers) = legacy_config.drivers {
            if let Some(driver) = drivers.get(&agent) {
                let b_path = driver
                    .bootstrap
                    .replace("~", &env::var("HOME").unwrap_or_default());
                if let Ok(content) = std::fs::read_to_string(b_path) {
                    println!(
                        "
\x1b[1m[BOOTSTRAP: {}]\x1b[0m
{}",
                        agent, content
                    );
                }
            }
        }

        if let Some(briefing) = mission_briefing {
            println!(
                "
\x1b[1mMission Briefing:\x1b[0m
{}",
                briefing
            );
        }
        println!(
            "\x1b[1m---------------------------------------\x1b[0m
"
        );
    }
    Ok(())
}
