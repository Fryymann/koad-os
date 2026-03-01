use std::sync::Arc;
use crate::engine::redis::RedisClient;
use std::time::Duration;
use tokio::time::sleep;
use chrono::Utc;
use sysinfo::System;
use serde::{Serialize, Deserialize};
use fred::interfaces::{PubsubInterface, HashesInterface, StreamsInterface};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStats {
    pub cpu_usage: f32,
    pub memory_usage: u64,
    pub uptime: u64,
    pub timestamp: i64,
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
    sys: Arc<tokio::sync::Mutex<System>>,
}

impl ShipDiagnostics {
    pub fn new(redis: Arc<RedisClient>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();
        Self { 
            redis,
            sys: Arc::new(tokio::sync::Mutex::new(sys)),
        }
    }

    pub async fn start_health_monitor(&self) {
        println!("ShipDiagnostics: Cognitive monitor active.");
        loop {
            // Heartbeat every 5 seconds
            sleep(Duration::from_secs(5)).await;
            
            // 1. Refresh System Stats
            if let Err(e) = self.run_integrity_scan().await {
                eprintln!("SHIP ALERT: Integrity Scan Failed: {}", e);
            }

            // 2. Check Service Health (Web Deck, Redis)
            if let Err(e) = self.check_services().await {
                eprintln!("SHIP ALERT: Service Health Check Failed: {}", e);
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
        
        // Publish to Hot Path (PubSub)
        let _: () = self.redis.client.publish("koad:telemetry:stats", payload.clone()).await?;

        // Add to Event Stream (Persistence)
        let _: () = self.redis.client.xadd(
            "koad:events:stream", 
            false, 
            None, 
            "*", 
            vec![
                ("source", "engine:diagnostics"),
                ("severity", "INFO"),
                ("message", "SYSTEM_HEARTBEAT"),
                ("metadata", &payload),
                ("timestamp", &stats.timestamp.to_string())
            ]
        ).await?;
        
        Ok(())
    }

    async fn check_services(&self) -> anyhow::Result<()> {
        // Simple TCP health check for the Web Deck (Placeholder logic)
        // In the future, this will probe the actual Axum port.
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
