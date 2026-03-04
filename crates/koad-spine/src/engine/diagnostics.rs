use crate::engine::redis::RedisClient;
use crate::engine::storage_bridge::KoadStorageBridge;
use koad_core::storage::StorageBridge;
use anyhow::Context;
use chrono::Utc;
use fred::interfaces::{
    ClientLike, HashesInterface, ListInterface, PubsubInterface, SetsInterface, StreamsInterface,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;
use std::path::PathBuf;
use std::collections::HashMap;
use sysinfo::System;
use tokio::time::sleep;
use tracing::{info, error, warn, debug};

use crate::discovery::SkillRegistry;
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub uptime: u64,
    pub timestamp: i64,
    pub skill_count: usize,
    pub active_tasks: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntry {
    pub name: String,
    pub host: String,
    pub port: u32,
    pub protocol: String,
    pub status: String,
    pub last_seen: i64,
}

pub struct ShipDiagnostics {
    redis: Arc<RedisClient>,
    storage: Arc<KoadStorageBridge>,
    _identity: Arc<crate::engine::identity::KAILeaseManager>,
    sys: Arc<Mutex<System>>,
    skill_registry: Arc<Mutex<SkillRegistry>>,
    pub last_heartbeat: Arc<AtomicI64>,
}

impl ShipDiagnostics {
    pub fn new(
        redis: Arc<RedisClient>,
        storage: Arc<KoadStorageBridge>,
        _identity: Arc<crate::engine::identity::KAILeaseManager>,
        skill_registry: Arc<Mutex<SkillRegistry>>,
    ) -> Self {
        let sys = System::new(); 
        Self {
            redis,
            storage,
            _identity,
            sys: Arc::new(Mutex::new(sys)),
            skill_registry,
            last_heartbeat: Arc::new(AtomicI64::new(Utc::now().timestamp())),
        }
    }

    pub async fn start_health_monitor(&self) {
        info!("ShipDiagnostics: Cognitive monitor loop starting...");
        let mut iteration = 0;
        loop {
            iteration += 1;
            
            // 1. Core Telemetry (Non-blocking PUBLISH)
            let _ = self.run_integrity_scan().await;

            // 2. Service & Port checks
            let _ = self.check_services().await;

            // 3. Crew manifest update (Identity-First)
            let _ = self.update_crew_manifest().await;

            // 4. Long-cycle Pruning (Every 1 minute)
            if iteration % 12 == 0 {
                let _ = self.prune_orphaned_sessions().await;
            }

            // 5. Vital Signs
            let _ = self.check_neural_bus().await;
            
            self.last_heartbeat.store(Utc::now().timestamp(), Ordering::SeqCst);
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn check_neural_bus(&self) -> anyhow::Result<()> {
        let _: String = self.redis.pool.next().ping().await?;
        Ok(())
    }

    async fn run_integrity_scan(&self) -> anyhow::Result<()> {
        let sys_arc = self.sys.clone();
        let scan_result = tokio::task::spawn_blocking(move || {
            if let Some(mut sys) = sys_arc.try_lock().ok() {
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                Ok((sys.global_cpu_info().cpu_usage(), sys.used_memory() / 1024 / 1024, System::uptime()))
            } else {
                Err(anyhow::anyhow!("System Mutex Locked"))
            }
        }).await;

        let (cpu, mem, uptime) = match scan_result {
            Ok(Ok(data)) => data,
            _ => (0.0, 0, 0)
        };

        let skill_count = {
            let registry = self.skill_registry.lock().await;
            registry.skills.len()
        };

        let stats = SystemStats {
            cpu_usage: cpu,
            memory_usage: mem,
            uptime,
            timestamp: Utc::now().timestamp(),
            skill_count,
            active_tasks: 0,
        };

        let payload = serde_json::to_string(&stats)?;
        let _: () = self.redis.pool.next().publish("koad:telemetry:stats", &payload).await?;
        // Persistent key for status --json
        let _: () = self.redis.pool.next().hset("koad:state", ("system_stats", payload)).await?;
        
        Ok(())
    }

    async fn check_services(&self) -> anyhow::Result<()> {
        let addr = std::env::var("GATEWAY_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        
        let status = match tokio::time::timeout(
            Duration::from_millis(500),
            tokio::net::TcpStream::connect(&addr)
        ).await {
            Ok(Ok(_)) => "UP".to_string(),
            _ => "DOWN".to_string()
        };

        let web_deck = ServiceEntry {
            name: "web-deck".to_string(),
            host: "0.0.0.0".to_string(),
            port: 3000,
            protocol: "http".to_string(),
            status,
            last_seen: Utc::now().timestamp(),
        };

        let payload = serde_json::to_string(&web_deck)?;
        let _: () = self.redis.pool.next().publish("koad:telemetry:services", &payload).await?;
        Ok(())
    }

    async fn update_crew_manifest(&self) -> anyhow::Result<()> {
        let sqlite = self.storage.sqlite.clone();
        let identities: Vec<String> = tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            let mut stmt = conn.prepare("SELECT name FROM identities")?;
            let rows = stmt.query_map([], |row: &rusqlite::Row| row.get::<_, String>(0))?;
            let mut names = Vec::new();
            for r in rows { names.push(r?); }
            Ok::<Vec<String>, anyhow::Error>(names)
        }).await.unwrap_or_else(|_| Ok(Vec::new()))?;

        let mut wake_count = 0;
        let mut manifest = HashMap::new();

        for name in identities {
            let key = format!("koad:kai:{}:lease", name);
            let lease_data: Option<String> = self.redis.pool.next().hget("koad:state", &key).await.unwrap_or(None);
            
            if let Some(data) = lease_data {
                if let Ok(lease) = serde_json::from_str::<serde_json::Value>(&data) {
                    // Robust timestamp parsing
                    let expires_at = lease["expires_at"].as_str().unwrap_or("");
                    if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                        if ts.with_timezone(&Utc) > Utc::now() {
                            manifest.insert(name, json!({
                                "status": "WAKE",
                                "session_id": lease["session_id"],
                                "driver": lease["driver_id"],
                                "last_seen": expires_at
                            }));
                            wake_count += 1;
                            continue;
                        }
                    }
                }
            }
            manifest.insert(name, json!({ "status": "DARK" }));
        }

        let payload = json!({
            "manifest": manifest,
            "wake_count": wake_count,
            "timestamp": Utc::now().timestamp()
        }).to_string();

        let _: () = self.redis.pool.next().publish("koad:telemetry:manifest", &payload).await?;
        // Persistent key for status --json
        let _: () = self.redis.pool.next().hset("koad:state", ("crew_manifest", payload)).await?;
        Ok(())
    }

    async fn prune_orphaned_sessions(&self) -> anyhow::Result<()> {
        let all_state: HashMap<String, String> = self.redis.pool.next().hgetall("koad:state").await.unwrap_or_default();
        let mut active_session_ids = Vec::new();
        
        // Find session IDs currently linked to leases
        for (key, val) in &all_state {
            if key.starts_with("koad:kai:") && key.ends_with(":lease") {
                if let Ok(lease) = serde_json::from_str::<serde_json::Value>(val) {
                    if let Some(sid) = lease["session_id"].as_str() {
                        active_session_ids.push(sid.to_string());
                    }
                }
            }
        }

        for (key, _) in all_state {
            if key.starts_with("koad:session:") {
                let sid = key.replace("koad:session:", "");
                if !active_session_ids.contains(&sid) {
                    info!("Autonomic Sentinel: Pruning orphaned session key: {}", key);
                    let _: () = self.redis.pool.next().hdel("koad:state", &key).await.unwrap_or(());
                }
            }
        }
        Ok(())
    }

    async fn restart_gateway(&self) -> anyhow::Result<()> {
        let addr = std::env::var("GATEWAY_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());
        let home = std::env::var("KOAD_HOME").context("KOAD_HOME not set. Cannot restart gateway.")?;
        let bin_path = PathBuf::from(&home).join("bin/kgateway");
        let log_path = PathBuf::from(&home).join("gateway.log");

        info!("Autonomic Sentinel: Re-spawning kgateway at {}...", addr);
        let _ = std::process::Command::new("nohup")
            .arg(bin_path)
            .arg("--addr")
            .arg(addr)
            .stdout(std::process::Stdio::from(std::fs::File::create(&log_path)?))
            .stderr(std::process::Stdio::from(std::fs::File::create(&log_path)?))
            .spawn()?;

        Ok(())
    }

    pub fn get_morning_report(&self) -> String {
        format!(
            "# Morning Report - {}\n\n- Hull: Stable\n- Engine: UDS/Redis Active\n- Services: Monitoring Web Deck\n- State: Condition Green",
            Utc::now().format("%Y-%m-%d")
        )
    }
}
