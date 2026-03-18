# Mission Brief: Spine Eradication & System Recovery
Date: 2026-03-16
Category: Side-Quest
Status: Problem and 

## 1. Problem Description
The KoadOS ecosystem is currently experiencing a "terminology ghost" effect. While the "Spine" was conceptually retired in favor of the "Citadel," legacy references to the Spine persist in the source code, build scripts, and documentation. This is not merely a naming issue; it has caused a critical system failure: **Agents are unable to boot.**

### The Boot Failure
Current boot attempts fail with:
`Error: missing field spine_grpc_port`

This error occurs because:
1.  The `KoadConfig` schema was partially updated, but some source code (notably `koad-citadel`) still attempts to access legacy fields.
2.  The `koad-agent` binaries currently installed in `~/.koad-os/bin/` are out of sync with the latest configuration schema. They were compiled against an older version of the codebase that expects `spine_grpc_port`, while the active `kernel.toml` has been updated to `citadel_grpc_port`.

## 2. Suggested Fix
A systematic synchronization of the codebase and a global rebuild of all system binaries are required. We must transition from "backward compatibility shims" to "architectural purity."

### Architectural Goal
- Remove all `#[serde(alias = "spine_...")]` shims.
- Ensure all source code references the `Citadel` and `CASS` terminology exclusively.
- Standardize all file paths (logs, sockets, PIDs) to the `kcitadel.*` naming convention.

## 3. Implementation Plan (Side-Quest Roadmap)

### Phase 1: Source Alignment (The "Surgical" Pass)
-   **Citadel Core:** Update `crates/koad-citadel/src/main.rs` to use `citadel_grpc_port`.
-   **Sandbox:** Update hardcoded test strings in `crates/koad-sandbox/src/lib.rs`.
-   **Core Utils:** Transition `kspine.pid` references in `crates/koad-core/src/utils/pid.rs` to `kcitadel.pid`.

### Phase 2: Build & Protocol Cleanup (The "Hygiene" Pass)
-   **Proto:** Remove `spine.proto` from `crates/koad-proto/build.rs` and update `README.md`.
-   **Crates:** Update any remaining `README.md` files (e.g., `koad-board`) that mention the Spine.

### Phase 3: Documentation & UI Scrub (The "Clarity" Pass)
-   **System Map:** Finalize `SYSTEM_MAP.md` to reflect the new log and socket names.
-   **CLI Output:** Update user-facing strings in `whoami.rs` and other handlers.

### Phase 4: Global Rebuild & Deployment (The "Recovery" Pass)
-   Execute `cargo build --workspace`.
-   Update all binaries in `~/.koad-os/bin/`.
-   Verify agent boot sequence for `Tyr`, `Sky`, and `Vigil`.

---
**Admiral Review Required.**
*Halt execution. Standing by for side-quest authorization.*
