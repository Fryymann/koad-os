# VIGIL_AUDIT.md — KoadOS Hardcoded Value & Legacy Audit Report

**Auditor:** Vigil [Security]
**Date:** 2026-03-10
**Scope:** Full codebase read-only audit. No source files modified.
**Methodology:** `Glob` + `Grep` + `Read` across all crates. Every .rs file, all config files, proto definitions, and existing documentation reviewed.
**Multi-agent lens:** Each finding is assessed for impact under simultaneous multi-agent operation.

---

## Executive Summary

KoadOS v3.2.0 has undergone a significant architecture refactor from a monolithic legacy config to a three-tier TOML Registry. The new architecture (kernel.toml → registry.toml → identities/*.toml) is fully implemented.

- **All active legacy `legacy config` references** have been removed from the CLI crate.
- **Hardcoded sovereign name-string lists** have been replaced with the Identity struct rank/tier check.
- **17+ hardcoded values** across source files that belong in TOML config.
- **10 missing security controls** that create exploitable attack surface, especially under multi-agent load.
- The Atomic Lease Lua scripts are correct for single-Redis-instance deployment. Concurrency is safe.
- The `access_keys` model is well-designed but has one critical bypass: `GITHUB_ADMIN_PAT` grants unconditional Sandbox approval.

---

## Part A: Legacy legacy config References

| File:Line | What It Does | Migration Target | Status |
|---|---|---|---|
| `crates/koad-cli/src/config_legacy.rs` (full file) | Defines `KoadLegacyConfig`, `KoadIdentity`, `KoadDriverConfig`, `ProjectConfig` structs for parsing `legacy config` | Removed. All identity data comes from `config/identities/<name>.toml`; project data from `config/registry.toml` | **RESOLVED — 2026-03-11** |
| `crates/koad-cli/src/main.rs:35` | Loads `$KOAD_HOME/legacy config` at CLI startup as fallback identity source | Removed. Uses `config.get_agent_name()` and `config.identities` map. | **RESOLVED — 2026-03-11** |

**Multi-agent risk:** If `legacy config` contains a stale identity from a prior session, it may inject incorrect identity context into a new agent boot, causing session confusion. Under multi-agent load, this could cause the wrong identity to receive authorization.

---

## Part B: Hardcoded Name-String Bypasses

These are the highest-priority findings. Privilege is granted by comparing the agent name string against a hardcoded list — not by evaluating the Identity struct's `rank` or `tier` fields. This means:
1. Any new sovereign agent added to `identities/*.toml` must also be added to these lists — easy to forget.
2. The sovereign identity list is exposed in source code, enabling enumeration.
3. An attacker who can register a name matching a hardcoded entry gains elevated privilege.

| File:Line | Hardcoded Name/Check | Should Use | Risk |
|---|---|---|---|
| `crates/koad-cli/src/handlers/boot.rs:90` | `if agent == "Tyr" \|\| agent == "Koad" \|\| agent == "Dood" \|\| agent == "Vigil"` — grants sovereign lease flow | `if identity.rank == Rank::Captain \|\| identity.rank == Rank::Admiral` | **Critical** |
| `crates/koad-spine/src/engine/identity.rs:94` | `kai_name == "Tyr" \|\| kai_name == "Koad" \|\| kai_name == "Ian" \|\| kai_name == "TestKoad" \|\| kai_name == "Vigil"` — sets `is_sovereign` and enforces Tier 1 model requirement | Derive `is_sovereign` from `identity.rank` loaded from TOML. Captain/Admiral → sovereign. | **Critical** |

**Note on discrepancy:** The two lists are inconsistent. `boot.rs` includes `"Dood"` but not `"Ian"` or `"TestKoad"`. `identity.rs` includes `"Ian"` and `"TestKoad"` but not `"Dood"`. This means Ian (`"Dood"`) gets the sovereign lease flow in boot but does NOT get the Tier 1 enforcement in Spine, and `"TestKoad"` gets Tier 1 enforcement without the sovereign lease flow. These inconsistencies could cause unpredictable behavior in multi-agent scenarios.

---

## Part C: Hardcoded Values — Critical (secrets, credentials, tokens)

| File:Line | Current Value (redacted) | Recommended TOML Tier | Key Name |
|---|---|---|---|
| `crates/koad-spine/src/engine/sandbox.rs:15` | `"GITHUB_ADMIN_PAT"` — hardcoded env var key name used to grant unconditional Sandbox bypass | `config/identities/*.toml [preferences] access_keys` — authorization should check whether the identity's `access_keys` includes any key in a configurable `admin_bypass_keys` list from `kernel.toml [sandbox]` | **Critical — single-point credential bypass** |

**Risk detail:** Any agent whose `access_keys` contains `"GITHUB_ADMIN_PAT"` bypasses the entire Sandbox policy tree. If `GITHUB_ADMIN_PAT` is ever granted to a lower-trust agent (by mistake in their TOML), that agent gains unconditional command execution. The bypass key name is hardcoded — changing it requires a source edit.

---

## Part D: Hardcoded Values — High (ports, hosts, URLs, paths)

| File:Line | Current Value | Recommended TOML Tier | Key Name |
|---|---|---|---|
| `crates/koad-core/src/constants.rs` | `"0.0.0.0:3000"` (gateway bind address) | `kernel.toml [network]` | `gateway_addr` |
| `crates/koad-core/src/constants.rs` | `3000` (gateway port) | `kernel.toml [network]` | `gateway_port` |
| `crates/koad-core/src/constants.rs` | `"http://127.0.0.1:50051"` (Spine gRPC addr) | `kernel.toml [network]` | `spine_grpc_addr` |
| `crates/koad-core/src/constants.rs` | `50051` (Spine gRPC port) | `kernel.toml [network]` | `spine_grpc_port` |
| `crates/koad-core/src/constants.rs` | `"https://api.github.com"` | `config/integrations/github.toml` | `api_base` |
| `crates/koad-core/src/constants.rs` | `"https://api.notion.com/v1"` | `config/integrations/notion.toml` (create) | `api_base` |
| `crates/koad-spine/src/engine/sandbox.rs:50-53` | `"--project skylinks-prod"`, `"--live"`, `"stripe listen"`, `"gcloud functions deploy"` — production trigger list | `kernel.toml [sandbox]` | `production_triggers` (array) |
| `crates/koad-spine/src/engine/sandbox.rs:99` | `"sudo "`, `"su "`, `"rm -rf /"`, `"koad boot"` — blacklisted commands | `kernel.toml [sandbox]` | `blacklisted_commands` (array) |
| `crates/koad-spine/src/engine/sandbox.rs:111` | `".koad-os"`, `"/etc"`, `"/var"`, `"/root"` — protected paths | `kernel.toml [sandbox]` | `protected_paths` (array) |

**Multi-agent risk for `gateway_addr = "0.0.0.0":`** Gateway is accessible on all network interfaces. In a multi-agent deployment, this exposes agent command execution to the local network if there is no firewall. Should be `127.0.0.1` unless external access is explicitly required.

---

## Part E: Hardcoded Values — Medium (agent names, IDs, defaults)

| File:Line | Current Value | Recommended TOML Tier | Key Name |
|---|---|---|---|
| `crates/koad-cli/src/handlers/boot.rs:90` | `"Tyr"`, `"Koad"`, `"Dood"`, `"Vigil"` (sovereign name list) | Derive from `identity.rank` loaded from TOML | Replace with rank check |
| `crates/koad-spine/src/engine/identity.rs:94` | `"Tyr"`, `"Koad"`, `"Ian"`, `"TestKoad"`, `"Vigil"` (sovereign name list) | Derive from `identity.rank` loaded from TOML | Replace with rank check |
| `crates/koad-spine/src/engine/storage_bridge.rs` (drain loop) | `30` seconds (drain interval hardcoded in `Duration`) | `kernel.toml [storage]` | `drain_interval_secs` |
| `crates/koad-spine/src/engine/storage_bridge.rs:286` | `["identities", "identity_roles", "knowledge", "principles", "canon_rules"]` (CIP sovereign keys) | `kernel.toml [cip]` | `sovereign_keys` (array) |
| `crates/koad-core/src/constants.rs` | `"koad:telemetry"`, `"koad:sessions"` (Redis pub/sub channels) | `kernel.toml [redis]` | `channel_telemetry`, `channel_sessions` |
| `crates/koad-core/src/constants.rs` | `"koad:config"`, `"koad:health_registry"`, `"koad:state"` (Redis key names) | `kernel.toml [redis]` | `key_config`, `key_health_registry`, `key_state` |

---

## Part F: Hardcoded Values — Low (magic numbers, timeouts, buffer sizes)

| File:Line | Current Value | Recommended TOML Tier | Key Name |
|---|---|---|---|
| `crates/koad-watchdog/src/main.rs` | `10` seconds (sleep after watchdog reboot) | `kernel.toml [watchdog]` | `reboot_settle_secs` |
| `crates/koad-spine/src/engine/storage_bridge.rs` | `2` (max context snapshots retained per agent) | `kernel.toml [storage]` | `max_context_snapshots` |
| `crates/koad-asm/src/main.rs` | Session diff comparison uses `i64` cast of `Duration` | No TOML change needed, but precision loss if timeout > i64::MAX seconds (not realistic) | Low |
| `proto/spine.proto` | Default enum values (0 = unspecified) | Protobuf convention — acceptable | Informational |

---

## Part G: Recommended TOML Schema Additions

### `kernel.toml [sandbox]` — New Section

```toml
[sandbox]
# Commands that are unconditionally denied for all non-Captain/Admiral identities
blacklisted_commands = ["sudo ", "su ", "rm -rf /", "koad boot"]

# Substrings that trigger SLE Isolation Mandate for Officer+ identities
production_triggers = [
    "--project skylinks-prod",
    "--live",
    "stripe listen",
    "gcloud functions deploy",
]

# Paths protected from write operations (rm, mv, cp, echo, redirect, editors)
protected_paths = [".koad-os", "/etc", "/var", "/root"]

# Environment variable key names that grant Sandbox bypass when present in identity.access_keys
# Currently hardcoded as "GITHUB_ADMIN_PAT" — making this configurable removes source-level secret enumeration
admin_bypass_keys = ["GITHUB_ADMIN_PAT"]
```

**Multi-agent implication:** These rules apply uniformly to all agents. Changes to sandbox policy affect all simultaneously running agents immediately on next config reload.

### `kernel.toml [storage]` — Additional Keys

```toml
[storage]
db_name = "koad.db"
drain_interval_secs = 30         # StorageBridge Redis→SQLite drain frequency
max_context_snapshots = 2         # Rolling context snapshots retained per agent
```

### `kernel.toml [watchdog]` — Additional Key

```toml
[watchdog]
check_interval_secs = 10
max_failures = 3
monitor_asm = true
reboot_settle_secs = 10           # Sleep after reboot before resuming health checks
```

### `kernel.toml [cip]` — New Section (Cognitive Integrity Protocol)

```toml
[cip]
# Redis key prefixes that Tier 2+ agents cannot write to
sovereign_keys = ["identities", "identity_roles", "knowledge", "principles", "canon_rules"]
```

**Multi-agent implication:** CIP sovereign keys protect shared state that all agents read. Any Tier 2+ write to these keys would corrupt the shared identity model visible to all running agents.

### `config/integrations/notion.toml` — New File

```toml
[api]
base = "https://api.notion.com/v1"
```

---

## Part H: Atomic Lease & Concurrency Audit

### H.1 ACQUIRE_LEASE_LUA Analysis

**Location:** `crates/koad-spine/src/engine/identity.rs`

```lua
local existing = redis.call("HGET", state_key, lease_key)
if existing then
    local lease = cjson.decode(existing)
    if not force then
        return {err = "IDENTITY_LOCKED"}
    end
end
redis.call("HSET", state_key, lease_key, lease_data)
return "OK"
```

**Assessment: CORRECT for single-Redis-instance.**
- The Lua script executes atomically on the Redis server. No other command can interleave between the HGET and HSET within one Lua execution.
- The TOCTOU window that exists in non-Lua boot.rs checks (Gate 1 session check) does NOT affect the lease acquisition itself, only the pre-flight UX check.

**Concern 1 — Expiry not enforced in Lua:**
The Lua script does not check whether the existing lease has expired before returning `IDENTITY_LOCKED`. An expired-but-not-pruned lease will block a legitimate new boot until ASM prunes it (up to `check_interval_secs = 10` seconds). This is a minor availability issue, not a security issue.

**Recommended fix:** Add expiry check in Lua:
```lua
if existing then
    local lease = cjson.decode(existing)
    local now = tonumber(ARGV[1])
    -- Only block if lease is still valid
    if lease.expires_at_unix and lease.expires_at_unix > now then
        if not force then
            return {err = "IDENTITY_LOCKED"}
        end
    end
end
```

**Concern 2 — Clustered Redis (future):**
In a Redis Cluster, `HGET` and `HSET` on the same hash key (`koad:state`) are always on the same slot, so Lua atomicity holds. However, if the key namespace is ever split across multiple hash keys (e.g., per-agent state hashes), Lua atomicity would NOT hold across keys. Current design is safe.

**Concern 3 — `--force` with no audit trail:**
`--force` bypasses the lease check entirely. There is no audit log entry when `--force` is used. In a multi-agent environment, this means one agent can silently evict another without leaving a trace.

**Recommended fix:** Emit a `warn!` tracing event and a Redis telemetry event (`koad:telemetry`) when `--force` overrides a live lease.

### H.2 HEARTBEAT_LUA Analysis

```lua
if lease["session_id"] == session_id then
    lease["expires_at"] = now_iso
    redis.call("HSET", ...)
    return "OK"
end
return {err = "LEASE_MISMATCH"}
```

**Assessment: CORRECT.**
- Session ID comparison prevents one agent's heartbeat from renewing another's lease.
- Atomic: the read-modify-write is one Lua execution.

**Concern — ISO timestamp comparison vs Unix epoch:**
`expires_at` is stored and compared as ISO 8601 strings (via `now_iso`). String comparison of ISO timestamps works correctly if all timestamps are UTC with consistent formatting. If any clock skew or formatting inconsistency occurs, heartbeat renewal could fail silently.

**Recommended fix:** Store `expires_at` as a Unix epoch integer for reliable numeric comparison.

### H.3 Boot Gate 1 (CLI-side session check)

**Location:** `crates/koad-cli/src/handlers/boot.rs`

```rust
if let Ok(existing_sid) = env::var("KOAD_SESSION_ID") {
    let val: Option<String> = redis.pool.hget("koad:state", &session_key).await?;
    // Verify status field == "active"
    if session_is_alive { bail!("CONSCIOUSNESS_COLLISION: ..."); }
}
```

**Assessment: This is a UX guard, not a security gate.** The actual atomic lease acquisition happens in Spine's Lua script. Gate 1 provides a fast-fail user experience before making the gRPC call. An adversary who bypasses the CLI can still attempt a boot RPC — the Lua script will reject it correctly.

**Multi-agent risk:** Two agents in separate terminals with no `KOAD_SESSION_ID` set could both pass Gate 1 simultaneously and both call `InitializeSession`. The first Lua execution wins; the second gets `IDENTITY_LOCKED`. This is correct behavior.

---

## Part I: access_keys Privilege Audit

### I.1 Current access_keys Assignments

| Agent | Rank | access_keys | Effective Privilege |
|---|---|---|---|
| Tyr | Captain | `["GITHUB_ADMIN_PAT"]` | Full Sandbox bypass (via `GITHUB_ADMIN_PAT` hardcoded check). Full system access (Captain rank bypass). Tier 1 model required. |
| Vigil | Officer | `["GITHUB_PERSONAL_PAT"]` | No Sandbox bypass. Officer policy (SLE Isolation + agent policy). Tier 1 model required. |

### I.2 Privilege Escalation Vectors

**Vector 1 — GITHUB_ADMIN_PAT bypass (Critical):**
`sandbox.rs:15` checks `identity.access_keys.contains("GITHUB_ADMIN_PAT")` before any rank or tier check. If any identity TOML — even a Crew-rank agent — is given `access_keys = ["GITHUB_ADMIN_PAT"]`, that agent gains unconditional command execution. The bypass is independent of rank. This is a privilege escalation path that bypasses all other Sandbox controls.

**Recommendation:** Remove the `access_keys`-based bypass entirely, or restrict it to identities with `rank >= Officer`. The correct model is: `access_keys` authorizes credential resolution, not command execution. Command authorization should be rank + tier only.

**Vector 2 — Captain rank unconditional bypass (High):**
Any identity with `rank = "Captain"` in their TOML gets `PolicyResult::Allowed` for all commands. There is no audit of what a Captain-rank agent executes. If a Captain-rank identity TOML is created for a non-admin use case, it grants full system access.

**Recommendation:** Captain-rank bypass should log all allowed commands to `koad:telemetry`. Consider adding a `require_human_confirmation` flag for destructive commands even for Captain rank.

**Vector 3 — access_keys not validated against kernel.toml allowlist (Medium):**
Any env var name can be placed in `access_keys`. An identity could claim `access_keys = ["AWS_SECRET_ACCESS_KEY", "STRIPE_SECRET_KEY"]` and resolve credentials for unrelated systems. There is no allowlist of which env vars are legitimate for KoadOS access.

**Recommendation:** Add a `kernel.toml [auth] allowed_env_keys` allowlist. Only env var names in this list may appear in identity `access_keys`.

**Vector 4 — No expiry on access_keys grants (Low):**
`access_keys` are static in the TOML file. There is no mechanism to grant a time-limited credential access or rotate which env vars an agent can resolve without editing the TOML.

**Recommendation:** Future: support `access_keys` with expiry via a session-policy or runtime grant mechanism.

### I.3 Observations on Current Agent Scope

- **Tyr** holds `GITHUB_ADMIN_PAT` — appropriate for a Captain-rank admin agent. Risk: if Tyr's session is compromised, the attacker has both Captain-rank command bypass AND admin GitHub credentials.
- **Vigil** holds `GITHUB_PERSONAL_PAT` — appropriate for a read-audit security agent. Vigil cannot bypass the Sandbox via `access_keys`. Correct.
- **No Crew-rank agents** are currently defined with any `access_keys` — appropriate.

---

## Part J: Missing Security Controls (Informational)

These are absent controls that create exploitable attack surface. Documented here for triage and future remediation.

| # | Control | Risk | Priority |
|---|---|---|---|
| J1 | No encryption at rest (SQLite, Redis) | Full data breach via filesystem access | High |
| J2 | No TLS on gRPC (`:50051`) | Plaintext session data on loopback; unacceptable if multi-host | High |
| J3 | No authentication on gRPC RPC calls | Any local process can call `InitializeSession`, `Execute`, `DrainAll` | High |
| J4 | No audit log for authorization decisions | Cannot detect or investigate privilege abuse | High |
| J5 | No rate limiting on boot/heartbeat RPC | Enables brute-force session ID guessing | Medium |
| J6 | Command blacklist uses substring matching | Bypasses via argument quoting, spacing, case variations | High |
| J7 | Protected path check does not canonicalize | Symlink traversal, `../` bypass possible | High |
| J8 | No revocation mechanism for live sessions | Cannot invalidate a compromised session without killing Redis state | Medium |
| J9 | Tier 1 Admin has no write restrictions in CIP | Compromised Tier 1 can overwrite identities, principles, canon_rules | Critical |
| J10 | `--force` boot leaves no audit trail | Silent session eviction in multi-agent environment | Medium |
| J11 | Deadman switch only covers Tier 1 | Tier 2/3 agents can silently disappear without state persistence | Low |
| J12 | Redis socket permissions not enforced in code | Security depends entirely on filesystem permissions of `$KOAD_HOME` | High |
| J13 | No allowlist for `access_keys` env var names | Agents can claim resolution of any system env var | Medium |

---

## Part K: Multi-Agent Specific Risks

| Risk | Description | Affected Components | Priority |
|---|---|---|---|
| K1 | Inconsistent sovereign name lists | `boot.rs` and `identity.rs` have different hardcoded sovereign sets. Dood/Ian/TestKoad get different privilege flows. | boot.rs, identity.rs | Critical |
| K2 | `config/` identity injection | Stale configs could inject wrong identity into a new agent boot | koad-cli | **Resolved** |
| K3 | Ghost enforcement races | If two agents with the same identity name boot simultaneously on separate hosts sharing Redis, ghost enforcement in ASM may purge the wrong session | koad-asm | High |
| K4 | CIP bypass by Tier 1 agents | If a Tier 1 admin agent is compromised, it can rewrite shared sovereign state visible to all simultaneously running agents | storage_bridge.rs | Critical |
| K5 | Production trigger bypass | `production_triggers` list uses substring matching — obfuscated production commands could pass the SLE check in an Officer-session while co-running agents assume production is protected | sandbox.rs | High |
| K6 | No per-session sandbox policy | All agents of the same rank share identical sandbox policy. Cannot grant one Officer narrower or broader scope than another | sandbox.rs | Medium |
| K7 | `access_keys` GITHUB_ADMIN_PAT bypass shared across identity TOML files | If any simultaneously-running agent holds this key, they all bypass the Sandbox — isolation is nominal only | sandbox.rs | Critical |

---

*VIGIL_AUDIT.md — End of Report — Vigil [Security] — 2026-03-10*
