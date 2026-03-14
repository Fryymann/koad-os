//! Citadel Session Service
//!
//! Handles the agent lifecycle (leases, sessions, heartbeats) and enforces
//! the "One Body, One Ghost" policy.

use crate::state::bay_store::BayStore;
use crate::state::docking::DockingState;
use crate::state::storage_bridge::CitadelStorageBridge;
use chrono::Utc;
use fred::interfaces::PubsubInterface;
use fred::prelude::*;
use koad_proto::citadel::v5::citadel_session_server::CitadelSession;
use koad_proto::citadel::v5::*;
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
use tracing::{info, warn};

/// Active session record held in memory by the broker.
struct SessionRecord {
    agent_name: String,
    state: DockingState,
    last_heartbeat: chrono::DateTime<Utc>,
    body_id: String,
    start_time: i64,
}

/// Service implementation for the `CitadelSession` gRPC interface.
#[derive(Clone)]
pub struct CitadelSessionService {
    storage: Arc<CitadelStorageBridge>,
    bay_store: Arc<BayStore>,
    sessions: Arc<Mutex<HashMap<String, SessionRecord>>>,
    lease_duration_secs: u64,
}

impl CitadelSessionService {
    /// Creates a new `CitadelSessionService`.
    pub fn new(
        storage: Arc<CitadelStorageBridge>,
        bay_store: Arc<BayStore>,
        lease_duration_secs: u64,
    ) -> Self {
        Self {
            storage,
            bay_store,
            sessions: Arc::new(Mutex::new(HashMap::new())),
            lease_duration_secs,
        }
    }

    /// Provides a handle to the active sessions map for monitoring or reaper tasks.
    pub fn sessions_handle(&self) -> Arc<Mutex<HashMap<String, SessionRecord>>> {
        self.sessions.clone()
    }

    /// The automated reaper task that transitions stale sessions to `DARK` or `TEARDOWN`.
    ///
    /// - `dark_timeout_secs`: Time since last heartbeat before marking session as offline.
    /// - `purge_timeout_secs`: Time since last heartbeat before force-closing the session.
    pub async fn reap(&self, dark_timeout_secs: u64, purge_timeout_secs: u64) {
        let now = Utc::now();
        let mut sessions = self.sessions.lock().await;
        let mut to_teardown = Vec::new();

        for (sid, record) in sessions.iter_mut() {
            let elapsed = (now - record.last_heartbeat).num_seconds() as u64;

            if elapsed > purge_timeout_secs && record.state != DockingState::Teardown {
                warn!(
                    "Reaper: Session '{}' ({}) exceeded purge timeout ({}s). Teardown.",
                    sid, record.agent_name, elapsed
                );
                record.state = DockingState::Teardown;
                to_teardown.push((sid.clone(), record.agent_name.clone()));
            } else if elapsed > dark_timeout_secs
                && record.state.is_alive()
                && record.state != DockingState::Dark
            {
                info!(
                    "Reaper: Session '{}' ({}) missed heartbeat ({}s). Marking DARK.",
                    sid, record.agent_name, elapsed
                );
                record.state = DockingState::Dark;

                // Log to bay
                let _ = self
                    .bay_store
                    .log_state_transition(
                        &record.agent_name,
                        sid,
                        "DARK",
                        Some(&format!("Heartbeat missed for {}s", elapsed)),
                    )
                    .await;
            }
        }

        // Clean up teardown sessions
        for (sid, agent_name) in to_teardown {
            if let Some(record) = sessions.remove(&sid) {
                // Drain to bay session history
                let _ = self
                    .bay_store
                    .record_session_end(&agent_name, &sid, record.start_time, None, "TEARDOWN")
                    .await;

                // Clean Redis
                let lease_key = format!("koad:kai:{}:lease", agent_name);
                let session_key = format!("koad:session:{}", sid);
                let _: Result<(), _> = self
                    .storage
                    .redis
                    .pool
                    .hdel("koad:state", vec![&lease_key, &session_key])
                    .await;
            }
        }
    }
}

#[tonic::async_trait]
impl CitadelSession for CitadelSessionService {
    /// Grants a new session lease to an agent, enforcing "One Body, One Ghost".
    ///
    /// # Errors
    /// Returns `ALREADY_EXISTS` if the agent is already active in another session,
    /// unless `force` is true.
    async fn create_lease(
        &self,
        request: Request<LeaseRequest>,
    ) -> Result<Response<LeaseResponse>, Status> {
        let req = request.into_inner();
        let agent_name = &req.agent_name;
        let force = req.force;
        let body_id = if req.body_id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            req.body_id.clone()
        };

        info!(
            "CreateLease: Agent '{}' requesting lease (force={})",
            agent_name, force
        );

        // --- 1. One Body One Ghost enforcement ---
        let lease_key = format!("koad:kai:{}:lease", agent_name);
        let existing: Option<String> = self
            .storage
            .redis
            .pool
            .hget("koad:state", &lease_key)
            .await
            .map_err(|e| Status::internal(format!("Redis error: {}", e)))?;

        if let Some(lease_data) = existing {
            if let Ok(lease) = serde_json::from_str::<serde_json::Value>(&lease_data) {
                let is_live = lease["expires_at"]
                    .as_str()
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                    .map(|exp| exp.with_timezone(&Utc) > Utc::now())
                    .unwrap_or(false);

                if is_live && !force {
                    let existing_sid = lease["session_id"].as_str().unwrap_or("unknown");
                    let existing_body = lease["body_id"].as_str().unwrap_or("unknown");
                    return Err(Status::already_exists(format!(
                        "SOVEREIGN_OCCUPIED: {} is active in Body {} (Session: {}). Use force=true to take over.",
                        agent_name, existing_body, existing_sid
                    )));
                }

                // Force takeover: clean up existing session
                if is_live && force {
                    let old_sid = lease["session_id"].as_str().unwrap_or("");
                    info!(
                        "CreateLease: Force takeover of existing session '{}' for '{}'",
                        old_sid, agent_name
                    );
                    let old_session_key = format!("koad:session:{}", old_sid);
                    let _: Result<(), _> = self
                        .storage
                        .redis
                        .pool
                        .hdel("koad:state", vec![&lease_key, &old_session_key])
                        .await;

                    let mut sessions = self.sessions.lock().await;
                    sessions.remove(old_sid);
                }
            }
        }

        // --- 2. Generate session ---
        let session_id = format!(
            "SID-{}-{}",
            agent_name.to_lowercase(),
            &uuid::Uuid::new_v4().to_string()[..8]
        );
        let token = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();
        let expires_at = now + chrono::Duration::seconds(self.lease_duration_secs as i64);

        // Write lease to Redis
        let lease_value = json!({
            "session_id": session_id,
            "body_id": body_id,
            "agent_name": agent_name,
            "token": token,
            "created_at": now.to_rfc3339(),
            "expires_at": expires_at.to_rfc3339(),
        });

        let lease_json = serde_json::to_string(&lease_value)
            .map_err(|e| Status::internal(format!("Serialization error: {}", e)))?;

        let _: () = self
            .storage
            .redis
            .pool
            .hset("koad:state", (&lease_key, &lease_json))
            .await
            .map_err(|e| Status::internal(format!("Redis lease write failed: {}", e)))?;

        // Write session record
        let session_value = json!({
            "session_id": session_id,
            "agent_name": agent_name,
            "body_id": body_id,
            "state": "ACTIVE",
            "last_heartbeat": now.to_rfc3339(),
        });

        let session_json = serde_json::to_string(&session_value)
            .map_err(|e| Status::internal(format!("Serialization error: {}", e)))?;

        let session_key = format!("koad:session:{}", session_id);
        let _: () = self
            .storage
            .redis
            .pool
            .hset("koad:state", (&session_key, &session_json))
            .await
            .map_err(|e| Status::internal(format!("Redis session write failed: {}", e)))?;

        // In-memory tracking
        let record = SessionRecord {
            agent_name: agent_name.clone(),
            state: DockingState::Active,
            last_heartbeat: now,
            body_id: body_id.clone(),
            start_time: now.timestamp(),
        };
        self.sessions
            .lock()
            .await
            .insert(session_id.clone(), record);

        // Log state transitions to bay
        let _ = self
            .bay_store
            .log_state_transition(agent_name, &session_id, "DOCKING", None)
            .await;
        let _ = self
            .bay_store
            .log_state_transition(agent_name, &session_id, "HYDRATING", None)
            .await;
        let _ = self
            .bay_store
            .log_state_transition(agent_name, &session_id, "ACTIVE", None)
            .await;

        // Ensure bay is provisioned
        let _ = self.bay_store.provision(agent_name).await;

        // Publish session event
        {
            use fred::interfaces::PubsubInterface;
            let _: Result<(), _> = self
                .storage
                .redis
                .pool
                .publish(
                    "koad:sessions",
                    json!({"type": "SESSION_CREATED", "session_id": &session_id, "agent": agent_name}).to_string(),
                )
                .await;
        }

        info!(
            "CreateLease: Session '{}' created for '{}'",
            session_id, agent_name
        );

        let context = req.context.map(|c| TraceContext {
            trace_id: c.trace_id,
            origin: "Citadel".to_string(),
            actor: "citadel".to_string(),
            timestamp: Some(prost_types::Timestamp {
                seconds: now.timestamp(),
                nanos: 0,
            }),
        });

        Ok(Response::new(LeaseResponse {
            session_id,
            token,
            context,
        }))
    }

    /// Heartbeat signal to keep a session lease active.
    async fn heartbeat(
        &self,
        request: Request<HeartbeatRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let session_id = &req.session_id;
        let now = Utc::now();

        let mut sessions = self.sessions.lock().await;
        let record = sessions
            .get_mut(session_id)
            .ok_or_else(|| Status::not_found(format!("Session '{}' not found", session_id)))?;

        record.last_heartbeat = now;

        // Recover from DARK if needed
        if record.state == DockingState::Dark {
            record.state = DockingState::Active;
            info!(
                "Heartbeat: Session '{}' recovered from DARK -> ACTIVE",
                session_id
            );
            let _ = self
                .bay_store
                .log_state_transition(
                    &record.agent_name,
                    session_id,
                    "ACTIVE",
                    Some("Recovered from DARK"),
                )
                .await;
        }

        // Refresh lease TTL in Redis
        let lease_key = format!("koad:kai:{}:lease", record.agent_name);
        if let Ok(Some(lease_data)) = self
            .storage
            .redis
            .pool
            .hget::<Option<String>, _, _>("koad:state", &lease_key)
            .await
        {
            if let Ok(mut lease) = serde_json::from_str::<serde_json::Value>(&lease_data) {
                let new_expiry = now + chrono::Duration::seconds(self.lease_duration_secs as i64);
                lease["expires_at"] = json!(new_expiry.to_rfc3339());

                if let Ok(json) = serde_json::to_string(&lease) {
                    let _: Result<(), _> = self
                        .storage
                        .redis
                        .pool
                        .hset("koad:state", (&lease_key, &json))
                        .await;
                }
            }
        }

        Ok(Response::new(StatusResponse {
            success: true,
            message: "Heartbeat acknowledged".to_string(),
            context: None,
        }))
    }

    /// Explicitly closes a session and releases the ghost lease.
    async fn close_session(
        &self,
        request: Request<CloseRequest>,
    ) -> Result<Response<StatusResponse>, Status> {
        let req = request.into_inner();
        let session_id = &req.session_id;

        let mut sessions = self.sessions.lock().await;
        let record = sessions
            .remove(session_id)
            .ok_or_else(|| Status::not_found(format!("Session '{}' not found", session_id)))?;

        info!(
            "CloseSession: Tearing down '{}' for '{}'",
            session_id, record.agent_name
        );

        // Log teardown
        let _ = self
            .bay_store
            .log_state_transition(&record.agent_name, session_id, "TEARDOWN", None)
            .await;

        // Record session end
        let _ = self
            .bay_store
            .record_session_end(
                &record.agent_name,
                session_id,
                record.start_time,
                if req.summary_path.is_empty() {
                    None
                } else {
                    Some(&req.summary_path)
                },
                "TEARDOWN",
            )
            .await;

        // Clean Redis
        let lease_key = format!("koad:kai:{}:lease", record.agent_name);
        let session_key = format!("koad:session:{}", session_id);
        let _: Result<(), _> = self
            .storage
            .redis
            .pool
            .hdel("koad:state", vec![&lease_key, &session_key])
            .await;

        // Publish event
        {
            use fred::interfaces::PubsubInterface;
            let _: Result<(), _> = self
                .storage
                .redis
                .pool
                .publish(
                    "koad:sessions",
                    json!({"type": "SESSION_CLOSED", "session_id": session_id, "agent": &record.agent_name}).to_string(),
                )
                .await;
        }

        Ok(Response::new(StatusResponse {
            success: true,
            message: format!("Session '{}' closed. EndOfWatch.", session_id),
            context: None,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Mock storage/bay_store and Tier 2 tests here in next turn
}
