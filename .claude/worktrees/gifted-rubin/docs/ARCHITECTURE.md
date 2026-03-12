# KoadOS Architecture — The Neural Grid

This document provides a technical deep-dive into the layers and components that make up the KoadOS ecosystem.

## 📐 High-Level Topology

The system is split into three distinct planes:

1.  **Control Plane (The Command Deck)**: The `koad` CLI and the TUI Dashboard. This is where directives are issued and system telemetry is visualized.
2.  **Data Plane (The Engine Room)**: A dual-bus architecture utilizing **Redis** for hot, high-frequency state and **SQLite** for durable, structured memory.
3.  **Backbone (The Spine)**: The Rust-based gRPC server that orchestrates all inter-component communication and enforces security policies.

## 🛰 The Spine (`koad-spine`)

The Spine is the central authority. It manages:
- **Identity Registry**: Verifying the rank and role of any agent attempting to boot.
- **Context Governance**: Managing the injection and pruning of session context.
- **Safety Sandbox**: Pre-flight command evaluation to prevent unauthorized operations.
- **RPC Routing**: Connecting the CLI, the ASM, and various bridges.

## 💓 The Agent Session Manager (`koad-asm`)

The ASM is a high-availability micro-daemon dedicated to:
- **Heartbeat Monitoring**: Detecting agent "flatlines" within 30 seconds.
- **Automated Pruning**: Purging volatile memory when an agent disconnects or expires.
- **Sovereignty Enforcement**: Ensuring only one ghost (agent) is tethered to a body (session) at a time.

## 🧠 Memory & Intelligence

### **1. Hot Context (Redis)**
- Stores the immediate "Working Memory" of an agent.
- High-frequency updates during research and strategy phases.
- Facilitates sub-second "Context Hydration" during agent wake-up.

### **2. Durable Memory (SQLite)**
- Stores the "Intel Bank" (Facts, Learnings, Reflections).
- Maintains the "Project Master Map" and the "Crew Manifest".
- Acts as the cold-storage for "Cognitive Quicksaves".

## 🔌 Bridges & Integration

KoadOS utilizes modular "Bridges" to interact with external ecosystems:
- **GitHub Bridge**: Implements the **Sovereign GitHub Protocol (SGP)** for task management.
- **Notion Bridge**: Acts as the "Context Sink" for project documentation and streaming logs.
- **Cloud Bridges**: (GCP, Stripe, Airtable) Provide domain-specific tools while enforcing the **SLE/SCE Isolation Mandate**.

## 🛠 Tech Stack
- **Languages**: Rust (Core/CLI), Python (Specialized Skills).
- **Communication**: gRPC (Tonic), Unix Domain Sockets (Redis).
- **Persistence**: Redis (v9+), SQLite (v3+).
- **Observability**: OpenTelemetry-compatible tracing and structured logging.

---
*Status: ARCHITECTURE VALIDATED. Grid Online.*
