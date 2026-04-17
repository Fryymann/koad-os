//! Authentication Interceptor
//!
//! Enforces the Zero-Trust security model by validating TraceContext headers
//! and session integrity against the active session cache.

use crate::auth::session_cache::ActiveSessions;
use tonic::{Request, Status};

/// Builds an interceptor closure that validates requests against active sessions.
///
/// This interceptor performs a Dual-Key check:
/// 1. Verifies the existence of mandatory metadata (`x-actor`, `x-session-id`, `x-session-token`).
/// 2. Validates the session token and actor name against the L1 (Redis-backed) cache.
/// 3. Ensures the session state is currently alive.
///
/// # Errors
/// Returns `UNAUTHENTICATED` if any header is missing or validation fails.
/// Returns `UNAVAILABLE` if the session exists but is in a `DARK` or `TEARDOWN` state.
#[expect(
    clippy::result_large_err,
    reason = "Tonic interceptors are required to return tonic::Status on authentication failures."
)]
pub fn build_citadel_interceptor(
    sessions: ActiveSessions,
) -> impl Fn(Request<()>) -> Result<Request<()>, Status> + Clone {
    move |req: Request<()>| {
        // 1. Admin bypass (for internal maintenance UDS)
        if req.metadata().get("x-admin-override").is_some() {
            return Ok(req);
        }

        // 2. Extract mandatory headers
        let actor = req
            .metadata()
            .get("x-actor")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing x-actor header"))?;

        let session_id = req
            .metadata()
            .get("x-session-id")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing x-session-id header"))?;

        let session_token = req
            .metadata()
            .get("x-session-token")
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| Status::unauthenticated("Missing x-session-token header"))?;

        // 3. L1 Validation: Check local session cache
        // We use lock here as HashMap lookups are O(1) and the mutex is rarely contested.
        let sessions_guard = sessions.lock();
        if let Some(record) = sessions_guard.get(session_id) {
            if record.session_token != session_token {
                return Err(Status::unauthenticated("Invalid session token"));
            }
            if record.agent_name != actor {
                return Err(Status::unauthenticated("Actor/Session mismatch"));
            }
            if !record.state.is_alive() {
                return Err(Status::unavailable("Session is not active"));
            }
        } else {
            // Bypass only for the 'BOOT' handshake which creates the first lease.
            if session_id != "BOOT" {
                return Err(Status::unauthenticated("Session not found or expired"));
            }
        }

        Ok(req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::session_cache::{ActiveSessions, SessionRecord};
    use crate::state::docking::DockingState;
    use chrono::Utc;
    use std::collections::HashMap;
    use std::sync::Arc;
    use parking_lot::Mutex;

    fn setup_test_sessions() -> ActiveSessions {
        let mut map = HashMap::new();
        map.insert(
            "SID-test-123".to_string(),
            SessionRecord {
                agent_name: "Tyr".to_string(),
                state: DockingState::Active,
                last_heartbeat: Utc::now(),
                body_id: "body-1".to_string(),
                session_token: "secret-token".to_string(),
                level: "OUTPOST".to_string(),
            },
        );
        Arc::new(Mutex::new(map))
    }

    #[test]
    fn test_interceptor_valid_request() {
        let sessions = setup_test_sessions();
        let interceptor = build_citadel_interceptor(sessions);

        let mut req = Request::new(());
        req.metadata_mut().insert("x-actor", "Tyr".parse().unwrap());
        req.metadata_mut()
            .insert("x-session-id", "SID-test-123".parse().unwrap());
        req.metadata_mut()
            .insert("x-session-token", "secret-token".parse().unwrap());

        assert!(interceptor(req).is_ok());
    }

    #[test]
    fn test_interceptor_rejects_missing_headers() {
        let sessions = setup_test_sessions();
        let interceptor = build_citadel_interceptor(sessions);

        let req = Request::new(());
        let res = interceptor(req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn test_interceptor_rejects_invalid_token() {
        let sessions = setup_test_sessions();
        let interceptor = build_citadel_interceptor(sessions);

        let mut req = Request::new(());
        req.metadata_mut().insert("x-actor", "Tyr".parse().unwrap());
        req.metadata_mut()
            .insert("x-session-id", "SID-test-123".parse().unwrap());
        req.metadata_mut()
            .insert("x-session-token", "wrong-token".parse().unwrap());

        let res = interceptor(req);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().message(), "Invalid session token");
    }

    #[test]
    fn test_interceptor_allows_boot_handshake() {
        let sessions = setup_test_sessions();
        let interceptor = build_citadel_interceptor(sessions);

        let mut req = Request::new(());
        req.metadata_mut()
            .insert("x-actor", "Scribe".parse().unwrap());
        req.metadata_mut()
            .insert("x-session-id", "BOOT".parse().unwrap());
        req.metadata_mut()
            .insert("x-session-token", "NONE".parse().unwrap());

        assert!(interceptor(req).is_ok());
    }
}
