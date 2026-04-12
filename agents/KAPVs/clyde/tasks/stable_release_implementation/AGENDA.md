# Clyde's Agenda: v3.2.0 Implementation (Sprint 2)
**Mission:** Stable Release Push
**Status:** 🟠 Sprint 2 Active

---

## 🛰️ PHASE 1: SANCTUARY ALIGNMENT
- [x] **[1.1] The Great Path Scrub**
- [x] **[1.2] Bootstrap Idempotency**
- [ ] **[1.4] Crew Manifest Standardization**
    - [ ] Redact `agents/crews/TEMPLATE.md` (remove Tyr, Clyde, etc.)
    - [ ] Update `agents/crews/CITADEL_JUPITER.md` with active crew
- [ ] **[1.3] The Admiral's Guide**
    - [ ] Update `README.md` Quick Start
    - [ ] Update `AGENTS.md` and `MISSION.md` architecture details (delegable to Scribe)

---

## 🛰️ PHASE 2: ZERO-GHOST POLICY
- [ ] **[2.2] Graceful Service Lifecycle**
    - [ ] Implement `SIGINT`/`SIGTERM` trapping in `main.rs`
    - [ ] Implement `Kernel::shutdown` with `storage.drain_all()`
- [ ] **[2.1] Autonomic Recovery (The Doctor is In)**
    - [ ] Implement stale socket removal
    - [ ] Implement `koad:state` reconciliation
- [ ] **[2.3] gRPC Error Boundary Polish**

---

## 🛰️ PHASE 3: VAULT PHASE 3
- [ ] **[3.1] Skill Blueprint Architecture**
- [ ] **[3.2] Vault Skill CLI Implementation**

---

## 🛰️ PHASE 4: FINAL POLISH
- [ ] **[4.1] Workspace Lint & Audit**
- [x] **[4.3] The Distribution Sanitizer** (Tool Implemented & Verified by Tyr)
- [ ] **[4.2] Nightly Bridge & Release Cut**
