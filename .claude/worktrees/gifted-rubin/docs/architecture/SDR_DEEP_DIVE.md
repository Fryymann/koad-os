# Strategic Design Review: KoadOS Architecture Deep Dives

> [!CAUTION]
> **Status: ACTIVE PRIORITY (CRITICAL)**
> The current system exhibits architectural friction (e.g., gRPC state desynchronization, ghost processes, locking deadlocks). Before proceeding with further feature implementation, we are executing a series of foundational deep dives to solidify the Koados Citadel's design.

## The Goal
To thoroughly define, map, and validate the intended architecture of KoadOS. We must eliminate "just-in-case" complexity, resolve state-ownership ambiguities, and ensure the system can cleanly support multiple concurrent agents (like Koad and Sky) and complex sub-systems (like the SLE).

## Deep Dive Angles

### 1. The Runtime & State Authority (The Engine Room)
*   **The Problem:** State is currently fragmented across SQLite, Redis, and in-memory Rust `Arc<Mutex<HashMap>>` (like ASM). This fragmentation leads to desynchronization (e.g., `GetSystemState` returning `{}` while the log says otherwise). Ghost processes cling to old state.
*   **The Focus:** 
    *   Single Source of Truth: Defining exactly what lives in Redis (hot state) vs. SQLite (durable state).
    *   Concurrency: Moving away from heavy `Mutex` locking towards message-passing (Actor model) or strict Redis-driven state.
    *   Process Lifecycle: Guaranteeing atomic startups and clean shutdowns to eradicate ghosts.

### 2. Agent Orchestration & The Neural Bus (The Spine)
*   **The Problem:** The gRPC interface is cumbersome. The overhead of JSON-in-gRPC stringification defeats the purpose of strong typing. The ASM is struggling to maintain accurate "Wake" vs "Dark" states.
*   **The Focus:**
    *   Strict Protobuf Contracts: Eliminating `string identity_json` in favor of strongly typed messages.
    *   Session Hydration: How context (like the 50k token limit) is efficiently packed and delivered to the driver.
    *   UDS vs TCP: Fully transitioning local CLI-to-Spine communication to Unix Domain Sockets for security and ghost-prevention.

### 3. Boundaries & Integrations (Bridges)
*   **The Problem:** MCP tools proved to be token sinks. We are pivoting to native Rust implementations (like `koad-bridge-notion`).
*   **The Focus:**
    *   The "Surgical Parser" Pattern: Standardizing how external data (Notion, Airtable, GCP) is compressed into high-signal Markdown before hitting the context window.
    *   The SLE Interface: How Sky interacts with the SCE safely via the Koados.

### 4. Resilience & Diagnostics (The Doctor & Watchdog)
*   **The Problem:** When the system fails, it fails silently or returns misleading data (`Total Wake Personnel: 0`). The Autonomic Sentinel tries to heal but often fights the database.
*   **The Focus:**
    *   Implementing a robust `koad doctor` command for deep E2E diagnostics.
    *   Heartbeat refactoring: Ensuring deadmen switches accurately prune dead sessions without killing active CLI invocations.

---
*Prepared by Captain Koad for Admiral Ian.*