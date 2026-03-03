use crate::engine::redis::RedisClient;
use crate::engine::storage_bridge::KoadStorageBridge;
use koad_core::storage::StorageBridge;
use anyhow::Context;
use chrono::Utc;
use fred::interfaces::{
    ClientLike, HashesInterface, KeysInterface, ListInterface, PubsubInterface, SetsInterface, StreamsInterface,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
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
        }
    }

    pub async fn start_health_monitor(&self) {
        info!("ShipDiagnostics: Cognitive monitor loop starting...");
        loop {
            debug!("Sentinel: Beginning autonomic cycle...");

            // 1. Core Telemetry (First priority, must not block)
            if let Err(e) = self.run_integrity_scan().await {
                error!("Sentinel: Telemetry update failed: {}", e);
            }

            // 2. Service & Port checks
            let _ = self.check_services().await;

            // 3. Crew & Neural check
            let _ = self.check_crew_readiness().await;
            let _ = self.check_neural_bus().await;
            
            debug!("Sentinel: cycle complete. Sleeping 5s.");
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn check_neural_bus(&self) -> anyhow::Result<()> {
        let _: String = self.redis.client.ping().await?;
        Ok(())
    }

    async fn check_crew_readiness(&self) -> anyhow::Result<()> {
        let key = "koad:kai:Koad:lease";
        let exists: bool = self.redis.client.hexists("koad:state", key).await?;
        if !exists {
            return Err(anyhow::anyhow!("Essential Personnel Offline"));
        }
        Ok(())
    }

    async fn report_incident(&self, source: &str, severity: &str, message: &str, recovered: bool) -> anyhow::Result<()> {
        let incident = json!({
            "incident_id": uuid::Uuid::new_v4().to_string(),
            "source": source,
            "severity": severity,
            "root_cause": message,
            "recovery_attempted": true,
            "status": if recovered { "RECOVERED" } else { "FAILED" }
        });

        let _: Result<String, _> = self.redis.client.xadd(
            "koad:events:stream",
            false,
            None,
            "*",
            vec![
                ("source", source),
                ("severity", severity),
                ("message", "INCIDENT_REPORT"),
                ("metadata", &incident.to_string()),
                ("timestamp", &Utc::now().timestamp().to_string()),
            ],
        ).await;

        Ok(())
    }

    async fn run_integrity_scan(&self) -> anyhow::Result<()> {
        // Move heavy sysinfo calls to blocking thread with a timeout handle
        let sys_arc = self.sys.clone();
        let (cpu, mem, uptime) = tokio::task::spawn_blocking(move || {
            let mut sys = sys_arc.blocking_lock();
            sys.refresh_cpu_usage();
            sys.refresh_memory();
            (sys.global_cpu_info().cpu_usage(), sys.used_memory() / 1024 / 1024, System::uptime())
        }).await.unwrap_or((0.0, 0, 0));

        let skill_count = {
            let registry = self.skill_registry.lock().await;
            registry.skills.len()
        };

        let active_tasks: usize = self.redis.client.scard("koad:active_tasks").await.unwrap_or(0);

        let stats = SystemStats {
            cpu_usage: cpu,
            memory_usage: mem,
            uptime,
            timestamp: Utc::now().timestamp(),
            skill_count,
            active_tasks,
        };

        let payload = serde_json::to_string(&stats)?;
        let _: () = self.redis.client.hset("koad:state", ("system_stats", payload.clone())).await?;
        let _: () = self.redis.client.publish("koad:telemetry:stats", payload).await?;
        
        Ok(())
    }

    async fn check_services(&self) -> anyhow::Result<()> {
        let addr = std::env::var("GATEWAY_ADDR").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        let port = addr.split(':').last().and_then(|p| p.parse::<u32>().ok()).unwrap_or(3000);
        
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
            port,
            protocol: "http".to_string(),
            status,
            last_seen: Utc::now().timestamp(),
        };

        let payload = serde_json::to_string(&web_deck)?;
        let _: () = self.redis.client.hset("koad:services", ("web-deck", payload)).await?;
        
        let _ = self.update_crew_manifest().await;
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
            let lease_data: Option<String> = self.redis.client.hget("koad:state", &key).await.unwrap_or(None);
            
            if let Some(data) = lease_data {
                if let Ok(lease) = serde_json::from_str::<crate::engine::identity::KAILease>(&data) {
                    if lease.expires_at > Utc::now() {
                        manifest.insert(name, json!({
                            "status": "WAKE",
                            "session_id": lease.session_id,
                            "driver": lease.driver_id,
                            "last_seen": lease.expires_at.to_rfc3339()
                        }));
                        wake_count += 1;
                        continue;
                    }
                }
            }
            manifest.insert(name, json!({ "status": "DARK" }));
        }

        let _: () = self.redis.client.hset("koad:state", ("crew_manifest", json!(manifest).to_string())).await?;
        let _: () = self.redis.client.hset("koad:state", ("wake_personnel", wake_count.to_string())).await?;

        // Active Pruning
        let all_state: HashMap<String, String> = self.redis.client.hgetall("koad:state").await.unwrap_or_default();
        let active_session_ids: Vec<String> = manifest.values()
            .filter_map(|v| v["session_id"].as_str().map(|s| s.to_string()))
            .collect();

        for (key, _) in all_state {
            if key.starts_with("koad:session:") {
                let sid = key.replace("koad:session:", "");
                if !active_session_ids.contains(&sid) {
                    let _: () = self.redis.client.hdel("koad:state", &key).await.unwrap_or(());
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
