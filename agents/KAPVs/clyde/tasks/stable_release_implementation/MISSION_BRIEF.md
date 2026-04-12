# Mission Brief: KoadOS v3.2.0 "Citadel Integrity"
**Commander:** Tyr (Project Manager)
**Assignee:** Clyde (Implementation Lead)
**Status:** 🟠 Sprint 2 Active

---

## 🛰️ Strategic Context
Clyde, excellent work on the Sanctuary Audit and the Bootstrap hardening. Phase 1 infrastructure is now stable and portable. We are moving into the "Polish & Reliability" stage of the release.

## 🗺️ Canonical Planning References
- **Master Agenda:** `~/.koad-os/agents/quests/stable_release/AGENDA.md`
- **Development Roadmap:** `~/.koad-os/agents/quests/stable_release/ROADMAP.md`
- **Task Manifests:** `~/.koad-os/agents/quests/stable_release/tasks/`

---

## 📋 Assigned Tasks & Implementation Sequence

### **Phase 1: Sanctuary Alignment (FINALIZATION)**
1.  **[1.1] The Great Path Scrub:** ✅ **DONE**
2.  **[1.2] Bootstrap Idempotency:** ✅ **DONE**
3.  **[1.4] Crew Manifest Standardization:** (Priority: Alpha | **CURRENT FOCUS**)
    - Redact `agents/crews/TEMPLATE.md`. Use generic placeholders for distribution.
    - Ensure `agents/crews/CITADEL_JUPITER.md` contains our current crew for local use.
4.  **[1.3] The Admiral's Guide:** (Priority: Beta | *Can be delegated to Scribe*)
    - Modernize documentation for v3.2.0.

### **Phase 2: Zero-Ghost Policy (INITIATION)**
5.  **[2.2] Graceful Service Lifecycle:** (Priority: Alpha | **CURRENT FOCUS**)
    - Coordinated shutdown and state drain for Citadel/CASS. This is the blocker for the Autonomic Recovery engine.
6.  **[2.1] Autonomic Recovery (The Doctor is In):** (Priority: Alpha)
    - Implement the `--fix` logic for `koad doctor`.
7.  **[2.3] gRPC Error Boundary Polish:** (Priority: Beta)

---

## ⚠️ Known Blockages & Dependencies
- **Task 2.2** (Graceful Shutdown) must be verified before **Task 2.1** (Doctor) can reliably distinguish between a "crashed" process and a "cleanly stopped" one.

## 🚀 Execution Instructions
1.  Focus on finishing Phase 1 (1.4 and 1.3) before going deep into Phase 2.
2.  Continue using the `refactor/` and `feature/` branch patterns.
3.  Provide a status update in your `TEAM-LOG.md` after completing each Phase.
