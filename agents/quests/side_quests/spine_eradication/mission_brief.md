# Mission Brief: Spine Eradication & System Recovery
Date: 2026-03-16
Category: Side-Quest
Status: 🟢 COMPLETE (2026-03-27)

## 1. Problem Description
The KoadOS ecosystem experienced a "terminology ghost" effect where legacy references to the "Spine" caused critical system failures and prevented agent booting.

## 2. Suggested Fix
A systematic synchronization of the codebase and a global rebuild of all system binaries to transition to architectural purity (Citadel/CASS terminology).

## 3. Implementation Plan (Side-Quest Roadmap)

### Phase 1: Source Alignment (The "Surgical" Pass)
- [x] **Citadel Core:** Update `crates/koad-citadel/src/main.rs` to use `citadel_grpc_port`.
- [x] **Sandbox:** Update hardcoded test strings in `crates/koad-sandbox/src/lib.rs`.
- [x] **Core Utils:** Transition `kspine.pid` references in `crates/koad-core/src/utils/pid.rs` to `kcitadel.pid`.

### Phase 2: Build & Protocol Cleanup (The "Hygiene" Pass)
- [x] **Proto:** Remove `spine.proto` from `crates/koad-proto/build.rs` and update `README.md`.
- [x] **Crates:** Update any remaining `README.md` files (e.g., `koad-board`) that mention the Spine.

### Phase 3: Documentation & UI Scrub (The "Clarity" Pass)
- [x] **System Map:** Finalize `SYSTEM_MAP.md` to reflect the new log and socket names.
- [x] **CLI Output:** Update user-facing strings in `whoami.rs` and other handlers.

### Phase 4: Global Rebuild & Deployment (The "Recovery" Pass)
- [x] Execute `cargo build --workspace`.
- [x] Update all binaries in `~/.koad-os/bin/`.
- [x] Verify agent boot sequence for `Tyr`, `Sky`, and `Vigil`.

---

## 🏁 Final Results (2026-03-27)
- **Binary Sync:** All 11 workspace crates recompiled and binaries updated in `bin/`.
- **Terminology:** Zero occurrences of "spine" remain in active crates or config.
- **Boot Success:** Tyr successfully booted into Jupiter Citadel with full XP hydration.
- **XP Sync:** Implemented automated seeding from identity TOMLs to Citadel SQLite.
