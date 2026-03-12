# Design Deep Dive — Sweep 03: The Lean Audit ("Do We Need It?")

> [!CAUTION]
> **Status:** PLAN MODE (Technical Pruning)
> **Goal:** Ruthlessly evaluate the KoadOS architecture. Identify unused crates, redundant abstractions, and features that add complexity without corresponding value. If it isn't strictly necessary for the core mission, it gets cut.

---

## 1. Crate Level Audit

### **A. Candidates for Excision / Archival**
*   `koad-gateway`: **Why do we need it?** We initially designed it as a Web Deck UI interface. However, the Admiral operates via CLI, and Sky interacts via Notion bridges. A full Axum-based web gateway adds massive dependency bloat (`tower-http`, WebSockets) without a current, critical use case. **Verdict: Archival.**
*   `koad-tui` / `kdnd-tui`: **Why do we need it?** TUI elements look cool but they fracture the UX. The CLI should output high-signal text, not interactive UI, as interactive UI cannot be easily parsed by secondary agents. **Verdict: Archival.**
*   `koad-skill-airtable`: **Why do we need it?** It is implemented as an entire Rust crate instead of a simple script or a lightweight bridge. The "Skill as a Crate" model scales poorly. **Verdict: Refactor into `koad-bridge-airtable` or deprecate in favor of Python scripts.**

### **B. Core Crates to Retain & Simplify**
*   `koad-spine`: Keep, but strip out the internal state maps (see Sweep 02).
*   `koad-cli`: Keep, but remove all logic that attempts to bypass the Spine (e.g., direct SQLite reads). It must be a "dumb terminal" pointing to the Spine.
*   `koad-bridge-notion`: Keep. This is the correct pattern for API interaction (Surgical Parser).

---

## 2. Abstraction Audit (The Engine Room)

### **A. KoadComplianceManager (KCM)**
*   **Why do we need it?** `kcm.rs` exists to route Governance Intents (like `Clean` or `Audit`) by shelling out to Python scripts (`repo-clean.py`) or invoking the `koad` CLI from *within* the Spine.
*   **The Flaw:** This is a recursive anti-pattern. The Spine shouldn't be invoking the CLI to perform an action.
*   **Verdict: Purge.** Governance actions should be handled directly by the CLI or specific, isolated background tasks, not routed through an internal manager shelling out to itself.

### **B. AgentSessionManager (ASM) internal `Mutex`**
*   **Why do we need it?** We don't. As established in Sweep 02, Redis is the hot state. Caching Redis in an `Arc<Mutex<HashMap>>` within the Spine only creates the "Ghost Spine" desynchronization bug.
*   **Verdict: Purge.** ASM should become a collection of static methods that read/write directly to Redis. No internal state.

---

## 3. Dependency Audit

### **A. Bloat to Remove**
*   `axum`, `tower-http`, `tungstenite` (from root workspace): Removing the Gateway and WebSockets will drastically reduce compile times and binary size.
*   `ratatui`, `crossterm`: Removing TUI components further slims the CLI and enforces the "programmatic-first" text interface.

---

## 4. The v5.0 Simplification Mandate

Based on the "Do We Need It?" protocol, the v5.0 architecture will be radically simplified:

1.  **Kill the Gateway:** Remove the Web Deck concept. The Citadel is operated via CLI and external Bridge interfaces (Notion/Slack).
2.  **Kill the TUI:** The CLI is a text-stream interface.
3.  **Dumb CLI, Smart Spine:** The CLI makes gRPC calls. It does not parse SQLite or maintain its own Redis connections.
4.  **Stateless Spine:** The Spine holds *zero* internal session maps. It is merely a high-speed router between the CLI (gRPC) and the Engine Room (Redis).

---
*Next Sweep: The v5.0 Implementation Blueprint.*
