# Incident Report: CLI Authentication & Integration Failure
**Date:** 2026-04-25
**Agent:** tyr (Principal Engineer)

## 🚨 Symptom
After restoring the Citadel Kernel (see previous report), the `koad` CLI continued to report services as unreachable or returned `Unauthenticated` errors:
- `Error: status: Unauthenticated, message: "Missing x-actor header"`
- Heartbeat and configuration commands failed despite active gRPC listeners.

## 🔍 Root Cause Analysis
1. **Security Interceptor Enforcement:** The Citadel Kernel's gRPC interceptor (`koad-citadel/src/auth/interceptor.rs`) enforces a Zero-Trust model requiring three mandatory headers: `x-actor` (agent name), `x-session-id`, and `x-session-token`.
2. **CLI Header Gap:** The CLI utility `authenticated_request` in `koad-cli/src/utils/mod.rs` was only providing `x-session-id`. It lacked the logic to hydrate the actor name and session token from the environment.
3. **Broken Internal Imports:** The `koad` crate (CLI) had several "stale" references to an internal `mod db`, whereas the actual `KoadDB` implementation has been centralized in `koad_core::db`. This caused significant compilation failures during the restoration attempt.

## 🛠️ Resolution

### 1. gRPC Header Hydration
Updated `crates/koad-cli/src/utils/mod.rs` to fully hydrate the authentication context from environment variables exported during `koad-agent boot`:

```rust
pub fn authenticated_request<T>(payload: T) -> tonic::Request<T> {
    let mut req = tonic::Request::new(payload);
    // x-actor: Required for identity-aware signal routing
    if let Ok(actor) = env::var("KOAD_AGENT_NAME") {
        req.metadata_mut().insert("x-actor", actor.parse().unwrap());
    }
    // x-session-id & x-session-token: Required for session integrity
    if let Ok(sid) = env::var("KOAD_SESSION_ID") {
        req.metadata_mut().insert("x-session-id", sid.parse().unwrap());
    }
    if let Ok(token) = env::var("KOAD_SESSION_TOKEN") {
        req.metadata_mut().insert("x-session-token", token.parse().unwrap());
    }
    req
}
```

### 2. CLI Refactoring (Import Consolidation)
- **Database Mapping:** Re-mapped all instances of `use crate::db::KoadDB` to `use koad_core::db::KoadDB` across 10+ files using `sed` batch processing.
- **Module Cleanup:** Removed the incorrect `mod db` declaration in `main.rs` and commented out the missing `abc` module to restore a clean build state.
- **Error Utility:** Created `crates/koad-cli/src/utils/errors.rs` to provide the `KoadGrpcError` mappings required by the CLI handlers.

## 📈 Long-term Recommendation
- **Unified Auth Trait:** Create a shared `KoadAuth` trait in `koad-core` that both Kernel and CLI can use to ensure header keys (`x-actor`, etc.) are never out of sync.
- **Dependency Audit:** The `koad-cli` crate should strictly favor `koad-core` for shared infrastructure (DB, Logging, Config) rather than maintaining internal "ghost" modules.
