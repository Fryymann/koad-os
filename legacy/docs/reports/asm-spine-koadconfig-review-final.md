
# ASM & Spine Reliability Review — Unified Implementation Plan

**Date:** 2026-03-11

**Reviewer:** Tyr (Captain)

**Authority:** Dood Override — Final Synthesis

## Executive Summary

This report synthesizes two deep-grid reviews (2026-03-10 and 2026-03-11) of the KoadOS Agent Session Manager (ASM) and Spine. The primary architectural risk is the **distributed race condition** during agent boot and the **hardcoded lifecycle constants** that ignore the tiered needs of different agent ranks (Admiral vs. Crew).

We are moving from a convention-based "best effort" session model to a **Strict Identity & Policy Protocol**. This includes implementing atomic Redis Lua scripts for lease management, config-driven session timeouts, and hardening the cognitive isolation between agents operating on the same host.

---

## 1. Unified Architecture Audit

### 1.1 The Boot Race Condition
- **Finding:** Both reviews confirmed that `KAILeaseManager::acquire_lease` performs a non-atomic `HGET` -> `HSET` check. In a concurrent multi-agent boot, two sessions can theoretically claim the same identity.
- **Fix:** Move lease acquisition to a server-side **Redis Lua Script** to ensure 100% atomicity.

### 1.2 Hardcoded Fragility
- **Finding:** Session timeouts (`dark`=60s, `purge`=300s, `deadman`=45s) are hardcoded in `koad-asm/src/main.rs`. This ignores the context-switching needs of complex agents like Tyr.
- **Fix:** Migrate all intervals to `crates/koad-core/src/config.rs` under a new `[sessions]` block, allow per-agent overrides in `identities/*.toml`.

### 1.3 Identity Resolution Drift
- **Finding:** The CLI formerly relied on `config/` (Legacy) while the Spine uses the new `Registry`.
- **Fix:** Purged `config/` entirely. Standardized on the `KoadConfig` Registry and `config/identities/` TOML files as the canonical source.

### 1.4 Recovery Gaps (Context Loss)
- **Finding:** "Hot Context" is only quicksaved every 5 minutes. A Redis crash results in significant memory loss.
- **Fix:** Integrate `koad:context:*` keys into the `StorageBridge` 30s drain loop.

---

## 2. Implementation Roadmap (The "Captain's Path")

### Phase 1: Core Configuration Expansion (High Priority)
- **Task 1.1:** Update `KoadConfig` (in `koad-core`) to include `SessionsConfig` and `WatchdogConfig` structs.
- **Task 1.2:** Implement per-agent `SessionPolicy` in `IdentityConfig`.
- **Task 1.3:** Refactor `koad-asm` to initialize using values from `KoadConfig::load()`.

### Phase 2: Atomic Identity Uplink (Critical Path)
- **Task 2.1:** Implement `acquire_lease.lua` and `heartbeat.lua` scripts in the Spine's `KAILeaseManager`.
- **Task 2.2:** Update `InitializeSession` to perform mandatory TOML profile validation.
- **Task 2.3:** Refactor `heartbeat` logic from O(n) key-scanning to O(1) direct mapping.

### Phase 3: Resilience & Monitoring (Maintenance Path)
- **Task 3.1:** Update `koad-watchdog` to monitor the `koad-asm` process independently or merge ASM into a dedicated Spine thread.
- **Task 3.2:** Harden "One Body, One Ghost" by validating `KOAD_BODY_ID` in every `ExecuteRequest`.
- **Task 3.3:** Move `drain_interval` and `reaper` thresholds to `kernel.toml`.

---

## 3. Implementation Priorities

| Priority | Component | Change | Reason |
| :--- | :--- | :--- | :--- |
| **CRITICAL** | Spine | Atomic Lease (Lua) | Prevents cognitive collisions. |
| **HIGH** | Core | Config Migration | Eliminates hardcoded fragility. |
| **HIGH** | CLI | Legacy Purge | Resolves Identity Resolution Drift. |
| **MEDIUM** | Spine | Context Drain | Prevents memory loss on crash. |

---

## 4. Open Questions for Dood

1. **ASM Integration:** Should we fold the `koad-asm` reaper logic directly into the Spine process to reduce IPC overhead and simplify monitoring? (Recommendation: **YES**)
2. **Strictness:** Should a `--force` boot notify the previous terminal (via Signal) before assassinating the session? (Recommendation: **YES**)
3. **Sovereign Lock:** Should we prevent *all* non-Sovereign agents from using `--force`? (Recommendation: **YES**)
