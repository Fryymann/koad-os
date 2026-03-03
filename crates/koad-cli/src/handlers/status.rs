use anyhow::Result;
use koad_core::config::KoadConfig;
use serde_json::Value;
use std::collections::HashMap;
use sysinfo::System;
use crate::db::KoadDB;
use crate::utils::find_ghosts;

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
    if config.redis_socket.exists() {
        match redis::Client::open(format!("redis+unix://{}", config.redis_socket.display()))
        {
            Ok(client) => {
                if let Ok(mut con) = client.get_connection() {
                    let _: String = redis::cmd("PING")
                        .query(&mut con)
                        .unwrap_or_else(|_| "FAIL".into());
                    println!("\x1b[32m[PASS]\x1b[0m Hot-stream energized.");
                } else {
                    println!("\x1b[31m[FAIL]\x1b[0m Ghost Socket Detected (Connection Refused).");
                }
            }
            Err(_) => println!("\x1b[31m[FAIL]\x1b[0m Client initialization failed."),
        }
    } else {
        println!("\x1b[31m[FAIL]\x1b[0m Neural Bus (koad.sock) missing.");
    }

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
    let is_gateway_running = sys.processes().values().any(|p| p.name().contains("kgateway"));
    if is_gateway_running {
        println!("\x1b[32m[PASS]\x1b[0m Gateway pulse detected.");
    } else {
        println!("\x1b[31m[FAIL]\x1b[0m Web Deck is DARK. The Spine is attempting autonomic recovery.");
    }

    // 3. Memory Bank (SQLite)
    print!("{:<30}", "Memory Bank (SQLite):");
    let db_path = config.get_db_path();
    if db_path.exists() {
        match rusqlite::Connection::open(&db_path) {
            Ok(conn) => {
                let res: rusqlite::Result<i32> =
                    conn.query_row("SELECT 1", [], |r| r.get(0));
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

        // 5. System Stats
        if config.redis_socket.exists() {
            if let Ok(client) = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display())) {
                if let Ok(mut con) = client.get_connection() {
                    let res: Option<String> = redis::cmd("HGET").arg("koad:state").arg("system_stats").query(&mut con).unwrap_or(None);
                    if let Some(s) = res {
                        let v: Value = serde_json::from_str(&s).unwrap_or_default();
                        println!("
\x1b[1m--- Resource Allocation ---\x1b[0m");
                        println!("CPU Usage: {:.1}%", v["cpu_usage"].as_f64().unwrap_or(0.0));
                        println!("Memory:    {} MB", v["memory_usage"].as_u64().unwrap_or(0));
                    }
                }
            }
        }

        // 6. Crew Manifest
        if config.redis_socket.exists() {
            if let Ok(client) = redis::Client::open(format!("redis+unix://{}", config.redis_socket.display())) {
                if let Ok(mut con) = client.get_connection() {
                    let manifest_json: Option<String> = redis::cmd("HGET").arg("koad:state").arg("crew_manifest").query(&mut con).unwrap_or(None);
                    if let Some(m) = manifest_json {
                        if let Ok(manifest) = serde_json::from_str::<HashMap<String, Value>>(&m) {
                            println!("
\x1b[1m--- Crew Manifest (WAKE) ---\x1b[0m");
                            let mut wake = 0;
                            for (name, data) in manifest {
                                if data["status"] == "WAKE" {
                                    println!("  - {:<10} [{}]", name, data["last_seen"].as_str().unwrap_or(""));
                                    wake += 1;
                                }
                            }
                            println!("Total Wake Personnel: {}", wake);
                        }
                    }
                }
            }
        }
    }

    println!("\x1b[1m---------------------------------------------------\x1b[0m
");
    Ok(())
}
