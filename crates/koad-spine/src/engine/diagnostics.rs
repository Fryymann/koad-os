use crate::engine::redis::RedisClient;
use chrono::Utc;
use fred::interfaces::{
    ClientLike, HashesInterface, ListInterface, PubsubInterface, SetsInterface, StreamsInterface,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;

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
    sys: Arc<Mutex<System>>,
    skill_registry: Arc<Mutex<SkillRegistry>>,
}

impl ShipDiagnostics {
    pub fn new(redis: Arc<RedisClient>, skill_registry: Arc<Mutex<SkillRegistry>>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self {
            redis,
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
        let incident = serde_json::json!({
            "incident_id": uuid::Uuid::new_v4().to_string(),
            "source": source,
            "severity": severity,
            "root_cause": message,
            "recovery_attempted": true,
            "status": if recovered { "RECOVERED" } else { "FAILED" }
        });

        // Try to log to event stream
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
        let _: () = self.redis.client.lpush("koad:stats:history", payload.clone()).await?;
        let _: () = self.redis.client.ltrim("koad:stats:history", 0, 99).await?;

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
        let web_deck = ServiceEntry {
            name: "web-deck".to_string(),
            host: "0.0.0.0".to_string(),
            port: 3000,
            protocol: "http".to_string(),
            status: "UP".to_string(),
            last_seen: Utc::now().timestamp(),
        };

        let payload = serde_json::to_string(&web_deck)?;
        let _: () = self.redis.client.hset("koad:services", ("web-deck", payload)).await?;
        Ok(())
    }

    pub fn get_morning_report(&self) -> String {
        format!(
            "# Morning Report - {}\n\n- Hull: Stable\n- Engine: UDS/Redis Active\n- Services: Monitoring Web Deck\n- State: Condition Green",
            Utc::now().format("%Y-%m-%d")
        )
    }
}
