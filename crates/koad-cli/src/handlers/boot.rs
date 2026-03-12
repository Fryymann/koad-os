use crate::utils::{detect_context_tags, detect_model_tier, get_gh_pat_for_path};
use anyhow::{Context, Result};
use fred::interfaces::{HashesInterface, PubsubInterface};
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
    force: bool,
    role: String,
    config: &KoadConfig,
) -> Result<()> {
    // --- [Laws of Consciousness: Occupancy Check] ---
    // Gate 1: If KOAD_SESSION_ID is set, verify the session is actually alive before blocking.
    // A stale env var (inherited from a parent shell) should not prevent a fresh boot.
    if let Ok(existing_sid) = env::var("KOAD_SESSION_ID") {
        if !existing_sid.is_empty() {
            let session_is_alive = {
                match koad_core::utils::redis::RedisClient::new(
                    &config.home.to_string_lossy(),
                    false,
                )
                .await
                {
                    Ok(rc) => {
                        let session_key = format!("koad:session:{}", existing_sid);
                        let val: Option<String> = rc
                            .pool
                            .hget("koad:state", &session_key)
                            .await
                            .unwrap_or(None);
                        if let Some(data) = val {
                            // Check status field
                            serde_json::from_str::<serde_json::Value>(&data)
                                .ok()
                                .and_then(|v| {
                                    let inner = if v.get("data").is_some() {
                                        v["data"].clone()
                                    } else {
                                        v
                                    };
                                    inner["status"].as_str().map(|s| s == "active")
                                })
                                .unwrap_or(false)
                        } else {
                            false
                        }
                    }
                    Err(_) => false,
                }
            };

            if session_is_alive {
                anyhow::bail!(
                    "\x1b[31mCONSCIOUSNESS_COLLISION\x1b[0m: Session {} is already active in this Body. \
                     Run `koad logout` to untether before booting a new agent.",
                    existing_sid
                );
            } else if !compact {
                println!(
                    "\x1b[33m[Warning] Stale KOAD_SESSION_ID detected (session {} is no longer active). \
                     Proceeding with fresh boot.\x1b[0m",
                    existing_sid
                );
            }
        }
    }

    let model_tier = detect_model_tier();
    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let tags = detect_context_tags(&current_dir);
    let (gh_pat, _) = get_gh_pat_for_path(&current_dir, &role, config);
    let gdrive_token = "GDRIVE_PERSONAL_TOKEN";

    // Generate a unique Body ID for this terminal session.
    // Reuse KOAD_BODY_ID if already set (survives re-runs within the same shell).
    let body_id = env::var("KOAD_BODY_ID")
        .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

    let mut client = SpineServiceClient::connect(config.network.spine_grpc_addr.clone())
        .await
        .context("Failed to connect to Spine Backbone.")?;

    // --- [Sovereign Pruning] ---
    // Requires explicit --force flag. Without it, an existing live sovereign session
    // blocks boot to prevent silent cross-terminal session assassination.
    let is_sovereign = if let Some(id_config) = config.identities.get(&agent) {
        let r = id_config.rank.to_lowercase();
        r == "admiral" || r == "captain"
    } else {
        // Fallback for legacy or unknown agents
        agent == "Tyr" || agent == "Dood"
    };

    if is_sovereign {
        let redis_client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
        let pool = &redis_client.pool;
        let lease_field = format!("koad:kai:{}:lease", agent);

        let existing_lease: Option<String> = pool.hget("koad:state", &lease_field).await?;
        let live_lease = existing_lease.as_deref().and_then(|data| {
            serde_json::from_str::<serde_json::Value>(data).ok().and_then(|v| {
                let expires_str = v["expires_at"].as_str()?.to_string();
                let session_id = v["session_id"].as_str()?.to_string();
                let body_id = v["body_id"].as_str().unwrap_or("unknown").to_string();
                chrono::DateTime::parse_from_rfc3339(&expires_str)
                    .ok()
                    .filter(|exp| exp.with_timezone(&chrono::Utc) > chrono::Utc::now())
                    .map(|_| (session_id, body_id))
            })
        });

        if let Some((ref live_sid, ref live_body)) = live_lease {
            if !force {
                anyhow::bail!(
                    "\x1b[31mSOVEREIGN_OCCUPIED\x1b[0m: {} is already active in Body {} (Session: {}).\n\
                     Options:\n  \
                     1. Run `koad logout --session {}` in that terminal to release cleanly.\n  \
                     2. Run `koad boot --agent {} --force` to take over (orphans the existing session).",
                    agent, live_body, live_sid, live_sid, agent
                );
            }

            // --force: explicit takeover — notify and prune the existing sovereign session
            if !compact {
                println!(
                    "\x1b[33m[Sovereign Override --force] Taking over existing session {} for {}...\x1b[0m",
                    live_sid, agent
                );
            }

            // Notification Broadcast
            let takeover_msg = serde_json::json!({
                "type": "SIG_FORCE_TAKEOVER",
                "session_id": live_sid,
                "agent": agent,
                "new_body_id": body_id.clone()
            });
            let _: () = pool.next().publish("koad:sessions", takeover_msg.to_string()).await?;
        }

        if force || existing_lease.is_none() || live_lease.is_none() {
            // Clear Lease
            let _: () = pool.next().hdel("koad:state", &lease_field).await?;

            // Clear identity mapping
            let identity_field = format!("koad:identity:{}", agent);
            let _: () = pool.next().hdel("koad:state", &identity_field).await?;

            // Scan and prune stale sessions
            let all_state: std::collections::HashMap<String, String> =
                pool.next().hgetall("koad:state").await?;
            for (key, val) in all_state {
                if key.starts_with("koad:session:")
                    && val.contains(&format!("\"name\":\"{}\"", agent))
                {
                    let _: () = pool.next().hdel("koad:state", &key).await?;
                    let sid = key.replace("koad:session:", "");
                    let msg = serde_json::json!({ "type": "SESSION_PRUNED", "session_id": sid });
                    let _: () = pool.next().publish("koad:sessions", msg.to_string()).await?;
                }
            }
        }
    }

    let driver_id = if env::var("GEMINI_CLI").is_ok() {
        "gemini".to_string()
    } else if env::var("CODEX_CLI").is_ok() {
        "codex".to_string()
    } else if env::var("CLAUDE_CODE").is_ok() {
        "claude".to_string()
    } else {
        "cli".to_string()
    };

    // --- [Path-Aware Project Detection] ---
    let mut project_name = if project { "active" } else { "default" }.to_string();
    if project {
        if let Some((name, _)) = config.resolve_project_context(&current_dir) {
            project_name = name;
        }
    }

    let resp = client
        .initialize_session(InitializeSessionRequest {
            agent_name: agent.clone(),
            agent_role: role.clone(),
            project_name,
            environment: EnvironmentType::Wsl as i32,
            driver_id: driver_id.clone(),
            model_tier,
            model_name: env::var("GEMINI_MODEL")
                .or_else(|_| env::var("CODEX_MODEL"))
                .unwrap_or_else(|_| "unknown".to_string()),
            body_id: body_id.clone(),
            force,
            session_id: env::var("KOAD_SESSION_ID").unwrap_or_default(),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Lease Denied: {}", e.message()))?;

    let package = resp.into_inner();
    let session_id = package.session_id;
    let mission_briefing = package.intelligence.map(|i| i.mission_briefing);

    // --- [Autonomic Nervous System: Heartbeat Daemon] ---
    if let Ok(bin_path) = env::current_exe() {
        let _ = std::process::Command::new(&bin_path)
            .arg("system")
            .arg("heartbeat")
            .arg("--daemon")
            .arg("--session")
            .arg(&session_id)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();

        // Ensure Watchdog is active
        use sysinfo::System;
        let mut sys = System::new_all();
        sys.refresh_all();
        let is_watchdog_running = sys
            .processes()
            .values()
            .any(|p| p.name().contains("koad-watchdog"));

        if !is_watchdog_running {
            let _ = std::process::Command::new(&bin_path)
                .arg("watchdog")
                .arg("--daemon")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }
    }

    // Extract true rank from identity_json
    let identity: koad_core::identity::Identity = serde_json::from_str(&package.identity_json)
        .context("Failed to deserialize identity from Spine")?;
    let rank_display = format!("{:?}", identity.rank);

    if compact {
        println!(
            "I:{}|R:{}|G:{}|D:{}|T:{}|S:{}|Q:1|X:export KOAD_SESSION_ID={} KOAD_BODY_ID={}",
            agent,
            rank_display,
            gh_pat,
            gdrive_token,
            tags.join(","),
            session_id,
            session_id,
            body_id
        );
    } else {
        println!(
            "
\x1b[1m--- KoadOS Neural Link Established ---\x1b[0m"
        );
        println!("Agent:    {}", agent);
        println!("Rank:     {}", rank_display);
        println!("Session:  {}", session_id);
        println!("Body:     {}", body_id);
        println!("Tags:     {}", tags.join(", "));
        println!("Lifeforce: Tethered via KOAD_SESSION_ID.");
        println!("Shell:    Run `export KOAD_SESSION_ID={} KOAD_BODY_ID={}` to bind this shell.", session_id, body_id);

        // --- [Ghost & Body Bootstrap] ---
        // 1. Load Persona Bootstrap (The Ghost)
        let identity_config = config.identities.get(&agent)
            .or_else(|| config.identities.get(&agent.to_lowercase()));

        if let Some(id_config) = identity_config {
            if let Some(ref b_path_raw) = id_config.bootstrap {
                let b_path = b_path_raw.replace("~", &env::var("HOME").unwrap_or_default());
                if let Ok(content) = std::fs::read_to_string(b_path) {
                    println!("\n\x1b[1m[PERSONA: {}]\x1b[0m\n{}", agent, content);
                }
            }
        }

        // 2. Load Interface Bootstrap (The Body)
        let interface_config = config.interfaces.get(&driver_id)
            .or_else(|| config.interfaces.get(&driver_id.to_lowercase()));

        if let Some(if_config) = interface_config {
            let b_path = if_config.bootstrap.replace("~", &env::var("HOME").unwrap_or_default());
            if let Ok(content) = std::fs::read_to_string(b_path) {
                println!("\n\x1b[1m[INTERFACE: {}]\x1b[0m\n{}", driver_id, content);
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

pub async fn handle_logout_command(
    session: Option<String>,
    config: &KoadConfig,
) -> Result<()> {
    let session_id = session.or_else(|| env::var("KOAD_SESSION_ID").ok())
        .context("No active session ID found. Provide --session or ensure KOAD_SESSION_ID is set.")?;

    let mut client = SpineServiceClient::connect(config.network.spine_grpc_addr.clone())
        .await
        .context("Failed to connect to Spine Backbone.")?;

    println!(">>> Terminating session {}...", session_id);

    client.terminate_session(crate::utils::authenticated_request(TerminateSessionRequest {
        session_id: session_id.clone(),
    })).await?;

    println!("\x1b[32m[OK]\x1b[0m Session untethered successfully.");
    println!("Shell:    Run `unset KOAD_SESSION_ID KOAD_BODY_ID` to clear this shell's binding.");
    Ok(())
}
