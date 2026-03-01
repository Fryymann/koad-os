use std::sync::Arc;
use crate::engine::redis::RedisClient;
use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;
use sysinfo::System;
use serde::{Serialize, Deserialize};
use fred::interfaces::PubsubInterface;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub uptime: u64,
    pub timestamp: i64,
}

pub struct ShipDiagnostics {
    _redis: Arc<RedisClient>,
    sys: Arc<tokio::sync::Mutex<System>>,
}

impl ShipDiagnostics {
    pub fn new(redis: Arc<RedisClient>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { 
            _redis: redis,
            sys: Arc::new(tokio::sync::Mutex::new(sys)),
        }
    }

    pub async fn start_health_monitor(&self) {
        println!("ShipDiagnostics: Cognitive monitor active.");
        loop {
            sleep(Duration::from_secs(2)).await;
            if let Err(e) = self.run_integrity_scan().await {
                eprintln!("SHIP ALERT: Cognitive Integrity Compromised: {}", e);
            }
        }
    }

    async fn run_integrity_scan(&self) -> anyhow::Result<()> {
        let mut sys = self.sys.lock().await;
        sys.refresh_cpu();
        sys.refresh_memory();

        let stats = SystemStats {
            cpu_usage: sys.global_cpu_info().cpu_usage(),
            memory_usage: sys.used_memory() / 1024 / 1024, // MB
            uptime: System::uptime(),
            timestamp: Utc::now().timestamp(),
        };

        let payload = serde_json::to_string(&stats)?;
        let _: () = self._redis.client.publish("koad:telemetry:stats", payload).await?;
        
        Ok(())
    }

    pub fn get_morning_report(&self) -> String {
        format!(
            "# Morning Report - {}\n\n- Hull: Stable\n- Engine: UDS/Redis Active\n- Crew: Discovery Complete\n- State: Condition Green",
            Utc::now().format("%Y-%m-%d")
        )
    }
}
