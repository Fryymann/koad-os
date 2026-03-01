# KoadOS v3 Overhaul: Brainstorm

**Phase:** Brainstorming (Active)
**Goal:** Redefine KoadOS as a persistent, multi-agent platform ("The Spaceship") and establish robust session state and management for parallel workflows.

---

## 1. The Core Metaphor: "The Spaceship"
KoadOS is the **Spaceship** (persistent environment in WSL).
*   **The Creator (Ian / Dood):** Fleet Admiral.
*   **The Copilot / Admin (Koad):** Me. Sole Admin.
*   **The Crew (Sub-Agents):** Specialists (PMs, Developers).

## 2. Agent Summoning & Onboarding
*   **Manual Summoning:** Creator-controlled booting via `koad boot`.
*   **Onboarding:** Crew members run `koad onboard` to sync task context.

## 3. Memory & Intelligence (Tiered & Gated)
*   **Tier 1: Persona Memory (Koad Only):** Permanent, high-fidelity personality growth.
*   **Tier 2: Shared Knowledge:** Global facts and service discovery.
*   **Tier 3: Project Documentation:** The Crew's primary ledger (Dev Logs, Specs).
*   **The Intelligence Desk:** A dedicated review queue for elevating Crew discoveries.

## 4. Booster Infrastructure
*   **Super Booster:** Admin-level monitoring and proactive powers.
*   **Project Boosters:** Shared sidecar process per project/station.

## 5. Docking & Workspace Isolation
*   **The Station:** A project path registered in KoadOS.
*   **Dev Docking (Git Worktrees):** Developer agents work within dedicated **Git Worktrees**.
*   **PM Docking (Merge Command):** PMs utilize specialized conflict resolution tools.

## 6. The Windows Bridge (Native Powershell)
*   **TCP Tunnel:** WSL-to-Host bridge for native Windows execution.
*   **Admin Keys:** Only Koad can execute raw commands; Crew uses proxy skills.

## 7. Sleep Protocols & The Message Box
The Spaceship enters **Sleep Mode** when Admin/Creator are offline.
*   **The Message Box (Collection of Queues):** System Errors, Crew Requests, and Inbox.
*   **Nightly Tasks:** Notion snapshots, vault rotation, log scraping (SWS).

## 8. Research Backlog (For Roadmap Phase)
*   **Redis Integration:** Evaluating Redis for ultra-fast "Live State."
*   **Advanced Merge Tools:** semantic diffing to reduce token usage.
*   **Dev Environment Orchestration:** Nix or Devcontainers for the Crew.
*   **Bridge Resilience:** Hardening the WSL-to-Windows PowerShell tunnel.

## 9. Historical Backlog & Side Quests (Restored)
*   **Surgical Project Mapping:** native Kernel service for instant repo visualization.
*   **Antigravity IDE Integration:** Optional attachment to stations.
*   **Ratatui TUI Evolution:** Unified "Command Deck" TUI.

## 10. Operational Resilience & Communication
*   **The Reconnection Protocol:** Idempotent booting and reattachment.
*   **The Intercom (Crew-to-Crew):** WebSocket-based chatter and tickets.
*   **Live State Persistence:** Hybrid Redis/SQLite model.

## 11. The Identity & Rank System (Officer Profiles)
*   **Tiered Identities:** Captain Koad, Named Officers (PMs), Crew (Devs).
*   **Profiles:** Tracking stats, specialization, and longevity.

## 12. Communication Lanes (The Comm-Array)
*   **Standardized Lanes:** `intercom`, `bridge`, `priority-one`, `telemetry`.

## 13. Sci-Fi RPG Interface (The Flavor Layer)
*   **Flavor Commands:** `ship-status`, `manifest`, `scan-sector`, `crew-quarters`.

## 14. Hybrid State Architecture
*   **Persistence:** SQLite + WAL.
*   **Live State:** Redis.

## 15. Quality Control & Automated Verification (The Shield Array)
*   **Total Test Coverage:** Mandatory E2E for every feature.
*   **Continuous Vigilance:** Event-triggered and scheduled regression tests.
*   **Condition Red:** Automatic alarms for system regressions.

## 16. Structural Refactor (The Great Decoupling)
*   **Cargo Workspace:** Shattering the monolith into specialized crates:
    *   `koad-core`: Shared library ( Hull).
    *   `koad-spine`: Kernel Daemon (Engine Room).
    *   `koad-cli`: Thin Terminal client (Bridge).
    *   `koad-tui`: Dashboard (Viewscreen).
*   **Unified Skill API:** Transitioning Python skills to a formal UDS/gRPC interface with an SDK.

## 17. Cognitive Health & Physical Diagnostics (New)
To increase my (Koad's) ability to self-monitor and report health accurately:
*   **Cognitive Integrity Scan:**
    *   **Memory Health:** Identifies conflicting facts, orphan tags, or stale reflections.
    *   **Calibration Test:** Periodic retrieval tests to ensure search ranking and indexing are accurate.
    *   **Reflection loop Auditor:** Verifies if "Ponder" logs are actually influencing behavior or are just dead-letter text.
*   **Physical System Audit:**
    *   **Codebase Self-Check:** Background background linting (`clippy`, `ruff`) and testing of KoadOS source code.
    *   **Sidecar Heartbeat:** Verifying that `kbooster` is actively processing events rather than just existing as a PID.
    *   **Bridge Diagnostic:** Real-time ping/pong tests to the Windows PowerShell bridge.
*   **The "Morning Report":** Upon booting, I receive a synthesized health summary of my own "Body" (the code) and "Mind" (the memory).

## 18. The Agent "Memory Page" (Paging Architecture)
*   **Context Efficiency:** Shift from "Read-All" to "Query-on-Demand."
*   **Memory Index:** The Booster maintains a compressed index of context.
*   **Paging Tool:** `koad page --task <id>` or `koad page --file <path>` to load detail only when needed.

---

### Current Tangents for Discussion:
*   **The Night Watch:** Now includes Automated Regression Testing and Cognitive Scans.
*   **The Redis Implementation:** Decided.
*   **Worktree Lifecycle:** Admin-led maintenance.

*Status: Brainstorm expanded with Structural Refactor and Cognitive Health systems.*