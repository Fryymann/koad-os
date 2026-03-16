use anyhow::{Context, Result};
use fred::interfaces::HashesInterface;
use koad_core::config::KoadConfig;
use koad_core::hierarchy::HierarchyManager;
use koad_core::utils::redis::RedisClient;
use koad_proto::cass::v1::hydration_service_client::HydrationServiceClient;
use koad_proto::cass::v1::HydrationRequest;
use koad_proto::citadel::v5::citadel_session_client::CitadelSessionClient;
use koad_proto::citadel::v5::{CloseRequest, LeaseRequest, TurnMetrics};
use std::env;
use std::io::Write;

/// Options for booting an agent session.
pub struct BootOptions {
    pub agent: String,
    pub project: bool,
    pub task: Option<String>,
    pub compact: bool,
    pub budget: u32,
    pub force: bool,
    pub role: String,
}

pub async fn handle_boot_command(
    opts: BootOptions,
    config: &KoadConfig,
) -> Result<()> {
    let agent = opts.agent;
    let force = opts.force;
    let budget = opts.budget;
    let compact = opts.compact;
    let project = opts.project;
    let role = opts.role;
    // --- [Laws of Consciousness: Occupancy Check] ---
    if let Ok(existing_sid) = env::var("KOAD_SESSION_ID") {
        if !existing_sid.is_empty() {
            let rc = RedisClient::new(&config.home.to_string_lossy(), false).await?;
            let session_key = format!("koad:session:{}", existing_sid);
            let val: Option<String> = rc
                .pool
                .hget("koad:state", &session_key)
                .await
                .unwrap_or(None);

            if val.is_some() {
                anyhow::bail!(
                    "\x1b[31mCONSCIOUSNESS_COLLISION\x1b[0m: Session {} is already active in this Body. \
                     Run `koad logout` to untether before booting a new agent.",
                    existing_sid
                );
            }
        }
    }

    let current_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let body_id = env::var("KOAD_BODY_ID").unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());

    // 1. Resolve Level
    let hierarchy = HierarchyManager::new(config.clone());
    let level = if project {
        hierarchy.resolve_level(&current_dir) as i32
    } else {
        0 // Level Unspecified
    };

    // 2. Pre-boot Hydration (CASS)
    // We do this BEFORE the lease so we can report the token cost in the CreateLease call.
    let mut cass = HydrationServiceClient::connect(config.network.cass_grpc_addr.clone())
        .await
        .context("Failed to connect to CASS Hydration service.")?;

    let hydration_resp = cass
        .hydrate(HydrationRequest {
            agent_name: agent.clone(),
            project_root: current_dir.to_string_lossy().to_string(),
            level,
            token_budget: budget,
            task_id: opts.task.unwrap_or_default(),
        })
        .await?;

    let packet = hydration_resp.into_inner();
    let context_file = config.home.join("current_context.md");
    let mut f = std::fs::File::create(&context_file)?;
    f.write_all(packet.markdown_packet.as_bytes())?;

    // 3. Citadel Handshake (Lease + Telemetry)
    let mut citadel = CitadelSessionClient::connect(config.network.spine_grpc_addr.clone())
        .await
        .context("Failed to connect to Citadel.")?;

    let lease_resp = citadel
        .create_lease(LeaseRequest {
            context: None,
            agent_name: agent.clone(),
            project_root: current_dir.to_string_lossy().to_string(),
            force,
            body_id: body_id.clone(),
            driver_id: "gemini-cli".to_string(),
            metrics: Some(TurnMetrics {
                input_tokens: 0,
                output_tokens: packet.estimated_tokens, // Reporting hydration cost as tokens_out
                thinking_tokens: 0,
                tool_calls: 0,
            }),
        })
        .await
        .map_err(|e| anyhow::anyhow!("Lease Denied: {}", e.message()))?;

    let lease = lease_resp.into_inner();
    let session_id = lease.session_id;

    // 4. Environment Export (The Link)
    if compact {
        println!(
            "I:{}|R:{}|S:{}|X:export KOAD_SESSION_ID={} KOAD_BODY_ID={} KOAD_CONTEXT_FILE={}",
            agent,
            role,
            session_id,
            session_id,
            body_id,
            context_file.display()
        );
    } else {
        println!("\n\x1b[1m--- KoadOS Neural Link Established ---\x1b[0m");
        println!("Agent:    {}", agent);
        println!("Session:  {}", session_id);
        println!(
            "Context:  {} ({} tokens)",
            context_file.display(),
            packet.estimated_tokens
        );
        println!("\n\x1b[32m[BOOT COMPLETE]\x1b[0m");

        // Output the actual eval strings
        println!("export KOAD_SESSION_ID=\"{}\"", session_id);
        println!("export KOAD_BODY_ID=\"{}\"", body_id);
        println!("export KOAD_CONTEXT_FILE=\"{}\"", context_file.display());
        println!("export KOAD_AGENT_NAME=\"{}\"", agent);
    }

    Ok(())
}

pub async fn handle_logout_command(session: Option<String>, config: &KoadConfig) -> Result<()> {
    let session_id = session
        .or_else(|| env::var("KOAD_SESSION_ID").ok())
        .context("No active session ID found.")?;

    let mut client = CitadelSessionClient::connect(config.network.spine_grpc_addr.clone()).await?;

    client
        .close_session(CloseRequest {
            context: None,
            session_id: session_id.clone(),
            summary_path: String::new(),
        })
        .await?;

    println!("\x1b[32m[OK]\x1b[0m Session {} untethered.", session_id);
    Ok(())
}
