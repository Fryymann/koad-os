use crate::db::KoadDB;
use anyhow::Result;
use koad_core::config::KoadConfig;
use koad_core::constants::{REDIS_KEY_STATE, REDIS_KEY_SYSTEM_STATS};
use koad_core::health::{HealthRegistry, HealthStatus};
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

    // 2. Backbone (Spine)
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
        if let Some(ref client) = redis_client {
            // 5. System Stats (Direct from Redis Data Plane)
            let res: Option<String> = client.pool.hget(REDIS_KEY_STATE, REDIS_KEY_SYSTEM_STATS).await?;
            if let Some(s) = res {
                let v: Value = serde_json::from_str(&s).unwrap_or_default();
                println!(
                    "
\x1b[1m--- Resource Allocation ---\x1b[0m"
                );
                println!("CPU Usage: {:.1}%", v["cpu_usage"].as_f64().unwrap_or(0.0));
                println!("Memory:    {} MB", v["memory_usage"].as_u64().unwrap_or(0));
                println!("Latency:   {:.2} ms (Bus)", v["latency_ms"].as_f64().unwrap_or(0.0));
            }

            // 6. Detailed Health Registry (The "Doctor" Report)
            let health_res: Option<String> = client.pool.hget(REDIS_KEY_STATE, "health_registry").await?;
            if let Some(h) = health_res {
                if let Ok(registry) = serde_json::from_str::<HealthRegistry>(&h) {
                    println!(
                        "
\x1b[1m--- Neural Grid Health Report ---\x1b[0m"
                    );
                    for check in registry.systems {
                        let status_str = match check.status {
                            HealthStatus::Pass => "\x1b[32m[OK]\x1b[0m",
                            HealthStatus::Warn => "\x1b[33m[WARN]\x1b[0m",
                            HealthStatus::Fail => "\x1b[31m[FAIL]\x1b[0m",
                            HealthStatus::Unknown => "\x1b[37m[???]\x1b[0m",
                        };
                        println!("{:<10} {:<30} | {}", status_str, check.name, check.message);
                    }
                }
            }

            // 7. Crew Manifest (Authoritative via ShipDiagnostics)
            println!(
                "
\x1b[1m--- Crew Manifest (Authoritative) ---\x1b[0m"
            );
            let manifest_res: Option<String> = client.pool.hget(REDIS_KEY_STATE, "crew_manifest").await?;
            let mut wake = 0;
            if let Some(m) = manifest_res {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&m) {
                    if let Some(crew) = json["manifest"].as_object() {
                        for (name, data) in crew {
                            if data["status"] == "WAKE" {
                                let sid = data["session_id"].as_str().unwrap_or("unknown");
                                let sid_short = if sid.len() > 8 { &sid[..8] } else { sid };
                                println!(
                                    "  - {:<10} [\x1b[32mWAKE\x1b[0m] (Session: {})",
                                    name,
                                    sid_short
                                );
                                wake += 1;
                            } else if full {
                                println!("  - {:<10} [\x1b[30mDARK\x1b[0m]", name);
                            }
                        }
                    }
                }
            }
            println!("Total Wake Personnel: {}", wake);
        } else {
            println!("\x1b[33m[WARN]\x1b[0m Redis offline. Telemetry unavailable.");
        }
    }

    println!("\x1b[1m---------------------------------------------------\x1b[0m");
    Ok(())
}

