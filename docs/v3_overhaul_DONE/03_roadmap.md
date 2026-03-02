# KoadOS v3 Overhaul: Roadmap (The Flight Plan)

**Phase:** Roadmap (Active)
**Strategy:** Option A (The Big Bang / Clean Slate)
**Scope:** Kernel-level refactor to a multi-crate Cargo Workspace with gRPC/Redis hybrid state.

---

## Sprint 0: Scaffolding the Hull (The Workspace) - [COMPLETED]
*Goal: Establish the new filesystem structure and protocol contracts.*

1.  **Workspace Init:** [DONE]
2.  **Protocol Definition (`proto/`):** [DONE]
3.  **Build System:** [DONE]

## Sprint 1: The Engine Room (State & IPC) - [COMPLETED]
*Goal: Get the Kernel talking to Redis and listening for gRPC.*

1.  **Redis Lifecycle Management:** [DONE]
2.  **Redis Live State:** [DONE]
3.  **SQLite Persistence:** [DONE]
4.  **The Spine Boot:** [DONE]
5.  **Command Execution Engine:** [DONE] - Integrated shell command execution with robust PATH handling.
6.  **Dual-Connection Redis Pattern:** [DONE] - Isolated Primary (Commands) and Subscriber (PubSub) clients.

## Sprint 2: The Skill SDK (The Bridge)
*Goal: Allow Python skills to talk to the Rust Kernel.*

1.  **Python gRPC SDK:** Create a lightweight Python wrapper around the `skill.proto`.
2.  **Skill Discovery:** [IN PROGRESS] Implement the `manifest.yaml` scanner in `kspine`.
3.  **UDS Handshake:** Secure the Unix Domain Socket connection between the Kernel and local skills.

## Sprint 3: The Command Deck (CLI & TUI) - [PARTIAL]
*Goal: Replace the monolith with the new distributed clients.*

1.  **koad-cli:** Implement the thin-client that sends commands to `kspine` via gRPC.
2.  **koad-tui:** Migrate the existing Ratatui logic to the new `koad-tui` crate, subscribing to Redis for live updates.
3.  **Identity Layer:** Port the `koad.json` identity/driver logic into `koad-core`.
4.  **Web Command Deck (Vite/React):** [DONE] - Functional dashboard with telemetry feed and command console.

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
