# v5.0 Interface Deck — Command & Observation

> [!IMPORTANT]
> **Core Role:** The Interface Deck provides professional-grade visibility and immersive interaction. It transforms the station from a cold terminal into a living simulation.

---

## 1. The TUI: Primary Command Portal
The TUI is the Admiral's physical presence in the Citadel.

### **Features:**
- **Identity Login:** Launching `koad tui` acquires an `Admiral` lease in Redis.
- **The Stream View:** A real-time, scrolling view of the **Event Bus** (Agent activity, SCE logs, Spine heartbeats).
- **The Hot-Key Array:** Instant access to `koad doctor`, `koad system purge`, and `koad intel query`.

## 2. The Observation Deck (Multi-Window)
Designed for secondary displays or dedicated monitoring windows.

### **Mechanism:**
- **`koad watch`:** A lightweight mode that subscribes to Redis Streams without consuming an identity lease.
- **Broadcast Streams:** The **Signal Corps** aggregates logs and telemetry into a unified broadcast.
- **Visuals:** Real-time graphs of CPU/Mem and "Pulse Lights" for every core component.

## 3. The Web Deck: Operational Dashboard
A browser-based mission control screen.

- **Purpose:** Long-term observability and low-friction idea entry.
- **The Think Tank:** An input buffer where the Admiral can queue thoughts for agents to pick up.
- **The SCE Mirror:** A visual status map of the Skylinks Cloud Ecosystem (GCP, Stripe, Airtable).

## 4. Professional Debugging (`koad dood`)
A dedicated namespace for surgical station management.

| Command | Role |
| :--- | :--- |
| `koad dood inspect <tid>` | View full lifecycle of a specific Trace ID. |
| `koad dood override <kai>` | Manually release a locked identity. |
| `koad dood pulse` | Forced station-wide heartbeat verification. |

---
*Next: [The Tool Layer — Surgical Parsers](TOOL_LAYER.md)*
