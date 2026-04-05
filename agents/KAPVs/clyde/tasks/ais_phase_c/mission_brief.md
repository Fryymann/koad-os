# Mission Brief: AIS Phase C (Infrastructure Resilience)
**Status:** ✅ COMPLETE
**Lead:** Clyde (Officer)
**Objective:** Implement Graceful Degradation (Offline Mode) and stabilize systemd/Docker service dependencies.

## Ⅰ. Background
The Citadel gRPC grid is currently a hard blocker for agent booting. If services are offline, agents cannot boot to fix them. AIS Phase C introduces a "Degraded Mode" for `koad-agent boot` and stabilizes the underlying infrastructure.

## Ⅱ. Strategic Roadmap (SDD)
- [x] **Phase 1: Boot Degradation Logic** (Tyr) — Implemented `boot.rs` failure handling.
- [x] **Phase 2: Review & Outage Simulation** (clyde-qa) — Verify resilience and fix Async Safety.
- [x] **Phase 3: Infrastructure Stabilization** (clyde-dev) — Docker/Qdrant health checks and systemd unit hardening.

## Ⅲ. Team Task Packets

### **clyde-qa (Verification & Audit)**
- **Task ID:** `clyde-20260403-qa-01`
- **Objective:** Fix "Async Safety" warnings in `boot.rs` and verify Degraded Mode via outage simulation.
- **Context:** `crates/koad-cli/src/handlers/boot.rs`, `docs/protocols/RUST_CANON.md`.
- **Requirements:** 
    - Use `tokio::fs` or `spawn_blocking` for file I/O in `boot.rs`.
    - Stop systemd services (`sudo systemctl stop koad-citadel koad-cass`).
    - Verify `agent-boot` succeeds with `[DEGRADED MODE]` warning.

### **clyde-dev (Infrastructure Stabilization)**
- **Task ID:** `clyde-20260403-dev-01`
- **Objective:** Stabilize the Docker/Qdrant dependency chain for CASS.
- **Context:** `config/systemd/koad-cass.service`, `config/kernel.toml`.
- **Requirements:** 
    - Create `scripts/verify-services.sh` to check for Qdrant readiness (port 6333).
    - Update `koad-cass.service` to depend on Docker/Qdrant availability.
    - Ensure `koad system start` reliably spins up the Qdrant container if missing.

## Ⅳ. Governance & Reporting
- Update `TEAM-LOG.md` in the project root after each task completion.
- Escalate any security denials or sovereignty blockers to `ESCALATIONS.md`.
- Final check: Ensure zero unwraps or expects are introduced.

---
*Authorized by Tyr (Captain) | 2026-04-03*
