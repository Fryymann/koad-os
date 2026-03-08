use crate::db::KoadDB;
use crate::utils::find_ghosts;
use anyhow::Result;
use koad_core::config::KoadConfig;
use koad_core::utils::redis::RedisClient;
use fred::interfaces::HashesInterface;
use serde_json::Value;
use sysinfo::System;

pub async fn handle_status_command(
    _json: bool,
    full: bool,
    config: &KoadConfig,
    _db: &KoadDB,
) -> Result<()> {
    println!("
\x1b[1m--- [TELEMETRY] Neural Link & Grid Integrity ---\x1b[0m");

    // 1. Engine Room (Redis Process/Socket)
    print!("{:<30}", "Engine Room (Redis):");
    let redis_client = match RedisClient::new(&config.home.to_string_lossy(), false).await {
        Ok(client) => {
            println!("\x1b[32m[PASS]\x1b[0m Hot-stream energized.");
            Some(client)
        }
        Err(_) => {
            println!("\x1b[31m[FAIL]\x1b[0m Neural Bus (koad.sock) missing or unresponsive.");
            None
        }
    };

    // 2. Backbone (kspine gRPC Socket)
    print!("{:<30}", "Backbone (Spine):");
    let spine_socket = config.home.join("kspine.sock");
    if spine_socket.exists() {
        println!("\x1b[32m[PASS]\x1b[0m Neural bus (kspine.sock) active.");
    } else {
        println!("\x1b[33m[WARN]\x1b[0m Orchestrator link severed. Some features offline.");
    }

    // 2.1 Web Deck (kgateway Process Check)
    print!("{:<30}", "Web Deck (Gateway):");
    let mut sys = System::new_all();
    sys.refresh_all();
    let is_gateway_running = sys
        .processes()
        .values()
        .any(|p| p.name().contains("kgateway"));
    if is_gateway_running {
        println!("\x1b[32m[PASS]\x1b[0m Gateway pulse detected.");
    } else {
        println!(
            "\x1b[31m[FAIL]\x1b[0m Web Deck is DARK. The Spine is attempting autonomic recovery."
        );
    }

    // 3. Memory Bank (SQLite)
    print!("{:<30}", "Memory Bank (SQLite):");
    let db_path = config.get_db_path();
    if db_path.exists() {
        match rusqlite::Connection::open(&db_path) {
            Ok(conn) => {
                let res: rusqlite::Result<i32> = conn.query_row("SELECT 1", [], |r| r.get(0));
                if res.is_ok() {
                    println!("\x1b[32m[PASS]\x1b[0m Sectors accessible.");
                } else {
                    println!("\x1b[31m[FAIL]\x1b[0m Database query failed.");
                }
            }
            Err(_) => println!("\x1b[31m[FAIL]\x1b[0m Database connection failed."),
        }
    } else {
        println!("\x1b[31m[FAIL]\x1b[0m Master record missing.");
    }

    if full {
        // 4. Ghost Process Detection
        let ghosts = find_ghosts(&config.home);
        if !ghosts.is_empty() {
            println!(
                "
\x1b[33m[WARN] Ghost Processes Detected ({}):\x1b[0m",
                ghosts.len()
            );
            for (pid, info) in ghosts {
                println!("  - PID {}: {}", pid, info);
            }
        }

        // 5. System Stats (Direct from Redis Data Plane)
        if let Some(ref client) = redis_client {
            let res: Option<String> = client.pool.hget("koad:state", "system_stats").await?;
            if let Some(s) = res {
                let v: Value = serde_json::from_str(&s).unwrap_or_default();
                println!(
                    "
\x1b[1m--- Resource Allocation ---\x1b[0m"
                );
                println!("CPU Usage: {:.1}%", v["cpu_usage"].as_f64().unwrap_or(0.0));
                println!("Memory:    {} MB", v["memory_usage"].as_u64().unwrap_or(0));
            }
        }

        // 6. Crew Manifest (Direct from Redis Data Plane - v5.0 CQRS)
        if let Some(ref client) = redis_client {
            println!(
                "
\x1b[1m--- Crew Manifest (Data Plane) ---\x1b[0m"
            );
            let state: std::collections::HashMap<String, String> = client.pool.hgetall("koad:state").await?;
            let mut wake = 0;
            
            for (key, val) in state {
                if key.starts_with("koad:session:") {
                    if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                        let data = if let Some(inner) = raw_json.get("data") {
                            inner
                        } else {
                            &raw_json
                        };
                        if let Ok(sess) = serde_json::from_value::<koad_core::session::AgentSession>(data.clone()) {
                            if sess.status == "active" {
                                println!(
                                    "  - {:<10} [{}] (Session: {})",
                                    sess.identity.name,
                                    sess.last_heartbeat.format("%H:%M:%S"),
                                    &sess.session_id[..8]
                                );
                                wake += 1;
                            }
                        }
                    }
                }
            }
            println!("Total Wake Personnel: {}", wake);
        } else {
            println!("\x1b[33m[WARN]\x1b[0m Redis offline. Manifest unavailable.");
        }
    }

    println!("\x1b[1m---------------------------------------------------\x1b[0m");
    Ok(())
}
