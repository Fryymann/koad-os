# Design Deep Dive — Sweep 01: Broad System Map

> [!IMPORTANT]
> **Status:** PLAN MODE (Brainstorming)
> **Goal:** Identify high-level components and their connection points. Establish tracing requirements and state ownership.

---

## 1. The Core Metaphor: The Citadel Station
The KoadOS is not a monolithic application; it is a **Citadel-class Station**. 
- **The Engine Room (Redis):** The "Hot Memory." No Redis = No Session. This is a hard requirement.
- **The Log (SQLite):** The "Durable History." Used for long-term persistence and cold boots.
- **The Backbone (Spine):** The gRPC orchestrator. Manages the connection points and routes intents.
- **The Command Deck (CLI):** The Admiral's interface.

## 2. State Authority (Angle 1)
**Current Flaw:** State is scattered.
**Proposed Design:**
- **Redis is the single source of truth for "Live" data.** (Active sessions, task status, real-time telemetry).
- **SQLite is the single source of truth for "Durable" data.** (Agent identities, project definitions, historical logs).
- **Rust In-Memory state (Mutexes) must be ephemeral.** It should only act as a high-speed cache of Redis, never the primary authority.

## 3. Connection Points & Tracing (Angle 2)
Every handoff between systems must be traceable.
- **Trace IDs:** Every intent (command) issued by the Admiral or an Agent must be assigned a unique Trace ID at the boundary (CLI/Gateway).
- **Handoff Points:** 
    - `CLI -> Spine (UDS)`
    - `Spine -> Redis (Hot State)`
    - `Spine -> Agent (gRPC Stream)`
- **Monitoring Service:** We need to design a "Signal Corps" service that monitors these handoffs. If a handoff fails (e.g., Spine receives request but Redis is locked), the fail must be explicitly reported, not swallowed.

## 4. Native Bridges & Surgical Parsers (Angle 3)
- **Design Pattern:** Standardize the "Request -> Fetch -> Parse -> Markdown" sequence.
- **The SLE Boundary:** Sky operates in the SLE. Her interactions with the SCE (Stripe, GCP) must be channeled through Koados bridges to ensure the **Isolation Mandate** is enforced at the design level.

## 5. Resilience & Debugging (Angle 4)
- **Koad Doctor:** Needs to be a diagnostic suite that verifies every "Link" in the chain.
    - Link 1: Redis Socket.
    - Link 2: Spine gRPC (TCP/UDS).
    - Link 3: SQLite WAL integrity.
    - Link 4: File system permissions.
- **Design for Debugging:** We will design the system such that `koad status` is a "summary" but `koad inspect <trace_id>` provides the full telemetry of a specific event lifecycle.

---
*Next Sweep: Deep dive into the Engine Room (Redis/SQLite Boundary).*
