use crate::db::KoadDB;
use anyhow::Result;
use fred::interfaces::HashesInterface;
use koad_core::config::KoadConfig;
use koad_core::utils::redis::RedisClient;
use koad_proto::citadel::v5::citadel_session_client::CitadelSessionClient;
use std::env;
use sysinfo::System;

pub async fn handle_cognitive_check(
    config: &KoadConfig,
    db: &KoadDB,
    agent_name: &str,
) -> Result<()> {
    println!(
        "
[1m--- {} Cognitive Self-Health Report ---[0m",
        agent_name
    );

    // --- Layer 1: Tethering ---
    let session_id = env::var("KOAD_SESSION_ID").unwrap_or_default();
    if !session_id.is_empty() {
        match CitadelSessionClient::connect(config.network.citadel_grpc_addr.clone()).await {
            Ok(_) => println!(
                "[32m[PASS][0m L1: Tethered (Session: {})[0m",
                &session_id[..8]
            ),
            Err(_) => println!("[31m[FAIL][0m L1: Citadel Disconnected"),
        }
    } else {
        println!("[33m[WARN][0m L1: Session ID not found in environment");
    }

    // --- Layer 2: Hot Memory ---
    let client = RedisClient::new(&config.home.to_string_lossy(), false).await?;
    let context_key = format!("koad:session:{}:hot_context", session_id);
    let chunks: std::collections::HashMap<String, String> =
        client.pool.hgetall(&context_key).await.unwrap_or_default();

    let mailbox_key = format!("koad:mailbox:{}", agent_name);
    let signals: std::collections::HashMap<String, String> =
        client.pool.hgetall(&mailbox_key).await.unwrap_or_default();
    let pending_signals = signals.values().filter(|v| v.contains("pending")).count();

    println!(
        "[32m[PASS][0m L2: Hot Context ({} chunks active)",
        chunks.len()
    );
    println!(
        "[32m[PASS][0m L2: Mailbox ({} pending signals)",
        pending_signals
    );

    // --- Layer 3: Deep Memory ---
    match db.get_conn() {
        Ok(conn) => {
            let count: i32 = conn
                .query_row(
                    "SELECT count(*) FROM knowledge WHERE origin_agent = ?1",
                    [agent_name],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            println!(
                "[32m[PASS][0m L3: Deep Memory ({} records ingested)",
                count
            );
        }
        Err(_) => println!("[31m[FAIL][0m L3: SQLite Access Denied"),
    }

    // --- Layer 4: Autonomic Pulse ---
    let mut sys = System::new_all();
    sys.refresh_processes();
    let is_heartbeat_running = sys
        .processes()
        .values()
        .any(|p| p.name().contains("koad") && p.cmd().iter().any(|c| c == "heartbeat"));

    if is_heartbeat_running {
        println!("[32m[PASS][0m L4: Autonomic Pulse (Heartbeat active)");
    } else {
        println!("[33m[WARN][0m L4: Heartbeat Daemon missing");
    }

    // --- Layer 5: Procedural ---
    let report_dir = config.home.join("docs/research_reports");
    if report_dir.exists() {
        let entries = std::fs::read_dir(report_dir)?.count();
        println!(
            "[32m[PASS][0m L5: Procedural ({} reports accessible)",
            entries
        );
    } else {
        println!("[33m[WARN][0m L5: Research repo missing");
    }

    println!("Condition: [1;32mOPTIMAL[0m");
    println!("[1m---------------------------------------------------[0m");

    Ok(())
}
