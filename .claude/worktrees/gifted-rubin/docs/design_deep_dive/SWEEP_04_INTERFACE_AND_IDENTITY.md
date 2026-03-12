# Design Deep Dive — Sweep 04: Interface & Identity (The Admiral's Deck)

> [!IMPORTANT]
> **Status:** PLAN MODE (Interface Design)
> **Goal:** Design the high-fidelity interaction points for the Koados Citadel. The system must provide "Professional Level Visibility" through a Web Command Deck and an immersive TUI Login Portal.

---

## 1. The TUI: Primary Command Portal
The TUI is the formal entry point for the Admiral. It is no longer an optional view; it is the "Physical Presence" of the human operator within the grid.

### **Functional Design:**
- **The Login Sequence:** Launching `koad tui` triggers an identity lease in the Spine. The Admiral is added to the `Crew Manifest` in Redis.
- **The Deck Layout:**
    - **Sector A (The Stream):** A real-time, filtered view of the Delegation Stream (Koad ↔ Sky ↔ Noti).
    - **Sector B (Grid Health):** Visual gauges for Redis, Spine, and SQLite health.
    - **Sector C (Active Personnel):** A dynamic list of who is WAKE, DARK, or COMMAND (Admiral).
    - **Sector D (The Hot-Key Console):** Quick-launch triggers for `koad doctor`, `koad system purge`, and `koad intel query`.

## 2. The Web Command Deck: Operational Visibility
The Web Deck serves as the "Station Dashboard" for long-term monitoring and low-friction interaction. It is designed to be left open in a dedicated window or secondary display.

### **Functional Design:**
- **The Observation Deck:** A persistent, high-fidelity live feed sector. It aggregates:
    - **Raw Logs (Surgically Filtered):** Real-time output from active agent sessions.
    - **The Pulse Array:** A visual heartbeat from every core service (Redis, Spine, SWS).
- **The Telemetry Array:** Real-time graphs of station resource allocation.
- **The Idea Queue:** A persistent "Input Buffer" where the Admiral can drop thoughts or issue requests that agents (like Noti or Koad) can pick up during their next wake cycle.

## 3. The "Signal Corps" Monitoring Service
To support multiple concurrent windows (Primary TUI + Dedicated Monitor), we need a "Broadcast" architecture.
- **Multi-Consumer Subscriptions:** The Signal Corps will publish to specific Redis streams (e.g., `koad:stream:logs`, `koad:stream:telemetry`). 
- **Watch Mode (`koad watch`):** A lightweight CLI/TUI mode that only subscribes to these streams. It does not acquire a full "COMMAND" lease, allowing you to have a primary interactive window and unlimited secondary "Observer" windows.
- **Role:** Aggregates raw logs and gRPC heartbeats into high-level "Signal Packets."
- **Distribution:** Publishes to a Redis `telemetry:broadcast` channel that both the Web Gateway (WebSockets) and the TUI (Async Stream) consume.

## 4. Professional-Level Tools (`koad dood`)
We will design a "Superuser" namespace for the Admiral to perform surgical station management:
- `koad dood inspect <trace_id>`: View the full gRPC/Redis/SQLite lifecycle of a specific request.
- `koad dood override <kai_name>`: Manually release a locked identity lease.
- `koad dood pulse`: Send a forced heartbeat signal to all active agents.

---
## **Refined Implementation Strategy (v5.0)**
1.  **Stateless Spine** (Authority remains in Redis).
2.  **UDS First** (Security and Ghost-prevention).
3.  **Identity-Rich Protobufs** (Admiral and Agents share the same session model).
4.  **TUI as Lead Interface** (Crate `koad-tui` is promoted to a core dependency).

---
*Next Sweep: The v5.0 Data Schema (Redis Keys & SQLite Migrations).*
