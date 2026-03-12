# ASM & Spine Reliability Review — KoadConfig Integration

**Date:** 2026-03-10

**Reviewer:** Tyr

**Authority:** Dood Override

## Executive Summary

This review analyzes the core session management and identity tethering systems of KoadOS, focusing on the Agent Session Manager (ASM), the Spine's gRPC layer, and the newly integrated `KoadConfig` system. The current architecture relies heavily on hardcoded intervals (e.g., 90s leases, 30s prune cycles, 45s deadman switches) and convention-based Redis keying, which creates fragility in multi-agent or high-latency environments.

The top recommendation is to transition from hardcoded constants to a **Config-Driven Session Policy**. By leveraging the new TOML-based Registry, we can eliminate "Consciousness Collisions" through stricter identity binding and per-agent heartbeat tolerances. Furthermore, moving the Spine's autonomic recovery parameters into `KoadConfig` will allow for more resilient self-healing without requiring code rebuilds.

## 1. Architecture Audit

### 1.1 Boot Sequence
- **Resolution:** Agent name is currently resolved in `koad-cli/src/handlers/boot.rs` via `KoadLegacyConfig` (loading `config/`). My recent changes began transitioning this to the `KoadConfig` Registry (`crates/koad-core/src/config.rs`).
- **Session Binding:** `KOAD_SESSION_ID` is generated as a UUID v4 in `crates/koad-spine/src/rpc/mod.rs` [L303] within `initialize_session`. It is bound to the shell via environment variables printed at the end of the boot sequence [L256 in `boot.rs`].
- **Order of Operations:** 
    1. CLI loads local config (Legacy + new Registry).
    2. CLI checks for stale `KOAD_SESSION_ID` in environment [L24 in `boot.rs`].
    3. CLI checks for existing sovereign lease in Redis [L88 in `boot.rs`].
    4. Spine `initialize_session` is called.
    5. Spine acquires a Redis lease (90s) via `KAILeaseManager` [L310 in `rpc/mod.rs`].
    6. ASM registers the session and broadcasts a `SESSION_UPDATE` [L348 in `rpc/mod.rs`].
- **Race Conditions:** If two agents boot concurrently, the lease check in `KAILeaseManager::acquire_lease` [L47 in `identity.rs`] uses a simple `hget` then `hset` pattern, which is not atomic. A high-concurrency race could allow two sessions to claim the same agent lease briefly before ASM prunes one.

### 1.2 Body/Ghost Tethering
- **Mechanism:** One Body, One Ghost is enforced in two places:
    1. **Pre-emptive Pruning:** `prune_body_ghosts` in `crates/koad-spine/src/engine/asm.rs` [L290] kills existing sessions for the same agent/driver/env combo during initialization.
    2. **Lease Guardrail:** `KAILeaseManager` prevents a different session from acquiring a lease if one is active [L52 in `identity.rs`].
- **Propagation:** `KOAD_SESSION_ID` is propagated via environment variables. Subagents and hooks must manually inherit or be passed this ID.
- **Failure Modes:** If a terminal crashes, the session remains "active" in Redis until the ASM reaper (`static_prune`) marks it as "dark" after 60s of silence [L116 in `koad-asm/src/main.rs`].

### 1.3 Agent Session Manager
- **Tracking:** ASM daemon (`koad-asm`) runs a `static_prune` loop every 30s [L52 in `main.rs`].
- **Heartbeats:** Tracked via `last_heartbeat` timestamp in the `AgentSession` struct.
- **Timeouts:** 
    - **Deadman (Tier 1):** 45s of silence triggers emergency save [L103 in `main.rs`].
    - **Dark State:** 60s of silence marks session as "dark" [L116 in `main.rs`].
    - **Purge:** 300s of silence removes the session key entirely [L111 in `main.rs`].

### 1.4 Spine Connectivity
- **Coordination:** The Spine `Kernel` spawns the `koad-asm` process [L73 in `kernel.rs`]. The `Watchdog` (in `koad-watchdog`) monitors the Spine via gRPC heartbeats and re-spawns it if it fails [L35 in `watchdog/src/main.rs`].
- **State Consistency:** Consistency relies on Redis `koad:state`. When the Spine restarts, it hydrates its local ASM cache from Redis [L84 in `asm.rs`]. In-flight sessions persist as long as Redis state is intact.

## 2. Failure Analysis

### 2.1 Cognitive Mismatch Scenarios
- **Leaked Context:** If `KAILeaseManager::heartbeat` [L118 in `identity.rs`] fails to correctly map a session to an identity (due to the current O(n) scan approach), a session might "lose" its lease but remain active in the CLI, causing memory hydration to pull from the wrong context.
- **Concurrent Boot:** Two agents booting with the same name simultaneously can bypass the non-atomic lease check, resulting in a "split-brain" where both think they have the lease until the next ASM prune cycle.

### 2.2 Session Integrity Failures
- **Premature Purge:** If the `koad-asm` reaper runs during a brief network/Redis blip, it might mark a session as "dark" [L116 in `main.rs`] even if the agent is still active, causing the next tool call to fail authentication.
- **Orphaned State:** If an agent process is killed (`kill -9`), the Redis session key persists for 5 minutes, potentially blocking new boots if `--force` isn't used.

### 2.3 Recovery Gaps
- **ASM Crash:** If `koad-asm` crashes, heartbeats aren't monitored, and stale sessions are never purged. While `Watchdog` monitors the Spine, it does not currently monitor the `koad-asm` child process independently (it only checks the Spine's gRPC).
- **Sentinel Lag:** Sentinel (diagnostics) hydration can lag behind the actual Redis state, leading to "Condition Green" reports even if the ASM is stalled.

## 3. KoadConfig Integration Recommendations

### 3.1 Session Policy
- **TOML Migration:** Move `deadman_timeout`, `dark_timeout`, and `purge_timeout` into `config/kernel.toml` under a `[sessions]` block.
- **Per-Agent Policies:** Allow `identities/<agent>.toml` to override these values (e.g., Tyr needs 120s `deadman` during complex builds).

### 3.2 Identity Binding
- **Canonical Source:** Use the `KoadConfig` Registry as the exclusive source of truth for identity validation. 
- **Validation:** Implement a `validate_identity` check in the Spine that compares the `InitializeSessionRequest` against the local TOML profile to prevent rogue/malformed agents from booting.

### 3.3 Cognitive Isolation
- **Namespacing:** Derive Redis namespaces (e.g., `koad:session:tyr:*`) from config to prevent key collisions in multi-user environments.
- **Path Verification:** ASM should verify that the `root_path` in `ProjectContext` matches the allowed paths in the `registry.toml` for that project.

### 3.4 Spine Resilience
- **Watchdog Hardening:** Update `Watchdog` to monitor both the Spine gRPC and the presence of the `koad-asm` process.
- **Recovery Strategy:** Add a `recovery_mode` to `kernel.toml` (options: `PERISTENT`, `VOLATILE`, `SAFE_BOOT`) to define if sessions should be recovered or purged after a Spine crash.

### 3.5 Migration Priority
1. **Critical:** Heartbeat/Prune intervals (Move from `koad-asm/main.rs` to `kernel.toml`).
2. **High:** Identity Resolution (Full migration from `config/` to `config/identities/`).
3. **Medium:** Watchdog Thresholds (Move from `watchdog/main.rs` to `kernel.toml`).

## 4. Implementation Roadmap
1. **Define [sessions] and [watchdog] schemas in `koad-core::config`** (Effort: Low, Risk: Low).
2. **Refactor `koad-asm` to use `KoadConfig` for prune intervals** (Effort: Medium, Risk: Medium).
3. **Implement Atomic Lease Acquisition using Redis Lua scripts** (Effort: Medium, Risk: High).
4. **Update `koad-watchdog` to monitor `koad-asm` child process** (Effort: Low, Risk: Low).

## 5. Open Questions for Dood
1. Should we enforce "Strict Ghosting" (reject any second boot) or "Forceful Takeover" (defaulting to the behavior of `--force`) for Sovereign agents?
2. Do we want to support `encrypted` config files for sensitive identity/credential profiles?
3. Should the ASM be moved into a dedicated thread *within* the Spine instead of a separate process to simplify process monitoring?
