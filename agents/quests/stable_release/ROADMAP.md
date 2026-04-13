# Development Roadmap: KoadOS v3.2.0 "Citadel Integrity"

This roadmap translates the high-level v3.2.0 Agenda into actionable sprints and task outlines. It serves as the foundation for generating detailed, spec-driven task manifests for delegation to the KoadOS crew.

## Phase 1: "Sanctuary Alignment" (Distribution Readiness) [COMPLETE]
**Goal:** Ensure the codebase is entirely portable and can be safely booted on any new Linux/WSL environment without manual path patching.

*   **Task 1.1: The Great Path Scrub (Audit & Refactor) [COMPLETE]**
    *   *Outline:* Systematically hunt down and replace all hardcoded absolute paths (e.g., `/home/ideans/`) within the `crates/` and `config/` directories. All file operations must dynamically resolve relative to `$KOAD_HOME` or `$KOADOS_HOME`.
*   **Task 1.2: Bootstrap Idempotency [COMPLETE]**
    *   *Outline:* Refactor `scripts/install.sh` and `koad system init` to ensure they safely handle repeated executions. It must dynamically set up the `.env` file, scaffold required data directories (`data/db`, `logs`, `run`), and initialize the basic identity TOMLs without relying on user-specific paths.
*   **Task 1.3: The Admiral's Guide (Documentation) [COMPLETE]**
    *   *Outline:* Rewrite the core onboarding documents (`MISSION.md` and `AGENTS.md`) to reflect the new, portable boot process. Provide clear, step-by-step instructions for a new user cloning the repository for the first time.
*   **Task 1.4: Crew Manifest Standardization [COMPLETE]**
    *   *Outline:* Decouple the "Instance" crew manifest from the "Distribution" template. Redact specific names from `agents/crews/TEMPLATE.md` and ensure instance manifests are gitignored.
*   **Task 1.5: Hierarchical Workspace Deployment [COMPLETE]**
    *   *Outline:* Implement `koad deploy station` and `koad deploy outpost` subcommands. These commands scaffold the necessary directory structure (`data`, `docs`, `config`, `updates`) and create hidden support folders (`.koados-station/outpost`) for mission-specific agent context and quests, while maintaining core identities at the Citadel level.

## Phase 2: "Zero-Ghost Policy" (Robustness & Stability) [COMPLETE]
**Goal:** KoadOS must elegantly handle process lifecycle events, preventing orphaned sockets, stale PIDs, and database locks.

*   **Task 2.1: Autonomic Recovery (The Doctor is In) [COMPLETE]**
    *   *Outline:* Expand the `koad doctor` command (`koad-cli/src/handlers/status.rs`). Implement automated fix logic to safely delete stale Unix sockets (`run/*.sock`), clear corrupted Redis state keys, and verify database integrity before a fresh boot.
*   **Task 2.2: Graceful Service Lifecycle [COMPLETE]**
    *   *Outline:* Enhance the `Kernel` in `koad-citadel/src/kernel.rs`. Ensure that `tokio::signal::ctrl_c()` and SIGTERM events trigger a coordinated teardown: closing gRPC listeners, flushing the `StorageBridge` to SQLite, and formally releasing agent bay locks.
*   **Task 2.3: gRPC Error Boundary Polish [COMPLETE]**
    *   *Outline:* Audit the error mapping between `koad-agent` CLI and `koad-citadel`. Instead of raw `tonic::Status` dumps, intercept connection failures and provide actionable guidance (e.g., "Citadel offline. Run `koad start` to ignite the kernel.").

## Phase 3: "Vault Phase 3" (Skill Standardization) [COMPLETE]
**Goal:** Formalize how agents load and execute specialized tools (Skills) dynamically.

*   **Task 3.1: Skill Blueprint Architecture [COMPLETE]**
    *   *Outline:* Define the schema for a "Skill Blueprint" vs. a "Skill Instance." Update `koad-core/src/config.rs` to parse skill manifests correctly, allowing agents to "equip" skills dynamically from the central `skills/` directory into their KAPV.
*   **Task 3.2: `koad vault skill` Implementation [COMPLETE]**
    *   *Outline:* Complete the stubbed `VaultAction::Skill` in `koad-cli`. Build the CLI interface for listing globally available skills, inspecting a skill's capabilities, and syncing a skill into the current agent's active memory.

## Phase 4: "Final Polish & Release" [ACTIVE]
**Goal:** Clean up technical debt, ensure CI/CD compliance, and merge to `main`.

*   **Task 4.1: Workspace Audit & Canon Compliance [ASSIGNED: CID]**
    *   *Outline:* Resolve all `cargo clippy` warnings and eliminate dead code using the knowledge graph. (See [TASK_4_1_LINT_AUDIT.md](./tasks/TASK_4_1_LINT_AUDIT.md))
*   **Task 4.3: The Distribution Sanitizer (`koad-scrub`) [READY]**
    *   *Outline:* Implement `koad system scrub` to purge instance-specific data for distribution. (See [TASK_4_3_SANITIZER.md](./tasks/TASK_4_3_SANITIZER.md))
*   **Task 4.2: Nightly Bridge & Release Cut [PLANNED]**
    *   *Outline:* Changelog generation, version orchestration, and the final merge to `main`. (See [TASK_4_2_RELEASE_CUT.md](./tasks/TASK_4_2_RELEASE_CUT.md))