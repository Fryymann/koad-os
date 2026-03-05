# Design Deep Dive — Sweep 06: Contracts & Signal Corps

> [!IMPORTANT]
> **Status:** PLAN MODE (Systems Integration)
> **Goal:** Eradicate the "JSON-in-gRPC" anti-pattern. Establish a strictly typed neural bus and design the Signal Corps for real-time station-wide observability.

---

## 1. Type Sovereignty: The v5.0 Protobuf Contracts
We are moving away from `string identity_json` and `string project_context_json`. Every core entity is now a first-class Protobuf message.

### **A. Identity & Session Messages**
```protobuf
message Identity {
  string name = 1;
  Rank rank = 2;
  repeated string permissions = 3;
  int32 tier = 4;
}

message ProjectContext {
  string project_name = 1;
  string root_path = 2;
  repeated string allowed_paths = 3;
  repeated string stack = 4;
}

message AgentSession {
  string session_id = 1;
  Identity identity = 2;
  ProjectContext context = 3;
  SessionStatus status = 4;
  google.protobuf.Timestamp last_heartbeat = 5;
  string trace_id = 6;
}
```

### **B. Refactored RPCs**
- `rpc GetSystemState(GetSystemStateRequest) returns (GetSystemStateResponse);`
    - `GetSystemStateResponse` will now return `repeated AgentSession active_sessions` instead of a JSON string.
- `rpc InitializeSession(InitializeSessionRequest) returns (AgentSession);`
    - Returns the full session object natively.

---

## 2. The Signal Corps: Station Broadcast Service
The Signal Corps is a high-priority background task within the Spine (or a standalone micro-daemon) responsible for aggregating internal events and broadcasting them to the **Observation Deck**.

### **A. Architecture:**
1.  **Aggregator:** Listens to internal gRPC heartbeats, Redis keyspace events, and Agent log streams.
2.  **Packetizer:** Groups these signals into 500ms "Signal Packets."
3.  **Broadcaster:** Pushes packets to Redis Streams (`koad:stream:*`).

### **B. Stream Map:**
- **`koad:stream:telemetry`**: Resource stats (CPU/MEM) + Spine Link Health.
- **`koad:stream:logs`**: Unified logs from Koad, Sky, Noti, and the Spine.
- **`koad:stream:events`**: Human-readable station events (e.g., "Admiral Ian entered Command Deck", "Sky initiated SWS Mapping").

---

## 3. The UDS Mandate (Local Sovereignty)
To prevent ghost processes and unauthorized interception:
- **Default Port:** TCP `50051` is **DEPRECATED** for local CLI use.
- **Enforcement:** The `koad` CLI will *only* attempt to connect via `/home/ideans/.koad-os/kspine.sock`.
- **Locking:** The Spine will acquire a file-lock on `kspine.sock`. If the lock is held, the Spine fails to start, preventing duplicate "Ghost Spines."

---

## 4. The "Signal-to-Noise" Filter (Agent Context)
The Signal Corps also manages what an agent "sees."
- **Context Packets:** Instead of dumping raw logs into an agent's context, the Signal Corps provides a "Summary Packet" of recent station activity.
- **Token Efficiency:** This ensures Sky knows what Koad is doing without re-reading thousands of lines of raw telemetry.

---
## **Refined Implementation Strategy (v5.0)**
1.  **Update `spine.proto`** with native message types.
2.  **Refactor `koad-cli`** to use these types (removing `serde_json` from the hot path).
3.  **Implement `kspine.sock` locking** in `KernelBuilder`.
4.  **Launch Signal Corps** as a core `Engine` component.

---
*Next Sweep: The v5.0 Migration Path & Boot Sequence.*
