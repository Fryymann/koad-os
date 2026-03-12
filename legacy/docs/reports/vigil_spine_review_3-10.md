# KOAD SPINE — VIGIL ASSESSMENT REPORT
**Date:** 2026-03-10
**Assessed by:** Vigil (Claude Code)
**File:** koad-os/reports/vigil_spin_review_3-10.md

---

## Scores
| Domain | Rating |
|---|---|
| Stability | :yellow_circle: |
| Role Fitness | :yellow_circle: |
| Integrity | :yellow_circle: |
| Sustainability | :yellow_circle: |

---

## Phase 1 — Orientation

### Entry Points & Runtime Surface

The Spine (`kspine`) is a long-running async Rust daemon. Single entry point at `crates/koad-spine/src/main.rs`. Runtime: tokio `#[tokio::main]`.

**Startup sequence:**
1. `KoadConfig::load()` — loads TOML config hierarchy
2. `PidGuard::new()` — acquires PID lock file (`kspine.pid`)
3. `init_logging()` — structured tracing with file appender
4. `KernelBuilder::start()`:
   - `Engine::new()` — initializes Redis client, SQLite connection (WAL mode), seeds config to Redis, creates all subsystem managers
   - Spawns standalone ASM daemon (`bin/koad-asm`) as child process
   - Spawns 5 background task loops: StorageBridge drain, ASM session monitor, ShipDiagnostics health monitor, diagnostics watchdog (stall detector), DirectiveRouter
   - Starts gRPC servers: TCP (`:50051`) + UDS (`kspine.sock`)
5. Awaits `ctrl_c` signal
6. Graceful shutdown: broadcast via `watch::channel`, kill ASM child, 500ms settle

### Subsystem Interactions

| Subsystem | Interaction Type | Notes |
|---|---|---|
| Redis (fred) | Unix socket (`koad.sock`) | Hot state, pub/sub, leases, sessions |
| SQLite (rusqlite) | File (`koad.db`) | Cold state, intelligence bank, context snapshots |
| ASM daemon (`koad-asm`) | Child process + Redis pub/sub | Session pruning, ghost enforcement, deadman switch |
| Watchdog (`koad-watchdog`) | gRPC client → Spine | Liveness check via `Heartbeat` RPC |
| CLI (`koad`) | gRPC client → Spine | `InitializeSession`, `Heartbeat`, `TerminateSession`, all RPCs |
| ShipDiagnostics | In-process | Health checks, autonomic recovery, intelligence curation |
| DirectiveRouter | In-process + Redis pub/sub | Command dispatch via `koad:commands` channel |

### Fragility Points

1. **ASM spawn timing assumption:** 500ms sleep after spawn, then `try_wait()` to check if it crashed. If ASM crashes after this window, Kernel continues unaware until ShipDiagnostics detects the missing process (every 5s).
2. **No SIGTERM handling:** Only SIGINT (`ctrl_c`) triggers shutdown. A `kill <pid>` (SIGTERM) would not trigger graceful shutdown — no drain, no ASM cleanup.
3. **Hardcoded ASM binary path:** `home_dir.join("bin/koad-asm")` — no config override.

### Inline Documentation

- `CLAUDE.md`: Comprehensive architecture docs, config system, conventions. Well-maintained.
- `AGENTS.md`: Agent onboarding manual (v5.0.0). Thorough.
- `VIGIL_AUDIT.md`: Existing security audit. Detailed findings referenced throughout this review.
- Inline code comments: Sparse but present at key decision points. Lua scripts are documented.

---

## Phase 2 — Stability Assessment

### Lifecycle Definition

**Well-defined.** Start → Ready → Running → Shutdown follows a clear path:
- `watch::channel<bool>` broadcast coordinates shutdown across all spawned tasks
- `PidGuard` prevents duplicate instances
- gRPC servers use `serve_with_shutdown` for clean stop

### Unhandled Crash Paths

| Path | Impact | Severity |
|---|---|---|
| Redis goes down mid-operation | Most operations fail silently (errors logged, not fatal). Diagnostics detects via PING. Self-healing hydration triggers on `initialized` flag loss. | Medium |
| SQLite corruption | `spawn_blocking` tasks will error. No circuit breaker — continues attempting operations. | Medium |
| ASM child crashes after 500ms window | Spine continues without session pruning until diagnostics detects (~5s). Recovery: autonomic restart via `nohup`. | Low |
| SIGTERM received | Unhandled. Process exits without drain, without ASM cleanup. Up to 30s of state could be lost. | **High** |
| `Engine::new()` panics on `expect()` calls | No `expect()` in Engine::new — uses `?` propagation. Good. | N/A |

### Partial Failure Resilience

- **Redis unavailable at startup:** Engine::new fails → process exits cleanly. Correct.
- **ASM binary missing:** Warning logged, continues without ASM features. Correct degradation.
- **Gateway down:** Autonomic recovery attempts restart. Correct.
- **ASM process missing:** Diagnostics detects via `sysinfo`, attempts restart via `nohup`. Correct.
- **Redis flush (state loss):** `initialized` flag missing → `check_registry_integrity` triggers `hydrate_all()` from SQLite. Self-healing. Good.

### Race Conditions & Timing

1. **`initialized` flag TOCTOU:** Between checking `hexists("koad:state", "initialized")` and completing `hydrate_all()`, another process could set the flag. Multiple simultaneous hydrations could produce inconsistent state. Mitigated by single-Spine design but not structurally prevented.
2. **Concurrent `drain_all()`:** The drain loop and the `DrainAll` RPC can race. Both read `HGETALL koad:state` and batch-write to SQLite. SQLite WAL mode handles concurrent writes, but a drain in progress could see partial state during another drain's write. Low risk due to idempotent `INSERT OR REPLACE`.
3. **ASM prune vs Spine session cache:** The standalone ASM daemon prunes Redis keys, then broadcasts `SESSION_PRUNED`. The Spine's in-process ASM watcher listens and removes from local cache. If the broadcast is missed, the local cache and Redis diverge. Cache staleness is eventual — next `hydrate_from_db()` reconciles.

### Signal Handling

- **SIGINT:** Handled. Triggers graceful shutdown.
- **SIGTERM:** Not handled. This is a gap — systemd sends SIGTERM for service stops.
- **SIGHUP:** Not handled. No config reload mechanism.

### Logging

- Structured tracing via `tracing` crate: `info!`, `warn!`, `error!`, `debug!`.
- File-based rotation via `tracing-appender`.
- **Inconsistency:** `kernel.rs`, `router.rs`, and `diagnostics.rs` use `println!`/`eprintln!` for many messages instead of `tracing`. This bypasses structured logging, log levels, and file appenders.
- Redis event stream (`koad:events:stream`) provides telemetry audit trail.
- Health registry written to `koad:state` for `koad status` inspection.

### Restart Behavior

- ShipDiagnostics has a self-healing restart loop for the health monitor (5s delay, stall detection via `AtomicI64` heartbeat timestamp, >30s gap triggers reset).
- Autonomic recovery spawns processes via `nohup` — no deduplication check. If recovery triggers repeatedly, multiple instances could be spawned. The PidGuard on Spine itself prevents this for kspine, but koad-asm and kgateway have no such guard in the recovery path.

**Rating: :yellow_circle:**

The Spine has a well-structured lifecycle with good self-healing capabilities. The core state model (Redis + SQLite with periodic drain) is resilient to crashes — at most 30 seconds of state loss. Self-healing hydration on Redis state loss is a strong feature.

However: missing SIGTERM handling is a significant gap for production use, the `println!`/`eprintln!` inconsistency undermines structured observability, and the lack of deduplication in autonomic recovery could produce ghost processes. These are all fixable without architectural changes.

---

## Phase 3 — Role Fitness

### Is Spine doing the right job?

The Spine's core identity is: **central gRPC kernel that manages agent sessions, state, and RPC dispatch.** This is appropriate and well-scoped for the load-bearing role it plays.

### Responsibilities that correctly belong to Spine

| Responsibility | Implementation | Assessment |
|---|---|---|
| gRPC service (TCP + UDS) | `rpc/mod.rs` via tonic | Correct |
| Identity/Lease management (Lua atomics) | `engine/identity.rs` | Correct — Spine is the serialization point |
| State bridge (Redis + SQLite) | `engine/storage_bridge.rs` | Correct |
| Hot config seeding | `engine/config.rs` | Correct |
| Command dispatch (DirectiveRouter) | `engine/router.rs` | Correct — Spine owns execution authorization |
| Signal management (A2A-S) | `engine/signal.rs` | Correct |
| Context caching | `engine/context_cache.rs` | Correct |
| Hydration management | `engine/hydration.rs` | Correct |
| Compliance management | `engine/kcm.rs` | Correct — lightweight governance dispatch |
| Skill discovery | `discovery/mod.rs` | Correct — lightweight manifest scanning |

### Responsibilities that should be elsewhere

| Responsibility | Current Owner | Correct Owner | Issue |
|---|---|---|---|
| **Autonomic service recovery** (restart gateway, restart ASM) | `ShipDiagnostics._restart_gateway()`, `_restart_asm()` | **koad-watchdog** | The Watchdog already exists for this purpose. Spine should not restart its own dependencies — this creates circular restart chains (Spine restarts ASM, Watchdog restarts Spine). |
| **Orphaned session pruning** | `ShipDiagnostics.prune_orphaned_sessions()` | **koad-asm** | ASM daemon already does `static_prune()` with per-agent policy resolution. Spine's pruning is a redundant second pass with different logic (300s hardcoded vs config-driven timeouts in ASM). |
| **Intelligence curation** (L2→L3 promotion) | `ShipDiagnostics.curate_intelligence()` | **Dedicated intelligence service or ASM** | This is cognitive work, not health monitoring. It reads all active sessions, scans all hot context chunks, and promotes high-signal ones to SQLite. This is conceptually separate from diagnosing system health. |
| **Cognitive quicksaves** | `ShipDiagnostics.perform_cognitive_quicksave()` | **koad-asm or StorageBridge** | Context snapshot persistence is a storage/session concern, not a diagnostics concern. |
| **Crew manifest assembly** | `ShipDiagnostics.update_crew_manifest()` | **koad-asm** | Manifest is a session/identity concern. ASM already tracks sessions. |

### Overlapping Responsibilities

| Function | Spine (in-process) | Standalone Daemon | Conflict |
|---|---|---|---|
| Session pruning | `diagnostics.rs:prune_orphaned_sessions()` — 300s hardcoded timeout, runs every 12 iterations (~1 min) | `koad-asm:static_prune()` — config-driven timeouts, per-agent policy | **Duplicate with different behavior.** ASM prune is more sophisticated (respects `session_policy`). Spine prune uses a flat 300s. If both run, they could race on `HDEL` operations. |
| Health checking | `diagnostics.rs:start_health_monitor()` — Redis ping, SQLite check, ghost detection, ASM process check | `koad-watchdog:main()` — Spine gRPC heartbeat, ASM process check | **Partial overlap.** Watchdog checks Spine liveness. Diagnostics checks everything else. Acceptable delineation but ASM process checking is duplicated. |
| ASM restart | `diagnostics.rs:_restart_asm()` | `koad-watchdog:reboot_spine()` — kills and restarts Spine (which spawns ASM) | **Conflicting recovery.** If Spine's diagnostics restarts ASM directly, and Watchdog also detects ASM down and kills/restarts Spine, both recoveries fire simultaneously. |
| Session monitoring | `asm.rs:start_session_monitor()` — subscribes to `koad:sessions`, maintains local HashMap cache | `koad-asm:run()` — subscribes to `koad:sessions`, logs events | **Acceptable overlap.** In-process ASM is a local cache; standalone ASM is the authoritative pruner. The subscription is shared but the purpose differs. |

### Scope Creep: ShipDiagnostics

`ShipDiagnostics` is the primary scope creep vector. It is a single struct with 12+ methods spanning:
- System health monitoring (Redis, SQLite, gateway, ASM process)
- Service recovery (restart gateway, restart ASM, purge ghost PID files)
- Session management (orphaned session pruning)
- Intelligence management (L2→L3 curation, cognitive quicksaves)
- Identity management (crew manifest)
- Telemetry publishing (system stats, service states)

This violates single-responsibility. The struct name says "diagnostics" but it performs recovery, curation, and identity operations.

### Single-Responsibility Proposal

```
ShipDiagnostics (keep)     → Health checks, telemetry publishing, stats only
koad-asm (delegate to)     → Session pruning, crew manifest, ghost enforcement
IntelligenceCurator (new)  → L2→L3 promotion, cognitive quicksaves
koad-watchdog (delegate to) → Service recovery (restart ASM, restart gateway)
```

**Rating: :yellow_circle:**

The Spine's core responsibilities (gRPC, leases, state, dispatch) are correctly scoped and well-implemented. The Spine is the right component to be the central kernel. However, ShipDiagnostics has accumulated significant scope creep, creating overlap with ASM and Watchdog. This isn't dangerous today but will create maintenance confusion and conflicting recovery behavior as the system grows. The delineation between in-process ASM (cache) and standalone ASM (authority) is correct in principle but the overlap in pruning logic is a concrete problem.

---

## Phase 4 — Integrity Check

### Security Concerns

The existing `VIGIL_AUDIT.md` provides a thorough security analysis. This section focuses on **new findings from this deeper code review** that were not covered in the prior audit.

#### New Finding I-1: `register_component` is an unconditional auth bypass

**File:** `crates/koad-spine/src/rpc/mod.rs:319-327`

```rust
async fn register_component(
    &self,
    _request: Request<RegisterComponentRequest>,
) -> Result<Response<RegisterComponentResponse>, Status> {
    Ok(Response::new(RegisterComponentResponse {
        session_id: uuid::Uuid::new_v4().to_string(),
        authorized: true,  // Always true, no check
    }))
}
```

Any caller receives a session ID and `authorized: true` regardless of identity. This is a stub that should either be removed or implemented with actual authorization.

#### New Finding I-2: `execute` RPC is a no-op stub

**File:** `crates/koad-spine/src/rpc/mod.rs:56-75`

The `Execute` RPC always returns `success: true` with a canned message. It does not execute anything. This is confusing — a caller would believe their command succeeded. If this is intentional (execute via `DispatchTask` instead), the `Execute` RPC should return `UNIMPLEMENTED`.

#### New Finding I-3: `dispatch_task` defaults to "admin" identity

**File:** `crates/koad-spine/src/rpc/mod.rs:99-103`

```rust
let identity = if req.identity.is_empty() {
    "admin"
} else {
    &req.identity
};
```

If the identity field is empty in the request, the task runs under the "admin" session ID. The DirectiveRouter then falls back to a "ghost" identity (Crew rank) when the "admin" session doesn't exist. This is a confusing fallback chain.

#### New Finding I-4: Hydration reads arbitrary files without path validation

**File:** `crates/koad-spine/src/engine/hydration.rs:38-48`

```rust
if let Some(ref path) = file_path {
    let path_buf = std::path::PathBuf::from(path);
    if path_buf.exists() && path_buf.is_file() {
        final_content = std::fs::read_to_string(&path_buf)?;
    }
}
```

The `file_path` from the gRPC request is used directly to read files from disk. No path validation, no sandbox check, no canonicalization. Any gRPC caller can read any file the Spine process has access to, including `/etc/shadow`, `~/.ssh/id_rsa`, or any sensitive file.

**Risk: High.** This is an arbitrary file read vulnerability via the `HydrateContext` RPC.

#### New Finding I-5: `GetFileSnippet` reads arbitrary files without path validation

**File:** `crates/koad-spine/src/engine/context_cache.rs:47-52`

```rust
let full_path = std::fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
if !full_path.exists() {
    anyhow::bail!("File not found: {:?}", full_path);
}
let full_content = std::fs::read_to_string(&full_path)?;
```

Same issue as I-4. The path is canonicalized (which resolves symlinks) but there is no check that the resolved path is within an allowed directory. Any file readable by the Spine process can be accessed via this RPC.

#### New Finding I-6: KCM uses `expect()` for KOAD_HOME

**File:** `crates/koad-spine/src/engine/kcm.rs:64,90,109`

```rust
let koad_home = std::env::var("KOAD_HOME").expect("KOAD_HOME not set");
```

Three `expect()` calls. If `KOAD_HOME` is unset, the Spine process panics. This should be `anyhow::bail!` or a fallback.

#### New Finding I-7: `SystemAction::Reboot` calls `process::exit(0)`

**File:** `crates/koad-spine/src/engine/router.rs:119`

```rust
SystemAction::Reboot => {
    println!("DirectiveRouter: System REBOOT initiated.");
    std::process::exit(0);
}
```

A `Reboot` intent via Redis pub/sub causes an immediate `process::exit(0)` — no graceful shutdown, no drain, no ASM cleanup. Any process that can publish to `koad:commands` can instantly kill the Spine without the shutdown sequence.

**Risk: High.** This bypasses the graceful shutdown path entirely.

#### New Finding I-8: Hydration duplicate check doesn't prevent re-insertion

**File:** `crates/koad-spine/src/engine/hydration.rs:68-78`

```rust
if self.redis.pool.hexists::<bool, _, _>(&context_key, &chunk_id).await? {
    info!("Hydration Canceled: Chunk {} already exists...", chunk_id, session_id);
    // NOTE: Falls through to persistence below — doesn't return early!
}
```

The duplicate check logs a message but does NOT return early. The chunk is re-inserted regardless. This is a logic bug.

### Secrets & Credentials

- **Appropriate:** Credentials are never stored in config files. Only env var names are stored in `access_keys`.
- **Appropriate:** The `access_keys` resolution is deferred to the CLI at runtime.
- **Concern:** The Sandbox bypass key name (`GITHUB_ADMIN_PAT`) is hardcoded in source. Documented in VIGIL_AUDIT.md.

### Dependency Risks

| Dependency | Version Status | Risk |
|---|---|---|
| `fred` (Redis) | Active, well-maintained | Low |
| `rusqlite` (bundled) | Stable, bundles SQLite | Low |
| `tonic` / `prost` | Standard gRPC stack, active | Low |
| `sysinfo` | Active, used for stats/process detection | Low |
| `walkdir` | Stable, minimal API | Low |
| `serde_yaml` | Active, YAML parsing | Low |
| `sha2` | Widely used crypto | Low |

No deprecated APIs detected. No pinning concerns.

### Contract Drift

| Proto Definition | Implementation | Drift |
|---|---|---|
| `Execute` RPC | Always returns success, no execution | **Stub — misleading** |
| `StreamTaskStatus` | Returns one initial message, stops | **Stub — incomplete** |
| `RegisterComponent` | Always returns `authorized: true` | **Stub — no authorization** |
| `Heartbeat` | Functional — updates lease and ASM | Correct |
| `InitializeSession` | Functional — full session lifecycle | Correct |
| `TerminateSession` | Functional — marks dark, releases lease | Correct |
| `DrainAll` | Functional — triggers state drain | Correct |
| `HydrateContext` | Functional but has file read vuln | Functional with bug |
| `CommitKnowledge` | Functional | Correct |
| `SendSignal` / `GetSignals` / `UpdateSignalStatus` | Functional | Correct |
| `GetFileSnippet` | Functional but has file read vuln | Functional with bug |
| `PostSystemEvent` | Functional | Correct |
| `GetSystemState` | Functional | Correct |
| `GetService` / `RegisterService` | Functional | Correct |
| `StreamSystemEvents` | Functional (Redis pub/sub relay) | Correct |

**Rating: :yellow_circle:**

The core identity and lease mechanisms are sound. The Lua scripts are correct for single-Redis deployment. The StorageBridge dual-path model is well-designed. However, this review uncovered 8 new findings not in VIGIL_AUDIT.md: two arbitrary file read paths (I-4, I-5), a process exit bypass (I-7), a hydration logic bug (I-8), three stub RPCs with misleading responses (I-1, I-2, I-3), and panic-on-missing-env (I-6). The arbitrary file reads and process exit bypass are the most actionable.

---

## Phase 5 — Sustainability Assessment

### Scalability Patterns

#### Current Scale
The system is designed for 1-10 concurrent agents on a single host. At this scale, the current architecture is adequate.

#### Patterns That Become Liabilities

| Pattern | Current Impact | At Scale (50+ agents) |
|---|---|---|
| **`HGETALL koad:state`** — called by drain_all, diagnostics (3+ places), ASM prune, crew manifest, intelligence curation, orphan pruning, quicksave | Negligible with <10 sessions | **Bottleneck.** Every 5-30 seconds, multiple subsystems read the entire state hash. With 50+ sessions and their context, this could be megabytes per read. |
| **Single SQLite file under `tokio::sync::Mutex`** | Works fine with sequential drains | **Contention.** `save_fact`, `save_knowledge`, `save_context_snapshot`, `drain_all`, `query_facts`, `get_identity_bio`, `hydrate_all` all compete for the same mutex. Under concurrent agent load, blocking lock contention will cause latency spikes. |
| **In-memory session HashMap under `tokio::sync::Mutex`** | Works fine with <10 sessions | **Lock contention.** Every heartbeat, session lookup, and pruning cycle acquires this lock. High-frequency heartbeats from many agents will bottleneck here. |
| **All state in one Redis hash** (`koad:state`) | Simple and queryable | **Fragile.** A single corrupt field or an overly large state hash can degrade all operations. No key isolation between session data, lease data, system stats, health registry, crew manifest. |
| **`println!`/`eprintln!` mixed with tracing** | Cosmetic issue | **Observability gap.** As the system grows, unstructured stdout/stderr becomes noise. Diagnostics, kernel startup, and router all use println — these won't appear in log files managed by tracing-appender. |

### Testability

| Test File | Coverage | Assessment |
|---|---|---|
| `engine/tests.rs` | Redis lifecycle, command execution, path integrity (3 tests) | **Integration tests requiring live Redis.** Not runnable in CI without Redis. Test assertions are timing-dependent (polling loops with sleep). |
| `discovery/tests.rs` | Skill manifest scanning (1 test) | Unit test, no external deps. Good. |
| `sandbox.rs` (inline) | Admin bypass, developer blacklist/sanctuary, compliance policy (5 tests) | Unit tests. Good coverage of sandbox rules. **But:** tests use old string-based `Sandbox::evaluate("admin", ...)` API — this doesn't compile against the current Identity-based API. **Tests are broken.** |

**Critical finding: The sandbox tests reference `Sandbox::evaluate("admin", cmd)` but the current implementation takes `Sandbox::evaluate(&Identity, cmd)`. These tests will not compile. This means sandbox policy changes are not validated by any test.**

#### Minimal Test Harness Proposal

1. **Unit tests (no Redis):** Sandbox policy, CIP enforcement, lease struct serialization, context budget calculation
2. **Integration tests (mock Redis):** Use `fred`'s mock or a testcontainers Redis instance. Test lease acquire/release/heartbeat, drain/hydrate, session lifecycle
3. **Property tests:** Fuzz sandbox blacklist with encoding variations to find bypasses

### Blast Radius

**If Spine fails completely:**
- All agent boots fail (CLI cannot call `InitializeSession`)
- All heartbeats fail → ASM marks all sessions dark → deadman triggers emergency save
- No commands can be dispatched via DirectiveRouter
- No signals can be sent/received
- No hydration, no context caching, no intelligence curation
- The Watchdog detects via failed `Heartbeat` RPC and triggers `reboot_spine()` after `max_failures` (3) consecutive failures at `check_interval_secs` (10s) intervals — **30 seconds to recovery.**
- Data loss: up to 30 seconds of Redis state (drain interval). Mitigated by `set_state()` doing immediate SQLite write for critical state.
- **Blast radius: Total system offline for ~30 seconds.** This is acceptable for a single-host deployment with Watchdog recovery, but the Spine is an absolute single point of failure.

**Rating: :yellow_circle:**

Maintainable and sustainable at current scale (1-10 agents, single host). The architecture is sound for this scope. However, three patterns will require attention as the system grows: the `HGETALL` anti-pattern, the single-mutex SQLite, and the monolithic `koad:state` hash. The most immediate sustainability risk is the broken sandbox tests — this means the security policy has no automated validation. Adding a test harness for the sandbox and lease manager should be the highest-priority sustainability investment.

---

## Critical Findings

1. **Arbitrary file read via `HydrateContext` and `GetFileSnippet` RPCs** — Any gRPC caller can read any file the Spine process has access to. No path validation or sandboxing. Paths are taken directly from untrusted RPC requests. (`hydration.rs:38-48`, `context_cache.rs:47-52`)

2. **`SystemAction::Reboot` via Redis pub/sub causes immediate `process::exit(0)`** — Any process that can publish to `koad:commands` can kill the Spine instantly without graceful shutdown, drain, or audit trail. (`router.rs:119`)

3. **Sandbox unit tests are broken** — Tests reference `Sandbox::evaluate("admin", cmd)` but current API takes `Sandbox::evaluate(&Identity, cmd)`. Sandbox policy changes have no automated validation. (`sandbox.rs:136-195`)

---

## High Findings

1. **No SIGTERM handler** — Only SIGINT (ctrl_c) triggers graceful shutdown. SIGTERM (systemd stop, `kill`) causes immediate exit without drain or ASM cleanup. Up to 30 seconds of state could be lost. (`main.rs:42-47`)

2. **ShipDiagnostics scope creep** — Diagnostics struct performs health monitoring, autonomic service recovery, session pruning, intelligence curation, cognitive quicksaves, and crew manifest assembly. This creates overlap with koad-asm (pruning, manifest) and koad-watchdog (recovery). (`diagnostics.rs` — 800+ lines, 12+ methods)

3. **Dual session pruning with different logic** — `ShipDiagnostics.prune_orphaned_sessions()` uses a hardcoded 300s timeout. `koad-asm:static_prune()` uses config-driven per-agent `session_policy` timeouts. Both run concurrently and can race on `HDEL` operations. (`diagnostics.rs:635`, `koad-asm/src/main.rs:92`)

4. **Hydration duplicate check doesn't prevent re-insertion** — `hexists` check logs "Hydration Canceled" but does not return early. The chunk is always re-inserted regardless of the check result. Logic bug. (`hydration.rs:68-78`)

5. **`register_component` always returns `authorized: true`** — No actual authorization check. Stub that should be implemented or return UNIMPLEMENTED. (`rpc/mod.rs:319-327`)

6. **KCM `expect()` panic on missing `KOAD_HOME`** — Three `expect()` calls in KCM that will panic and crash the Spine if `KOAD_HOME` is unset. Should use `bail!`. (`kcm.rs:64,90,109`)

7. **Inconsistent logging** — `kernel.rs`, `router.rs`, and `diagnostics.rs` use `println!`/`eprintln!` for many operational messages instead of `tracing` macros. These bypass structured logging and file appenders.

---

## Medium Findings

1. **`Execute` RPC is a misleading stub** — Always returns `success: true` without executing anything. A caller would believe their command succeeded. Should return `Status::unimplemented()`. (`rpc/mod.rs:56-75`)

2. **`dispatch_task` defaults to "admin" identity** — Empty identity field defaults to `"admin"`. DirectiveRouter then falls back to a "ghost" Crew identity when "admin" session doesn't exist. Confusing fallback chain. (`rpc/mod.rs:99-103`, `router.rs:144-151`)

3. **No deduplication in autonomic recovery** — `_restart_asm()` and `_restart_gateway()` use `nohup` to spawn processes without checking if one is already running. Repeated recovery triggers could spawn duplicates. (`diagnostics.rs:393-424`)

4. **`HGETALL koad:state` called from 6+ locations** — Full state hash read every 5-30 seconds from drain, diagnostics, pruning, manifest, curation, and quicksave. Will become a bottleneck at scale.

5. **`stream_task_status` is a stub** — Returns one initial message and stops. Not functional for real task status streaming. (`rpc/mod.rs:139-157`)

6. **Hardcoded `nohup` for process management** — Service recovery uses `nohup` with hardcoded binary paths. No systemd integration or process supervisor awareness. (`diagnostics.rs:761-796`)

---

## Recommendations

1. **Add SIGTERM handler** alongside SIGINT. Both should trigger the same graceful shutdown path with drain.

2. **Add path validation to `HydrateContext` and `GetFileSnippet`** — Restrict file reads to `$KOAD_HOME` and registered project paths. Reject paths containing `..` after canonicalization.

3. **Remove or gate `SystemAction::Reboot`** — Either remove the `process::exit(0)` path entirely, or require an authenticated admin-rank session to trigger it (verify via session ID in metadata).

4. **Fix sandbox tests** — Update to use `Identity` struct API. Add tests for edge cases (argument quoting, path traversal, case variations).

5. **Replace `expect()` with `bail!`** in KCM — prevent panics from crashing the Spine.

6. **Fix hydration duplicate check** — Add `return Ok(existing_chunk)` after the `hexists` check to prevent redundant re-insertion.

7. **Mark stub RPCs as UNIMPLEMENTED** — `Execute`, `StreamTaskStatus`, `RegisterComponent` should return `Status::unimplemented()` rather than misleading success responses.

8. **Migrate `println!`/`eprintln!`** to `tracing` macros across kernel.rs, router.rs, and diagnostics.rs.

---

## Role Boundary Proposals

### Proposal 1: Extract Autonomic Recovery from ShipDiagnostics to Watchdog

**Current:** ShipDiagnostics detects service failures AND attempts recovery (restart ASM, restart gateway).
**Proposed:** ShipDiagnostics detects and reports failures to a health registry. Watchdog reads the health registry and performs recovery.
**Boundary:** Spine diagnoses. Watchdog recovers.

### Proposal 2: Consolidate Session Pruning in koad-asm

**Current:** Both ShipDiagnostics (300s hardcoded) and koad-asm (config-driven) prune sessions.
**Proposed:** Remove `prune_orphaned_sessions()` from ShipDiagnostics. Let koad-asm be the sole authority for session lifecycle management.
**Boundary:** Spine manages live sessions (create, heartbeat, terminate). ASM manages dead sessions (dark, prune, deadman).

### Proposal 3: Extract Intelligence Curation from Diagnostics

**Current:** ShipDiagnostics promotes hot context chunks to L3 FactCards and performs cognitive quicksaves.
**Proposed:** Create a `CognitiveCurator` engine module (or delegate to ASM) that handles L2→L3 promotion and quicksaves on its own schedule.
**Boundary:** Diagnostics monitors health. Curator manages intelligence.

---

## Summary

The Koad Spine is a well-architected central kernel that correctly serves as the serialization point for agent identity, session state, and command authorization. The dual-path storage model (Redis hot / SQLite cold) with periodic drain and self-healing hydration is a strong foundation for crash resilience. The Lua-script-based atomic lease acquisition is correct and race-free for single-Redis deployments.

The primary concerns are: (1) two arbitrary file read paths in the gRPC service that bypass all path validation — these are the most actionable security findings; (2) a `process::exit(0)` path reachable via Redis pub/sub that bypasses graceful shutdown; (3) scope creep in ShipDiagnostics creating overlap with ASM and Watchdog; (4) broken sandbox tests leaving the security policy unvalidated; and (5) missing SIGTERM handling. None of these require architectural redesign — they are surgical fixes and boundary clarifications that can be addressed incrementally.

The Spine is stable enough for current single-host, small-fleet operation. The 30-second Watchdog recovery SLA is acceptable. For growth beyond 10+ agents, the `HGETALL` pattern, single-mutex SQLite, and monolithic state hash will need refactoring.

---

VIGIL SIGNING OFF — Awaiting Admiral review.
