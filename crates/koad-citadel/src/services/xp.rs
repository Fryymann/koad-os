//! # XP and Skill Management Service
//!
//! Provides gRPC interfaces for querying agent XP status and awarding points.
//! This service is fully config-driven, utilizing `KoadConfig` for leveling 
//! curves and skill definitions.

use anyhow::Result;
use koad_core::config::KoadConfig;
use koad_proto::citadel::v5::xp_service_server::XpService;
use koad_proto::citadel::v5::{StatusResponse, XpAwardRequest, XpStatusRequest, XpStatusResponse};
use rusqlite::params;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{info, warn};

/// Service implementation for XP and Skill logic.
#[derive(Clone)]
pub struct CitadelXpService {
    db: Arc<Mutex<rusqlite::Connection>>,
    config: KoadConfig,
}

impl CitadelXpService {
    /// Creates a new `CitadelXpService`.
    pub async fn new(db: Arc<Mutex<rusqlite::Connection>>, config: KoadConfig) -> Result<Self> {
        // Initialize the XP ledger table and seed from config if empty
        {
            let mut conn = db.lock().await;
            conn.execute(
                "CREATE TABLE IF NOT EXISTS xp_ledger (
                    id INTEGER PRIMARY KEY,
                    agent_name TEXT NOT NULL,
                    amount INTEGER NOT NULL,
                    reason TEXT NOT NULL,
                    source INTEGER NOT NULL,
                    source_id TEXT,
                    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP
                )",
                [],
            )?;

            // Check if table is empty
            let count: i64 = conn.query_row("SELECT COUNT(*) FROM xp_ledger", [], |r| r.get(0))?;
            if count == 0 {
                info!("XP Ledger is empty — seeding from KoadConfig identities.");
                let tx = conn.transaction()?;
                for (key, id) in &config.identities {
                    if id.xp > 0 {
                        info!(agent = %id.name, xp = %id.xp, "Seeding initial XP balance");
                        tx.execute(
                            "INSERT INTO xp_ledger (agent_name, amount, reason, source, source_id) 
                             VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![
                                id.name,
                                id.xp as i32,
                                "Opening balance (migrated from identity TOML)",
                                0, // Source: System
                                format!("init:{}", key)
                            ],
                        )?;
                    }
                }
                tx.commit()?;
            }
        }
        Ok(Self { db, config })
    }

    fn calculate_level(&self, total_xp: i32) -> (i32, String, i32) {
        let base = self.config.xp.base_xp_per_level as f32;
        let exp = self.config.xp.level_curve_exponent;
        
        // Level = (XP / Base)^(1/Exp)
        let level = (total_xp as f32 / base).powf(1.0 / exp).floor() as i32;
        
        let tier_name = match level {
            0..=2 => "Initiate",
            3..=5 => "Sentinel",
            6..=9 => "Architect",
            _ => "Grandmaster",
        };

        // Next level XP = Base * (level + 1)^Exp
        let next_level_xp = (base * ((level + 1) as f32).powf(exp)) as i32;

        (level, tier_name.to_string(), next_level_xp)
    }
}

#[tonic::async_trait]
impl XpService for CitadelXpService {
    /// Retrieves the current XP status for an agent.
    async fn get_status(
        &self,
        request: Request<XpStatusRequest>,
    ) -> Result<Response<XpStatusResponse>, Status> {
        let req = request.into_inner();
        let conn = self.db.lock().await;

        let total_xp: i32 = conn
            .query_row(
                "SELECT SUM(amount) FROM xp_ledger WHERE agent_name = ?1",
                params![req.agent_name],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let (level, tier_name, next_level_xp) = self.calculate_level(total_xp);

        Ok(Response::new(XpStatusResponse {
            agent_name: req.agent_name,
            total_xp,
            level,
            tier_name,
            next_level_xp,
            context: req.context,
        }))
    }

    /// Awards XP to an agent, validating against config and rank.
    async fn award_xp(
        &self,
        request: Request<XpAwardRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        
        // --- 1. Validation ---
        if req.amount > self.config.xp.grant_cap_per_turn {
            warn!(agent = %req.agent_name, amount = %req.amount, "XP Award exceeds turn cap");
            return Err(Status::invalid_argument("Award exceeds per-turn safety cap."));
        }

        // --- 2. Persistence ---
        {
            let conn = self.db.lock().await;
            conn.execute(
                "INSERT INTO xp_ledger (agent_name, amount, reason, source, source_id) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    req.agent_name,
                    req.amount,
                    req.reason,
                    req.source as i32,
                    req.source_id
                ],
            ).map_err(|e| Status::internal(e.to_string()))?;
        }

        info!(agent = %req.agent_name, amount = %req.amount, reason = %req.reason, "XP Awarded");

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Successfully awarded {} XP to {}.", req.amount, req.agent_name),
            context: req.context,
        }))
    }
}
