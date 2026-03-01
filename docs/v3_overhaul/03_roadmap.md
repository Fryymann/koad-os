# KoadOS v3 Overhaul: Roadmap (The Flight Plan)

**Phase:** Roadmap (Active)
**Strategy:** Option A (The Big Bang / Clean Slate)
**Scope:** Kernel-level refactor to a multi-crate Cargo Workspace with gRPC/Redis hybrid state.

---

## Sprint 0: Scaffolding the Hull (The Workspace)
*Goal: Establish the new filesystem structure and protocol contracts.*

1.  **Workspace Init:** Create root `Cargo.toml` and initialize:
    *   `crates/koad-core`: Shared types, constants, and the "Hull" traits.
    *   `crates/koad-spine`: The Kernel/Engine (gRPC Server, Redis Logic).
    *   `crates/koad-cli`: The Bridge (Thin client for command execution).
    *   `crates/koad-tui`: The Viewscreen (Ratatui dashboard).
2.  **Protocol Definition (`proto/`):** 
    *   Define `kernel.proto` (Agent-to-Kernel commands).
    *   Define `skill.proto` (Kernel-to-Skill gRPC streaming).
3.  **Build System:** Configure `build.rs` in `koad-spine` for Tonic/Prost generation.

## Sprint 1: The Engine Room (State & IPC)
*Goal: Get the Kernel talking to Redis and listening for gRPC.*

1.  **Redis Lifecycle Management:** 
    *   Install `redis-server` (apt).
    *   Create `~/.koad-os/config/redis.conf` for Koad-managed Redis (UDS enabled, local-only).
    *   Implement Redis process auto-start/stop in `koad-spine`.
2.  **Redis Live State:** Implement the `fred` client in `koad-spine`.
3.  **SQLite Persistence:** Implement the "Write-Behind" drain from Redis to `rusqlite` (WAL).
4.  **The Spine Boot:** A minimal `kspine` daemon that can start, connect to Redis, and accept a "Ping" gRPC call.

## Sprint 2: The Skill SDK (The Bridge)
*Goal: Allow Python skills to talk to the Rust Kernel.*

1.  **Python gRPC SDK:** Create a lightweight Python wrapper around the `skill.proto`.
2.  **Skill Discovery:** Implement the `manifest.yaml` scanner in `kspine`.
3.  **UDS Handshake:** Secure the Unix Domain Socket connection between the Kernel and local skills.

## Sprint 3: The Command Deck (CLI & TUI)
*Goal: Replace the monolith with the new distributed clients.*

1.  **koad-cli:** Implement the thin-client that sends commands to `kspine` via gRPC.
2.  **koad-tui:** Migrate the existing Ratatui logic to the new `koad-tui` crate, subscribing to Redis for live updates.
3.  **Identity Layer:** Port the `koad.json` identity/driver logic into `koad-core`.

## Sprint 4: The Great Migration (Feature Porting)
*Goal: Move legacy logic from the monolith to the new architecture.*

1.  **Notion Sync:** Port the Notion logic as a "Long-Running Bridge Skill."
2.  **Airtable/Vault:** Port secondary services to the new Skill API.
3.  **Cognitive Scans:** Implement the self-monitoring integrity checks.

## Sprint 5: Commissioning (Final Polish)
*Goal: Final testing and decommissioning of the v2 monolith.*

1.  **E2E Validation:** Run the full "Shield Array" regression tests.
2.  **Documentation:** Update `AGENT_INSTALL.md` and `README.md` for the v3 architecture.
3.  **Final Cleanup:** Archive the legacy `src/main.rs` and update the `nightly` branch.

---

## Operational Mandates for Development:
*   **Small Commits:** Commit after every sub-task completion.
*   **No approval required:** I (Koad) am authorized to stage and push to `nightly` autonomously.
*   **Validation First:** No logic is "done" until a test case proves it.
*   **Mandatory Sprint Review:** Every sprint must conclude with a formal "Condition Green" review covering:
    *   **Code Quality:** Adherence to Rust idioms, modularity, and gRPC contracts.
    *   **Test Coverage:** Every new module must have unit tests; critical paths must have integration tests.
    *   **Developer Documentation:** Doc-comments (`///`) and architectural updates in `docs/`.
