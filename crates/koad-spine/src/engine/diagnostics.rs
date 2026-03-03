use crate::engine::redis::RedisClient;
use crate::engine::storage_bridge::KoadStorageBridge;
use koad_core::storage::StorageBridge;
use chrono::Utc;
use fred::interfaces::{
    ClientLike, HashesInterface, ListInterface, PubsubInterface, SetsInterface, StreamsInterface,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use std::path::PathBuf;
use std::collections::HashMap;
use sysinfo::System;
use tokio::time::sleep;
use tracing::info;

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
    identity: Arc<crate::engine::identity::KAILeaseManager>,
    sys: Arc<Mutex<System>>,
    skill_registry: Arc<Mutex<SkillRegistry>>,
}

impl ShipDiagnostics {
    pub fn new(
        redis: Arc<RedisClient>,
        storage: Arc<KoadStorageBridge>,
        identity: Arc<crate::engine::identity::KAILeaseManager>,
        skill_registry: Arc<Mutex<SkillRegistry>>,
    ) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            redis,
            storage,
            identity,
            sys: Arc::new(Mutex::new(sys)),
            skill_registry,
        }
    }

    pub async fn start_health_monitor(&self) {
        println!("ShipDiagnostics: Cognitive monitor active.");
        loop {
            // Heartbeat every 5 seconds
            sleep(Duration::from_secs(5)).await;

            // 1. Neural Bus Sentinel
            if let Err(e) = self.check_neural_bus().await {
                eprintln!("\x1b[31mSHIP ALERT: Neural Bus Disconnected: {}\x1b[0m", e);
                let _ = self.report_incident("Sentinel:NeuralBus", "CRITICAL", &format!("Redis connectivity lost: {}", e), false).await;
            }

            // 2. Crew Sentinel
            if let Err(e) = self.check_crew_readiness().await {
                eprintln!("\x1b[31mSHIP ALERT: Crew Status Critical: {}\x1b[0m", e);
            }

            // 3. Refresh System Stats
            if let Err(e) = self.run_integrity_scan().await {
                eprintln!("SHIP ALERT: Integrity Scan Failed: {}", e);
            }

            // 4. Check Service Health (Web Deck, Redis)
            if let Err(e) = self.check_services().await {
                eprintln!("SHIP ALERT: Service Health Check Failed: {}", e);
            }
        }
    }

    async fn check_neural_bus(&self) -> anyhow::Result<()> {
        let _: String = self.redis.client.ping().await?;
        Ok(())
    }

    async fn check_crew_readiness(&self) -> anyhow::Result<()> {
        // Essential Personnel Check
        let key = "koad:kai:Koad:lease";
        let exists: bool = self.redis.client.hexists("koad:state", key).await?;
        if !exists {
            let msg = "Essential Personnel Offline: KAI 'Koad' is currently DARK.";
            let _ = self.report_incident("Sentinel:Crew", "CRITICAL", msg, false).await;
            anyhow::bail!(msg);
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
        let mut sys = self.sys.lock().await;
        sys.refresh_cpu_usage();
        sys.refresh_memory();

        let skill_count = {
            let registry = self.skill_registry.lock().await;
            registry.skills.len()
        };

        let active_tasks: usize = self.redis.client.scard("koad:active_tasks").await.unwrap_or(0);

        let stats = SystemStats {
            cpu_usage: sys.global_cpu_info().cpu_usage(),
            memory_usage: sys.used_memory() / 1024 / 1024,
            uptime: System::uptime(),
            timestamp: Utc::now().timestamp(),
            skill_count,
            active_tasks,
        };

        let payload = serde_json::to_string(&stats)?;
        let _: () = self.redis.client.publish("koad:telemetry:stats", payload.clone()).await?;
        
        let _: Result<String, _> = self.redis.client.xadd(
            "koad:events:stream",
            false,
            None,
            "*",
            vec![
                ("source", "engine:diagnostics"),
                ("severity", "INFO"),
                ("message", "SYSTEM_HEARTBEAT"),
                ("metadata", &payload),
                ("timestamp", &stats.timestamp.to_string()),
            ],
        ).await;

        Ok(())
    }

    async fn check_services(&self) -> anyhow::Result<()> {
        let port = 3000;
        let addr = format!("127.0.0.1:{}", port);
        
        let status = match tokio::time::timeout(
            Duration::from_millis(500),
            tokio::net::TcpStream::connect(&addr)
        ).await {
            Ok(Ok(_)) => "UP".to_string(),
            _ => {
                let msg = format!("Web Deck (kgateway) unresponsive on port {}. Initiating recovery...", port);
                let _ = self.report_incident("Sentinel:Gateway", "WARN", &msg, false).await;
                let _ = self.restart_gateway().await;
                "RECOVERING".to_string()
            }
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
        
        // Update Identity-First Manifest
        let _ = self.update_crew_manifest().await;
        
        Ok(())
    }

    async fn update_crew_manifest(&self) -> anyhow::Result<()> {
        // 1. Get all registered identities from DB
        let sqlite = self.storage.sqlite.clone();
        let identities: Vec<String> = tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            let mut stmt = conn.prepare("SELECT name FROM identities")?;
            let rows = stmt.query_map([], |row: &rusqlite::Row| row.get::<_, String>(0))?;
            let mut names = Vec::new();
            for r in rows { names.push(r?); }
            Ok::<Vec<String>, anyhow::Error>(names)
        }).await??;

        // 2. Check leases and build manifest
        let mut wake_count = 0;
        let mut manifest = HashMap::new();

        for name in identities {
            let key = format!("koad:kai:{}:lease", name);
            let lease_data: Option<String> = self.redis.client.hget("koad:state", &key).await?;
            
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

        // 3. Publish Manifest to Redis for UI consumption
        let _: () = self.redis.client.hset("koad:state", ("crew_manifest", json!(manifest).to_string())).await?;
        let _: () = self.redis.client.hset("koad:state", ("wake_personnel", wake_count.to_string())).await?;

        // 4. ACTIVE PRUNING: Remove any koad:session:* keys that don't match an active lease
        let all_state: HashMap<String, String> = self.redis.client.hgetall("koad:state").await?;
        let active_session_ids: Vec<String> = manifest.values()
            .filter_map(|v| v["session_id"].as_str().map(|s| s.to_string()))
            .collect();

        for (key, _) in all_state {
            if key.starts_with("koad:session:") {
                let sid = key.replace("koad:session:", "");
                if !active_session_ids.contains(&sid) {
                    info!("Autonomic Sentinel: Pruning orphaned session key: {}", key);
                    let _: () = self.redis.client.hdel("koad:state", &key).await?;
                }
            }
        }

        Ok(())
    }

    async fn restart_gateway(&self) -> anyhow::Result<()> {
        println!("Autonomic Sentinel: Restarting kgateway...");
        let home = std::env::var("KOAD_HOME").unwrap_or_else(|_| "/home/ideans/.koad-os".to_string());
        let bin_path = PathBuf::from(&home).join("bin/kgateway");
        let log_path = PathBuf::from(&home).join("gateway.log");

        let _ = std::process::Command::new("nohup")
            .arg(bin_path)
            .arg("--addr")
            .arg("0.0.0.0:3000")
            .stdout(std::process::Stdio::from(std::fs::File::create(&log_path)?))
            .stderr(std::process::Stdio::from(std::fs::File::create(&log_path)?))
            .spawn()?;

        let _ = self.report_incident("Sentinel:Gateway", "INFO", "Autonomic recovery initiated: kgateway process spawned.", true).await;
        Ok(())
    }

    pub fn get_morning_report(&self) -> String {
        format!(
            "# Morning Report - {}\n\n- Hull: Stable\n- Engine: UDS/Redis Active\n- Services: Monitoring Web Deck\n- State: Condition Green",
            Utc::now().format("%Y-%m-%d")
        )
    }
}
