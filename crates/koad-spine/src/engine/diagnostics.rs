use koad_core::health::{HealthCheck, HealthRegistry, HealthStatus};
use koad_core::intelligence::FactCard;
use koad_core::utils::redis::RedisClient;
use crate::engine::storage_bridge::KoadStorageBridge;
use anyhow::Context;
use chrono::Utc;
use fred::interfaces::{ClientLike, HashesInterface, PubsubInterface, StreamsInterface};
use koad_core::storage::StorageBridge;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::System;
use tokio::time::sleep;
use tracing::{error, info, warn};

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
    pub latency_ms: f64,
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
    health_registry: Arc<Mutex<HealthRegistry>>,
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
            health_registry: Arc::new(Mutex::new(HealthRegistry::new())),
            last_heartbeat: Arc::new(AtomicI64::new(Utc::now().timestamp())),
        }
    }

    pub async fn start_health_monitor(&self) {
        info!("ShipDiagnostics: Cognitive monitor loop starting...");
        let mut iteration = 0;
        loop {
            iteration += 1;

            let mut current_health = HealthRegistry::new();

            // 1. Registry Integrity (Self-Healing - MUST BE FIRST)
            let _ = self.check_registry_integrity(&mut current_health).await;

            // 1.1 Memory Bank (SQLite)
            let _ = self.check_memory_bank(&mut current_health).await;

            // 1.2 Ghost Process Detection
            let _ = self.check_ghosts(&mut current_health).await;

            // 2. Vital Signs (Latency Check)
            let latency_ms = match self.check_neural_bus(&mut current_health).await {
                Ok(ms) => ms,
                Err(_) => 0.0,
            };

            // 3. Core Telemetry (Non-blocking PUBLISH)
            let _ = self.run_integrity_scan(latency_ms).await;

            // 4. Service & Port checks
            let _ = self.check_services(&mut current_health).await;

            // 5. Crew manifest update (Identity-First)
            let _ = self.update_crew_manifest().await;

            // 5.1 Context Curation (Intelligence L2 -> L3)
            let _ = self.curate_intelligence().await;

            // 5.2 Cognitive Quicksave (Every 5 minutes)
            if iteration % 60 == 0 {
                let _ = self.perform_cognitive_quicksave().await;
            }

            // 6. Long-cycle Pruning (Every 1 minute)
            if iteration % 12 == 0 {
                let _ = self.prune_orphaned_sessions().await;
            }

            // Store Health Registry in state
            {
                let mut registry = self.health_registry.lock().await;
                *registry = current_health;
                if let Ok(payload) = serde_json::to_string(&*registry) {
                    let _: () = self
                        .redis
                        .pool
                        .next()
                        .hset("koad:state", ("health_registry", payload))
                        .await
                        .unwrap_or(());
                }
            }

            self.last_heartbeat
                .store(Utc::now().timestamp(), Ordering::SeqCst);
            sleep(Duration::from_secs(5)).await;
        }
    }

    async fn check_neural_bus(&self, registry: &mut HealthRegistry) -> anyhow::Result<f64> {
        let start = Instant::now();
        let result: anyhow::Result<String> = self.redis.pool.next().ping().await.map_err(|e| e.into());
        let duration = start.elapsed();
        let latency_ms = duration.as_secs_f64() * 1000.0;

        match result {
            Ok(_) => {
                registry.add(HealthCheck {
                    name: "Neural Bus (Redis)".to_string(),
                    status: if latency_ms < 10.0 { HealthStatus::Pass } else { HealthStatus::Warn },
                    message: format!("Hot-stream energized. Latency: {:.2}ms", latency_ms),
                    last_checked: Utc::now().timestamp(),
                    metadata: Some(json!({ "latency_ms": latency_ms })),
                });
                Ok(latency_ms)
            }
            Err(e) => {
                registry.add(HealthCheck {
                    name: "Neural Bus (Redis)".to_string(),
                    status: HealthStatus::Fail,
                    message: format!("Neural Bus unresponsive: {}", e),
                    last_checked: Utc::now().timestamp(),
                    metadata: None,
                });
                Err(e)
            }
        }
    }

    async fn check_registry_integrity(&self, registry: &mut HealthRegistry) -> anyhow::Result<()> {
        let initialized: bool = self
            .redis
            .pool
            .next()
            .hexists("koad:state", "initialized")
            .await?;
        
        if !initialized {
            warn!("Sentinel: Registry state loss detected. Triggering autonomic hydration...");
            let _ = self
                .report_incident(
                    "Sentinel:Registry",
                    "CRITICAL",
                    "Redis state incomplete. Auto-hydration initiated.",
                    false,
                )
                .await;

            if let Err(e) = self.storage.hydrate_all().await {
                error!("Sentinel: Registry hydration FAILED: {}", e);
                registry.add(HealthCheck {
                    name: "Registry Integrity".to_string(),
                    status: HealthStatus::Fail,
                    message: format!("State loss detected. Hydration FAILED: {}", e),
                    last_checked: Utc::now().timestamp(),
                    metadata: None,
                });
            } else {
                // Mark as initialized
                let _: () = self
                    .redis
                    .pool
                    .next()
                    .hset("koad:state", ("initialized", "true"))
                    .await?;
                info!("Sentinel: Registry hydration complete. State restored.");
                registry.add(HealthCheck {
                    name: "Registry Integrity".to_string(),
                    status: HealthStatus::Pass,
                    message: "State recovered via autonomic hydration.".to_string(),
                    last_checked: Utc::now().timestamp(),
                    metadata: None,
                });
            }
        } else {
            registry.add(HealthCheck {
                name: "Registry Integrity".to_string(),
                status: HealthStatus::Pass,
                message: "Hot-path state synchronized with Memory Bank.".to_string(),
                last_checked: Utc::now().timestamp(),
                metadata: None,
            });
        }
        Ok(())
    }

    async fn report_incident(
        &self,
        source: &str,
        severity: &str,
        message: &str,
        recovered: bool,
    ) -> anyhow::Result<()> {
        let incident = json!({
            "incident_id": uuid::Uuid::new_v4().to_string(),
            "source": source,
            "severity": severity,
            "root_cause": message,
            "recovery_attempted": true,
            "status": if recovered { "RECOVERED" } else { "FAILED" }
        });

        let _: Result<String, _> = self
            .redis
            .pool
            .next()
            .xadd(
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
            )
            .await;

        Ok(())
    }

    async fn run_integrity_scan(&self, latency_ms: f64) -> anyhow::Result<()> {
        let sys_arc = self.sys.clone();
        let scan_result = tokio::task::spawn_blocking(move || {
            if let Ok(mut sys) = sys_arc.try_lock() {
                sys.refresh_cpu_usage();
                sys.refresh_memory();
                Ok((
                    sys.global_cpu_info().cpu_usage(),
                    sys.used_memory() / 1024 / 1024,
                    System::uptime(),
                ))
            } else {
                Err(anyhow::anyhow!("System Mutex Locked"))
            }
        })
        .await;

        let (cpu, mem, uptime) = match scan_result {
            Ok(Ok(data)) => data,
            _ => (0.0, 0, 0),
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
            latency_ms,
        };

        let payload = serde_json::to_string(&stats)?;
        let _: () = self
            .redis
            .pool
            .next()
            .publish("koad:telemetry:stats", &payload)
            .await?;
        // Persistent key for status --json
        let _: () = self
            .redis
            .pool
            .next()
            .hset("koad:state", ("system_stats", payload))
            .await?;

        Ok(())
    }

    async fn check_services(&self, registry: &mut HealthRegistry) -> anyhow::Result<()> {
        let addr = std::env::var("GATEWAY_ADDR")
            .unwrap_or_else(|_| koad_core::constants::DEFAULT_GATEWAY_ADDR.to_string());

        let status = match tokio::time::timeout(
            Duration::from_millis(500),
            tokio::net::TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => "UP".to_string(),
            _ => "DOWN".to_string(),
        };

        if status == "DOWN" {
            // Attempt autonomic recovery if down
            let _ = self.autonomic_recovery("web-deck").await;
        }

        registry.add(HealthCheck {
            name: "Web Deck (Gateway)".to_string(),
            status: if status == "UP" { HealthStatus::Pass } else { HealthStatus::Fail },
            message: if status == "UP" { "Gateway pulse detected." .to_string() } else { "Gateway is DARK. Recovery attempted.".to_string() },
            last_checked: Utc::now().timestamp(),
            metadata: Some(json!({ "addr": addr })),
        });

        let web_deck = ServiceEntry {
            name: "web-deck".to_string(),
            host: "0.0.0.0".to_string(),
            port: koad_core::constants::DEFAULT_GATEWAY_PORT,
            protocol: "http".to_string(),
            status,
            last_seen: Utc::now().timestamp(),
        };

        let payload = serde_json::to_string(&web_deck)?;
        let _: () = self
            .redis
            .pool
            .next()
            .publish("koad:telemetry:services", &payload)
            .await?;
        Ok(())
    }

    async fn autonomic_recovery(&self, service: &str) -> anyhow::Result<()> {
        warn!("ShipDiagnostics: Autonomic recovery triggered for {}...", service);
        match service {
            "web-deck" => {
                if let Err(e) = self._restart_gateway().await {
                    error!("ShipDiagnostics: Recovery failed for web-deck: {}", e);
                } else {
                    info!("ShipDiagnostics: web-deck restart command issued.");
                }
            }
            "ghosts" => {
                let home = std::env::var("KOAD_HOME").unwrap_or_default();
                let ghosts = koad_core::utils::pid::find_ghosts(std::path::Path::new(&home));
                for (_pid, pf) in ghosts {
                    warn!("ShipDiagnostics: Purging stale PID file: {}", pf);
                    let pf_path = std::path::Path::new(&home).join(pf.split_whitespace().last().unwrap_or(""));
                    if pf_path.exists() {
                        let _ = std::fs::remove_file(pf_path);
                    }
                }
            }
            _ => warn!("ShipDiagnostics: No recovery protocol for {}", service),
        }
        Ok(())
    }

    async fn check_memory_bank(&self, registry: &mut HealthRegistry) -> anyhow::Result<()> {
        let sqlite = self.storage.sqlite.clone();
        let result = tokio::task::spawn_blocking(move || {
            let conn = sqlite.blocking_lock();
            let res: rusqlite::Result<i32> = conn.query_row("SELECT 1", [], |r| r.get(0));
            res.is_ok()
        })
        .await
        .unwrap_or(false);

        registry.add(HealthCheck {
            name: "Memory Bank (SQLite)".to_string(),
            status: if result { HealthStatus::Pass } else { HealthStatus::Fail },
            message: if result { "Sectors accessible." .to_string() } else { "Database query failed.".to_string() },
            last_checked: Utc::now().timestamp(),
            metadata: None,
        });
        Ok(())
    }

    async fn check_ghosts(&self, registry: &mut HealthRegistry) -> anyhow::Result<()> {
        let home = std::env::var("KOAD_HOME").unwrap_or_default();
        let ghosts = koad_core::utils::pid::find_ghosts(std::path::Path::new(&home));
        
        if ghosts.is_empty() {
            registry.add(HealthCheck {
                name: "Ghost Processes".to_string(),
                status: HealthStatus::Pass,
                message: "No stale PID files detected.".to_string(),
                last_checked: Utc::now().timestamp(),
                metadata: None,
            });
        } else {
            let message = format!("{} stale PID files detected. Recovery required.", ghosts.len());
            registry.add(HealthCheck {
                name: "Ghost Processes".to_string(),
                status: HealthStatus::Warn,
                message,
                last_checked: Utc::now().timestamp(),
                metadata: Some(json!({ "count": ghosts.len() })),
            });
            let _ = self.autonomic_recovery("ghosts").await;
        }
        Ok(())
    }

    async fn update_crew_manifest(&self) -> anyhow::Result<()> {
        let sqlite = self.storage.sqlite.clone();
        
        let db_identities: HashMap<String, String> = tokio::task::spawn_blocking({
            let sqlite_clone = sqlite.clone();
            move || -> HashMap<String, String> {
                let mut identities = HashMap::new();
                let conn = sqlite_clone.blocking_lock();
                if let Ok(mut stmt) = conn.prepare("SELECT name, role FROM identities") {
                    if let Ok(rows) = stmt.query_map([], |row: &rusqlite::Row| {
                        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
                    }) {
                        for r in rows {
                            if let Ok((name, role)) = r {
                                identities.insert(name, role);
                            }
                        }
                    }
                }
                identities
            }
        }).await.unwrap_or_default();

        let all_state: HashMap<String, String> = self
            .redis
            .pool
            .next()
            .hgetall("koad:state")
            .await
            .unwrap_or_default();

        let mut wake_count = 0;
        let mut manifest = HashMap::new();

        // 1. Process all active leases first to ensure no one is dropped
        for (key, val) in &all_state {
            if key.starts_with("koad:kai:") && key.ends_with(":lease") {
                let name = key.replace("koad:kai:", "").replace(":lease", "");
                if let Ok(lease) = serde_json::from_str::<serde_json::Value>(val) {
                    let expires_at = lease["expires_at"].as_str().unwrap_or("");
                    if let Ok(ts) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                        if ts.with_timezone(&Utc) > Utc::now() {
                            let role = db_identities.get(&name).cloned().unwrap_or_else(|| "Unknown".to_string());
                            manifest.insert(
                                name.clone(),
                                json!({
                                    "status": "WAKE",
                                    "role": role,
                                    "session_id": lease["session_id"],
                                    "driver": lease["driver_id"],
                                    "last_seen": expires_at
                                }),
                            );
                            wake_count += 1;
                        }
                    }
                }
            }
        }

        // 2. Add offline agents from DB that don't have active leases
        for (name, role) in db_identities {
            if !manifest.contains_key(&name) {
                manifest.insert(name, json!({ "status": "DARK", "role": role }));
            }
        }

        let payload = json!({
            "manifest": manifest,
            "wake_count": wake_count,
            "timestamp": Utc::now().timestamp()
        })
        .to_string();

        let _: () = self
            .redis
            .pool
            .next()
            .publish("koad:telemetry:manifest", &payload)
            .await?;
        // Persistent key for status --json
        let _: () = self
            .redis
            .pool
            .next()
            .hset("koad:state", ("crew_manifest", payload))
            .await?;
        Ok(())
    }

    async fn curate_intelligence(&self) -> anyhow::Result<()> {
        let all_state: HashMap<String, String> = self
            .redis
            .pool
            .next()
            .hgetall("koad:state")
            .await
            .unwrap_or_default();

        let mut active_sessions = Vec::new();
        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                    let data = if let Some(inner) = raw_json.get("data") {
                        inner
                    } else {
                        &raw_json
                    };
                    if let Ok(sess) = serde_json::from_value::<koad_core::session::AgentSession>(data.clone()) {
                        if sess.status == "active" {
                            active_sessions.push(sess);
                        }
                    }
                }
            }
        }

        for sess in active_sessions {
            let context_key = format!("koad:context:session:{}", sess.session_id);
            let hot_context: HashMap<String, String> = self
                .redis
                .pool
                .next()
                .hgetall(&context_key)
                .await
                .unwrap_or_default();

            for (chunk_id, val) in hot_context {
                if let Ok(chunk) = serde_json::from_str::<koad_core::types::HotContextChunk>(&val) {
                    // Promote high-signal chunks to durable L3 FactCards
                    if chunk.significance_score >= 0.8 {
                        info!("Intelligence Curation: Promoting chunk {} from session {} to L3 Memory Bank.", chunk_id, sess.session_id);
                        
                        let fact = FactCard {
                            id: uuid::Uuid::new_v4(),
                            source_agent: sess.identity.name.clone(),
                            session_id: sess.session_id.clone(),
                            domain: chunk.tags.first().cloned().unwrap_or_else(|| "general".to_string()),
                            content: chunk.content.clone(),
                            confidence: chunk.significance_score,
                            tags: chunk.tags.clone(),
                            created_at: Utc::now(),
                            ttl_seconds: 0, // Permanent by default for promoted facts
                        };

                        if let Err(e) = self.storage.save_fact(fact).await {
                            error!("Intelligence Curation Error: Failed to save fact card: {}", e);
                        } else {
                            // Tag the chunk in Redis as promoted to prevent redundant processing
                            let mut updated_chunk = chunk;
                            updated_chunk.tags.push("promoted".to_string());
                            updated_chunk.significance_score = 0.5; // Lower score to prevent re-promotion
                            let payload = serde_json::to_string(&updated_chunk)?;
                            let _: () = self.redis.pool.next().hset(&context_key, (chunk_id, payload)).await?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn prune_orphaned_sessions(&self) -> anyhow::Result<()> {
        let all_state: HashMap<String, String> = self
            .redis
            .pool
            .next()
            .hgetall("koad:state")
            .await
            .unwrap_or_default();
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

        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                let sid = key.replace("koad:session:", "");
                
                // Only prune if the session has no active lease
                if !active_session_ids.contains(&sid) {
                    // Grace period: Check if the session object itself is old enough to prune
                    if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                        let created_at = raw_json["created_at"].as_i64().unwrap_or(0);
                        let now = Utc::now().timestamp();
                        
                        if now - created_at > 30 {
                            info!("Autonomic Sentinel: Pruning orphaned session key: {}", key);
                            let _: () = self
                                .redis
                                .pool
                                .next()
                                .hdel("koad:state", &key)
                                .await
                                .unwrap_or(());
                        }
                    } else {
                        // If we can't parse it, it's likely corrupt sludge; prune it.
                        let _: () = self.redis.pool.next().hdel("koad:state", &key).await.unwrap_or(());
                    }
                }
            }
        }
        Ok(())
    }

    async fn perform_cognitive_quicksave(&self) -> anyhow::Result<()> {
        info!("Autonomic Sentinel: Commencing periodic cognitive quicksave...");
        
        let all_state: HashMap<String, String> = self
            .redis
            .pool
            .next()
            .hgetall("koad:state")
            .await
            .unwrap_or_default();

        let mut active_sessions = Vec::new();
        for (key, val) in all_state {
            if key.starts_with("koad:session:") {
                if let Ok(raw_json) = serde_json::from_str::<serde_json::Value>(&val) {
                    let data = if let Some(inner) = raw_json.get("data") {
                        inner
                    } else {
                        &raw_json
                    };
                    if let Ok(sess) = serde_json::from_value::<koad_core::session::AgentSession>(data.clone()) {
                        if sess.status == "active" {
                            active_sessions.push(sess);
                        }
                    }
                }
            }
        }

        for sess in active_sessions {
            let context_key = format!("koad:session:{}:hot_context", sess.session_id);
            let hot_context: HashMap<String, String> = self
                .redis
                .pool
                .next()
                .hgetall(&context_key)
                .await
                .unwrap_or_default();

            if hot_context.is_empty() {
                continue;
            }

            let snapshot = json!({
                "session": sess,
                "hot_context": hot_context,
                "timestamp": Utc::now().timestamp()
            });

            if let Ok(json_str) = serde_json::to_string(&snapshot) {
                if let Err(e) = self.storage.save_context_snapshot(&sess.identity.name, &sess.session_id, json_str).await {
                    error!("Autonomic Sentinel: Failed to save context snapshot for {}: {}", sess.identity.name, e);
                } else {
                    info!("Autonomic Sentinel: Periodic quicksave complete for {}.", sess.identity.name);
                }
            }
        }

        Ok(())
    }

    async fn _restart_gateway(&self) -> anyhow::Result<()> {
        let addr = std::env::var("GATEWAY_ADDR")
            .unwrap_or_else(|_| koad_core::constants::DEFAULT_GATEWAY_ADDR.to_string());
        let home =
            std::env::var("KOAD_HOME").context("KOAD_HOME not set. Cannot restart gateway.")?;
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

