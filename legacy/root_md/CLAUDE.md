# KoadOS — Project Context

> **Vigil Security Review** — Generated 2026-03-10. Read in full before touching code.

---

## Architecture Overview

KoadOS is a multi-agent orchestration runtime written in Rust. It manages the full lifecycle of KAI (KoadOS AI) agent sessions: identity resolution, session leasing, command authorization, hybrid state persistence, and autonomic health monitoring.

### Workspace Structure (v3.2.0)

```
.koad-os/
├── Cargo.toml              Workspace root (8 crates)
├── proto/
│   └── spine.proto         gRPC service definitions
├── config/
│   ├── kernel.toml         System-wide: ports, storage, session timeouts, watchdog
│   ├── registry.toml       Per-project: directory → GitHub repo mappings, credential refs
│   ├── identities/         Per-agent: bio, rank, access_keys, session_policy
│   │   ├── tyr.toml
│   │   └── vigil.toml
│   └── integrations/
│       └── github.toml     GitHub API defaults
└── crates/
    ├── koad-core/          Shared types, traits, config loader — library only
    ├── koad-proto/         Protobuf/tonic compiled bindings — library only
    ├── koad-spine/         Central gRPC kernel + engine — binary (kspine)
    ├── koad-asm/           Agent Session Manager daemon — binary (koad-asm)
    ├── koad-watchdog/      Autonomic health monitor — binary (koad-watchdog)
    ├── koad-cli/           CLI user interface — binary (koad)
    ├── koad-board/         GitHub board integration — library
    └── koad-bridge-notion/ Notion API bridge — library
```

### Crate Dependency Graph

```
koad-cli ──────────────────────────────────────┐
koad-asm ──────────┬──────────────────────────►koad-core
koad-watchdog ──────┤                          │
koad-spine ─────────┘◄──────────────────────── koad-proto
koad-board ─────────────────────────────────── (standalone)
```

**Entry points (binaries):**
- `koad` → `crates/koad-cli/src/main.rs`
- `kspine` → `crates/koad-spine/src/main.rs`
- `koad-asm` → `crates/koad-asm/src/main.rs`
- `koad-watchdog` → `crates/koad-watchdog/src/main.rs`

---

## Key Concepts

| Concept | Definition | Primary File(s) |
|---|---|---|
| **Body** | A physical terminal session. Identified by `KOAD_BODY_ID` (UUID). One Body hosts One Ghost. | `crates/koad-cli/src/handlers/boot.rs` |
| **Ghost** | An active KAI agent consciousness running in a Body. | Same |
| **One Body One Ghost** | Enforced constraint: a terminal can host exactly one active agent at a time. Checked via `KOAD_SESSION_ID` + Redis lease. | `boot.rs`, `identity.rs` |
| **Spine** | The central gRPC kernel. Manages all agent sessions, state, and RPC dispatch. Binary: `kspine`. | `crates/koad-spine/` |
| **ASM** | Agent Session Manager. Background daemon that prunes stale sessions, enforces ghost rules, triggers deadman saves. | `crates/koad-asm/src/main.rs` |
| **Sentinel / Hydration** | On boot, the Spine hydrates the agent's working memory from SQLite into Redis. | `crates/koad-spine/src/engine/storage_bridge.rs` |
| **Watchdog** | Autonomic health monitor. Checks Spine and ASM liveness; restarts if max_failures exceeded. | `crates/koad-watchdog/src/main.rs` |
| **Atomic Lease** | A Redis Lua-script-protected identity lock. Prevents concurrent boot of the same KAI name. Stored at `koad:state` → `koad:kai:{name}:lease`. | `crates/koad-spine/src/engine/identity.rs` |
| **KAILease** | Struct: `identity_name`, `session_id`, `body_id`, `expires_at`, `is_sovereign`. | `crates/koad-spine/src/engine/identity.rs` |
| **Deadman Switch** | If a Tier 1 agent misses heartbeats for `deadman_timeout_secs`, ASM triggers an emergency SQLite drain. | `crates/koad-asm/src/main.rs:155` |
| **StorageBridge** | Hot path (Redis) + Cold path (SQLite) dual-store. 30-second periodic drain loop. | `crates/koad-spine/src/engine/storage_bridge.rs` |
| **CIP** | Cognitive Integrity Protocol. Tier > 1 agents cannot write sovereign Redis keys (`identities`, `identity_roles`, `knowledge`, `principles`, `canon_rules`). | `storage_bridge.rs:286` |
| **Cognitive Isolation** | Each KAI has a dedicated SQLite partition in `intelligence_bank` keyed by `source_agent`. Context snapshots retained (last 2 per agent). | `storage_bridge.rs` |
| **Identity-Aware Sandbox** | Command authorization checks the full `Identity` struct (`rank`, `tier`, `access_keys`) — not hardcoded name strings. | `crates/koad-spine/src/engine/sandbox.rs` |
| **Memory Banks** | Per-agent FactCards stored in `intelligence_bank` (SQLite). Hydrated into Redis on boot. | `storage_bridge.rs` |
| **KOAD_HOME** | Root of the koad-os installation. Defaults to `~/.koad-os`. All paths are relative to this. | `crates/koad-core/src/config.rs` |

---

## Config System — TOML Registry

### Three-Tier Hierarchy

```
kernel.toml          (system-wide defaults)
    ↓
registry.toml        (per-project overrides + directory mappings)
    ↓
identities/*.toml    (per-agent identity, access_keys, session_policy)
    ↓
KOAD_ env vars       (runtime overrides, highest priority)
```

**Load order in `KoadConfig::load()` (`crates/koad-core/src/config.rs`):**
1. `$KOAD_HOME/config/kernel.toml` — base network, storage, session, watchdog defaults
2. `$KOAD_HOME/config/integrations/github.toml` — GitHub API defaults
3. `$KOAD_HOME/config/registry.toml` — project directory → repo mappings
4. All `$KOAD_HOME/config/identities/*.toml` — merged into `config.identities` HashMap
5. Environment variables with `KOAD_` prefix override any value from TOML

### TOML Tier Discipline

| Value Type | TOML File | Examples |
|---|---|---|
| Ports, network addresses, socket paths | `kernel.toml [network]` | `gateway_port`, `spine_grpc_addr` |
| Storage paths, DB names | `kernel.toml [storage]` | `db_name` |
| Session timeouts (system defaults) | `kernel.toml [sessions]` | `deadman_timeout_secs`, `lease_duration_secs` |
| Watchdog settings | `kernel.toml [watchdog]` | `check_interval_secs`, `max_failures` |
| GitHub repo mappings, credential key refs | `registry.toml [projects.*]` | `path`, `github_owner`, `credential_key` |
| Agent bio, rank, preferences | `identities/<name>.toml [identity]` | `name`, `rank`, `bio` |
| Agent credential authorization | `identities/<name>.toml [preferences]` | `access_keys` |
| Per-agent session timeout overrides | `identities/<name>.toml [session_policy]` | `deadman_timeout_secs`, `purge_timeout_secs` |

### How `access_keys` Work

`access_keys` in an identity TOML is a list of **environment variable names** (not values). The agent is authorized to resolve these env vars at runtime. Example:

```toml
# config/identities/tyr.toml
[preferences]
access_keys = ["GITHUB_ADMIN_PAT"]
```

Means Tyr's session can resolve and use the `GITHUB_ADMIN_PAT` environment variable. The token itself is never stored in config.

**GitHub token resolution priority (descending):**
1. Project's `credential_key` (from `registry.toml`)
2. Identity's `access_keys` array
3. Fallback: `GITHUB_PAT` → `GITHUB_ADMIN_PAT` → `GITHUB_PERSONAL_PAT`

### How `session_policy` Works

Per-identity overrides for ASM timeout rules:

```toml
# config/identities/vigil.toml
[session_policy]
deadman_timeout_secs = 60    # overrides kernel.toml default (45)
dark_timeout_secs = 120      # overrides kernel.toml default (60)
purge_timeout_secs = 600     # overrides kernel.toml default (300)
lease_duration_secs = 300    # overrides kernel.toml default (90)
```

If `session_policy` is absent, ASM uses `kernel.toml [sessions]` defaults.

### Adding a New Config Value

1. Identify the correct TOML tier (see table above).
2. Add the key with a sensible default to the appropriate `.toml` file.
3. Add the corresponding field to the `KoadConfig` struct in `crates/koad-core/src/config.rs`.
4. Wire the `KOAD_` environment variable override in `KoadConfig::load()`.
5. Add a constant with the default value to `crates/koad-core/src/constants.rs`.
6. Document the new key in this file under the relevant section.

### Adding a New Agent Identity

1. Create `$KOAD_HOME/config/identities/<name>.toml`.
2. Required fields:
   ```toml
   [identity]
   name = "AgentName"       # Exact match for boot --agent
   role = "Role description"
   rank = "Officer"         # Admiral | Captain | Officer | Crew
   bio  = "One-line bio"

   [preferences]
   access_keys = ["ENV_VAR_KEY_NAME"]  # env var names this agent may resolve
   ```
3. Optional `[session_policy]` section for timeout overrides.
4. Run `koad boot --agent AgentName` — Spine auto-hydrates from the new TOML.
5. No code changes required. The config loader globs all `identities/*.toml`.

---

## Multi-Agent Architecture

### Atomic Leases — Preventing Boot Collisions

When a KAI boots, Spine runs a Lua script **atomically in Redis** to acquire a named lease:

```lua
-- ACQUIRE_LEASE_LUA (crates/koad-spine/src/engine/identity.rs)
local existing = redis.call("HGET", state_key, lease_key)
if existing and not force then
    return {err = "IDENTITY_LOCKED"}
end
redis.call("HSET", state_key, lease_key, lease_data)
return "OK"
```

Heartbeat renewal is also atomic:

```lua
-- HEARTBEAT_LUA
if lease["session_id"] == session_id then
    lease["expires_at"] = now_iso
    redis.call("HSET", ...)
    return "OK"
end
return {err = "LEASE_MISMATCH"}
```

**Key property:** If two agents try to boot the same identity simultaneously, exactly one succeeds. The Lua script is the single serialization point.

### Session Isolation

- **`KOAD_SESSION_ID`** — unique UUID per boot, injected into shell env. The CLI checks this first (Gate 1) to prevent double-boot in the same terminal.
- **`KOAD_BODY_ID`** — UUID identifying the physical terminal session. Persists across reboots of the same terminal. Stored in `KAILease.body_id`.
- **Redis key namespacing:** `koad:session:{session_id}`, `koad:kai:{agent_name}:lease` — sessions are isolated by session_id, not agent name.
- **SQLite partitioning:** `intelligence_bank.source_agent` column isolates FactCards per agent. Context snapshots keyed by agent name.

### Heartbeat / Health Model

- CLI sends periodic heartbeats via `SpineService::Heartbeat` RPC.
- Heartbeat renews the Redis lease via `HEARTBEAT_LUA`.
- ASM checks all sessions every `check_interval_secs` (default: 10s):
  - **Ghost enforcement:** If a newer lease exists for an identity, old sessions are purged.
  - **Deadman switch:** Tier 1 agents silent for `deadman_timeout_secs` → emergency SQLite drain.
  - **Dark state:** Silent for `dark_timeout_secs` → marked `"dark"`, retained.
  - **Purge:** Silent for `purge_timeout_secs` → removed from Redis.

### StorageBridge Drain Loop

Every 30 seconds, `start_drain_loop()` runs `drain_all()`:
1. `HGETALL koad:state` — reads full Redis state hash.
2. Batch `INSERT OR REPLACE` into `state_ledger` (SQLite).
3. On Deadman trigger: ASM calls `SpineService::DrainAll` RPC for immediate drain.

Cognitive context (`hot_context` in sessions) is included in the drain, ensuring crash resilience.

---

## Build & Test

```bash
# Build all crates
cargo build --workspace

# Build release
cargo build --workspace --release

# Run the Spine
cargo run -p koad-spine

# Run the CLI
cargo run -p koad-cli -- boot --agent Tyr

# Run a specific crate's tests
cargo test -p koad-core

# Run all tests
cargo test --workspace
```

**Quirks:**
- `rusqlite` uses the `bundled` feature — no system SQLite required.
- `fred` (Redis client) uses Unix socket support — ensure `koad.sock` exists at `$KOAD_HOME`.
- Proto compilation via `prost-build` runs at build time. `proto/spine.proto` changes require rebuild.
- `koad-watchdog` has its own `Cargo.toml` with direct dependency versions (not all inherited from workspace).

---

## File Map

```
Cargo.toml                  Workspace root, shared dependencies
proto/spine.proto           gRPC service + message definitions
config/kernel.toml          System-wide network, storage, session, watchdog config
config/registry.toml        Project directory → GitHub repo mappings
config/identities/          Per-agent identity TOML files (one per KAI)
config/integrations/        Third-party API config (GitHub, etc.)
crates/koad-core/           Shared types: Identity, AgentSession, KoadConfig, constants
crates/koad-proto/          Compiled protobuf bindings (tonic/prost)
crates/koad-spine/          Spine kernel: RPC server, engine, identity, storage, sandbox
crates/koad-cli/            CLI: boot, status, signal, intel, board, cognitive handlers
crates/koad-asm/            ASM daemon: session pruning, ghost enforcement, deadman
crates/koad-watchdog/       Watchdog daemon: health checks, autonomic reboot
crates/koad-board/          GitHub issue/board integration
crates/koad-bridge-notion/  Notion API bridge
docs/                       Admiral orders, agent thoughts, review docs
reports/                    Audit and review reports
```

---

## Conventions

**Naming:**
- Crate names: `koad-<component>` (kebab-case)
- Module names: snake_case
- Redis keys: `koad:{component}:{identifier}` pattern
- Config env vars: `KOAD_` prefix

**Error handling:**
- `anyhow::Result` used throughout for propagation
- `bail!()` macro for fatal errors with descriptive messages
- Errors surface with context strings (e.g., `"CONSCIOUSNESS_COLLISION: ..."`, `"SOVEREIGN_OCCUPIED: ..."`)

**Logging:**
- `tracing` crate: `info!`, `warn!`, `error!`, `debug!`
- `tracing-appender` for file-based log rotation
- Structured events for telemetry via `REDIS_CHANNEL_TELEMETRY`

**Async:**
- `tokio` runtime (`#[tokio::main]`)
- `async-trait` for trait-based async methods
- Blocking SQLite operations wrapped in `tokio::task::spawn_blocking`

---

## Configuration System

KoadOS utilizes a three-tier TOML Registry for all settings. The legacy `koad.json` has been fully deprecated and removed.

### Config Hierarchy
1.  **Kernel** (`config/kernel.toml`): System-wide network, storage, and session defaults.
2.  **Filesystem** (`config/filesystem.toml`): Path mappings and workspace symlinks.
3.  **Registry** (`config/registry.toml`): Project-to-repository mappings and credentials.
4.  **Integrations** (`config/integrations/*.toml`): Third-party service IDs (GitHub, Notion, Airtable).
5.  **Personas** (`config/identities/*.toml`): Agent bios, ranks, and strategic bootstraps.
6.  **Interfaces** (`config/interfaces/*.toml`): Technical driver bootstraps and tool sets.

### Identity-Based Authority
Sovereign status and feature access are derived from `identity.rank` and `model_tier`, never from name strings.
- **Admiral / Captain**: Full Sovereign clearance.
- **Officer**: High-clearance technical agents.
- **Tier 1**: Mandatory for Admiral, Captain, and Officer ranks.

---

## Hardcoded Values Registry

Complete inventory of values in source that should be in TOML config.

| File:Line | Current Value | Recommended TOML | Key | Risk |
|---|---|---|---|---|
| `koad-core/src/constants.rs` | `3000` | `kernel.toml [network]` | `gateway_port` | Medium |
| `koad-core/src/constants.rs` | `"0.0.0.0:3000"` | `kernel.toml [network]` | `gateway_addr` | High — binds all interfaces |
| `koad-core/src/constants.rs` | `50051` | `kernel.toml [network]` | `spine_grpc_port` | Medium |
| `koad-core/src/constants.rs` | `"http://127.0.0.1:50051"` | `kernel.toml [network]` | `spine_grpc_addr` | Medium |
| `koad-core/src/constants.rs` | `"koad.sock"` | `kernel.toml [network]` | `redis_socket` | Low |
| `koad-core/src/constants.rs` | `"kspine.sock"` | `kernel.toml [network]` | `spine_socket` | Low |
| `koad-core/src/constants.rs` | `"koad.db"` | `kernel.toml [storage]` | `db_name` | Low |
| `koad-core/src/constants.rs` | `"https://api.github.com"` | `integrations/github.toml` | `api_base` | Medium |
| `koad-core/src/constants.rs` | `"https://api.notion.com/v1"` | future `integrations/notion.toml` | `api_base` | Low |
| `koad-spine/src/engine/sandbox.rs:15` | `"GITHUB_ADMIN_PAT"` | identity `access_keys` check | — | **Critical — privilege bypass** |
| `koad-spine/src/engine/sandbox.rs:50-53` | Production trigger strings | `kernel.toml [sandbox]` | `production_triggers` | High |
| `koad-spine/src/engine/sandbox.rs:99` | Blacklist commands | `kernel.toml [sandbox]` | `blacklisted_commands` | High |
| `koad-spine/src/engine/sandbox.rs:111` | Protected paths | `kernel.toml [sandbox]` | `protected_paths` | High |
| `koad-spine/src/engine/storage_bridge.rs` | `30` seconds drain interval | `kernel.toml [storage]` | `drain_interval_secs` | Low |
| `koad-cli/src/handlers/boot.rs` | `"Tyr"`, `"Dood"`, `"Vigil"` | Resolved | 2026-03-11 |
| `koad-spine/src/engine/identity.rs` | `"Tyr"`, `"Koad"`, `"Ian"` | Resolved | 2026-03-11 |
| `koad-core/src/constants.rs` | `3000` | `kernel.toml [network]` | `gateway_port` |

---

## Security Notes

> Full details in `VIGIL_AUDIT.md`. Summary of critical concerns:

**Trust Boundaries:**
- The Spine gRPC endpoint (`:50051`) is unauthenticated — any local process can call it. Mitigated by localhost-only binding, but insufficient for multi-user hosts.
- Redis Unix socket (`koad.sock`) is protected only by filesystem permissions on `$KOAD_HOME`. Ensure restrictive permissions (`700`).
- SQLite database (`koad.db`) is unencrypted at rest.

**Sandbox Integrity:**
- The `GITHUB_ADMIN_PAT` credential bypass in `sandbox.rs:15` grants unconditional command execution to any agent holding that key. This is a single point of failure.
- Command blacklist uses substring matching — bypasses possible via argument quoting, spacing, or case variations.
- Protected path checks do not canonicalize paths — symlink traversal and `../` sequences can bypass them.

**Atomic Lease Integrity:**
- The Lua scripts are correct for the single-Redis-instance case. In a clustered Redis environment, EVAL atomicity guarantees do not hold across slots.
- `--force` boot bypasses the lease check entirely. This is intentional but must be admin-gated.

**Identity Enumeration:**
- Hardcoded sovereign name lists in `boot.rs` and `identity.rs` reveal the full set of privileged identities to anyone reading the source. Should derive from `rank` field instead.

**Tier 1 Unconstrained Write:**
- The CIP (`storage_bridge.rs:286`) protects sovereign keys from Tier 2+ agents, but Tier 1 Admin models have no write restrictions at all. A compromised Tier 1 session can overwrite `identities`, `principles`, and `canon_rules`.

**No TLS, No Auth on gRPC:**
- All RPC traffic is plaintext. For localhost-only deployment this is acceptable; for any networked deployment, TLS + mutual auth is required.
