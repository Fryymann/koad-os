# Task: KoadOS Shell Environment Hydration Buildout (Phase 1.5 - 2.0)
**Status:** PLANNED / ACTIVE
**Owner:** Tyr (Captain)
**Trace ID:** TRC-TYR-20260314-HYDRATE
**Priority:** HIGH (Token Efficiency & Orientation Offload)

---

## 🎯 Mission Objective
Implement a high-performance shell hydration layer in the `koad-agent` bootstrap tool to eliminate agent orientation overhead and reduce token consumption by 40-60%.

---

## 🏗️ Architectural Design (Dark Mode Optimized)

### 1. Identity & Context (Environment Variables)
Exported directly by `koad-agent boot <agent> --shell`.
- `KOAD_AGENT_ROLE`: Full role string from `identity.toml`.
- `KOAD_AGENT_RANK`: Canon rank (Captain, Officer, Crew).
- `KOAD_AGENT_SCOPE`: Colon-separated authorized paths.
- `KOAD_SESSION_ID`: Unique UUID for trace auditing.
- `KOAD_BOOT_MODE`: "dark" (Citadel offline) | "full" (Citadel online).
- `KOAD_PROMPT_CACHE_HASH`: MD5 of core config + identity (for provider caching).

### 2. Pre-Generated Context Files (Zero-Infra)
Generated at boot in `~/.koad-os/cache/`.
- `session-brief.md`: Compiled from current Git branch, last 5 commits, and `~/.<agent>/memory/WORKING_MEMORY.md`.
- `doc-index.md`: A 3-layer structural map (domain -> directory -> key files) generated via fast README/Header parsing.
- `GEMINI.md` / `CLAUDE.md`: Dynamic system anchors injected into the agent's workspace to keep provider-level prompt caches warm.

### 3. Shell Functions & Utilities (The "Body" Surface)
Injected into the active Bash session.
- `koad-auth`: Directory-aware GitHub PAT selection (Skylinks vs. Fryymann).
- `koad-refresh`: Rapidly regenerate `session-brief.md` without a full reboot.
- `koad-eow`: End-Of-Watch utility to persist current session context to `WORKING_MEMORY.md`.
- `koad-map`: Immediate terminal-friendly output of the `doc-index.md`.

---

## 📅 Implementation Roadmap (Prioritized)

### P0: Core Hydration (Immediate Deployment)
- [ ] **Identity Extraction:** Update `koad-agent` to parse `role` and `rank` from TOML.
- [ ] **Token Hydration:** Implement `access_keys` export (Status: PARTIAL).
- [ ] **Session Brief:** Implement the "Dark Mode" filesystem-based context aggregator.
- [ ] **Prompt Anchoring:** Auto-generate `GEMINI.md` / `CLAUDE.md` based on active agent.
- **Estimated Time:** 4-6 hours.
- **Token Impact:** CRITICAL (30-40% reduction).

### P1: Structural Mapping (Advanced Discovery)
- [ ] **Doc Indexer:** Build a fast Rust-based project structural parser.
- [ ] **Handoff Logic:** Implement logic to read/rotate `EndOfWatch` summaries.
- [ ] **Git Integration:** Inject real-time git status into the shell environment.
- **Estimated Time:** 8-12 hours.
- **Token Impact:** MEDIUM (20% reduction).

### P2: Citadel Link (Post-Rebuild)
- [ ] **CASS Connectivity:** Add gRPC checks to set `KOAD_BOOT_MODE=full`.
- [ ] **Memory Uplink:** Pull semantic memories from Qdrant into the boot brief.
- **Estimated Time:** TBD (Citadel Dependent).

---

## 🛡️ Risk Assessment
- **Performance:** All boot operations MUST complete in <2s.
- **Privacy:** `access_keys` must never be logged or saved to the shared cache directory.
- **Staleness:** `session-brief.md` must be regenerated if the Git branch changes (handled via `koad-refresh`).

---
**Approval:** Awaiting Dood (Admiral) Signature.
