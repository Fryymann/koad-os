# Design Deep Dive — Sweep 05: Data Schema (v5.0 Intention)

> [!IMPORTANT]
> **Status:** PLAN MODE (Data Engineering)
> **Goal:** Define a high-performance, strongly-typed data schema. We must eliminate "Split-Brain" state by making Redis the absolute authority for live operations and SQLite the source of truth for durability.

---

## 1. The Engine Room: Redis (Hot State Authority)
Redis is no longer a mirror; it is the **Authority**. The Spine will be stateless and query these keys directly.

### **A. Session & Identity Keys**
- `koad:sessions:{session_id}` (Hash): 
    - `identity`: Serialized Agent object.
    - `status`: `WAKE` | `DARK` | `COMMAND` (Admiral).
    - `last_heartbeat`: Unix timestamp.
    - `trace_id`: The ID of the current/last active request.
- `koad:identities:leases` (Hash):
    - Key: `{agent_name}`
    - Value: `{session_id}` (Acts as the identity lock).

### **B. Task & Intent Keys**
- `koad:tasks:{task_id}` (Hash):
    - `state`: Serialized Task object.
    - `trace_id`: Link to the initiating request.
- `koad:intents:queue` (List/Stream):
    - High-speed buffer for commands from the CLI to the Spine.

### **C. The Broadcast Streams (Multi-Consumer)**
We will use **Redis Streams** to support multiple interactive and monitoring windows.
- `koad:stream:telemetry`: System stats (CPU, Mem, Disk) broadcasted every 5s.
- `koad:stream:logs`: Aggregated logs from all agents and the Spine.
- `koad:stream:delegation`: High-level events (e.g., "Sky started mapping SWS").

---

## 2. The Log: SQLite (Durable Persistence)
SQLite is the **Durable Source of Truth**. The Spine will run migrations at boot to ensure schema integrity.

### **A. Core Tables**
- `identities`: Persistent agent configurations (Name, Rank, Bio, Model Tier).
- `projects`: Definitions of project roots and allowed paths.
- `knowledge`: The collective memory bank (Facts, Learnings, Ponderings).
- `audit_trail`: A long-term record of every `trace_id`, the actor, and the outcome.

### **B. The Schema Enforcement Mandate**
The v5.0 Spine will include a `SchemaManager` that:
1. Verifies the existence of all tables at startup.
2. Performs atomic migrations if the version in `koad.json` exceeds the database version.
3. Fails the boot sequence if the database is corrupted or inaccessible.

---

## 3. The Trace ID Lifecycle
Every interaction must be traceable from the Command Deck to the Engine Room.

1. **Generation:** The CLI generates a `trace_id` (e.g., `TRC-XXXX-XXXX`) at invocation.
2. **Propagation:** The ID is passed in the gRPC metadata.
3. **Execution:** The Spine logs all Redis/SQLite operations using this `trace_id`.
4. **Reporting:** If a failure occurs, the Admiral can run `koad dood inspect TRC-XXXX-XXXX` to see the exact point of failure.

---

## 4. Summary of Shifts
| Feature | v4.x (Current) | v5.0 (Intended) |
| :--- | :--- | :--- |
| **Session State** | Mutex<HashMap> in Spine | Direct Redis Hash reads |
| **Identity Lock** | Fragile memory lease | Atomic Redis Lease with TTL |
| **Schema** | Accidental/Manual | Enforcement via Spine |
| **Observability** | Passive Status | Real-time Streams |
| **Tracing** | Log strings only | Formalized Trace IDs |

---
*Next Sweep: The Signal Corps & gRPC Contract Refactor.*
