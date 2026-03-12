# v5.0 Implementation Strategy: Phases & Visibility

> [!IMPORTANT]
> **Status:** PLAN MODE (Sprint Initialization)
> **Goal:** Define the sequential development phases for the v5.0 overhaul. A core mandate is "Visibility First"—we must build the diagnostic tools and tracing hooks *before* we refactor the core systems.

---

## The "Visibility First" Mandate
Before we dismantle the current `koad-spine` or `AgentSessionManager`, we must construct a diagnostic harness. We need to see the "blood flow" of the system clearly so that when we refactor, we can instantly identify blockages.

### **Tracing & Hooking Architecture**
1.  **The Trace Context:** Every function signature in the core Rust crates will be updated to accept a `TraceContext` struct. This context holds the `trace_id` and the `origin_component` (e.g., CLI, Spine, Chassis).
2.  **The Event Exhaust:** We will implement a macro (e.g., `emit_signal!(ctx, level, msg)`) that automatically publishes to the `koad:stream:logs` Redis stream, prepended with the `trace_id`.
3.  **The Dood Probe (`koad dood inspect <trace_id>`):** A development tool that queries Redis Streams for a specific `trace_id` and reconstructs the exact lifecycle of a failed command across all systems.

---

## Phase 0: The Diagnostic Harness (Building the Flashlight)
*Objective: Build the tools we need to debug the v5.0 overhaul.*

1.  **Refactor Logger:** Update the `koad-core` logging to push directly to Redis Streams instead of just stdout/files.
2.  **The Pulse Command:** Implement `koad dood pulse`. This sends a ping down the gRPC bus to Redis and back, returning the round-trip latency. It's our primary "Is it alive?" check.
3.  **The Raw Watcher:** Implement `koad watch --raw`. A simple TUI that subscribes to all `koad:stream:*` channels and streams them to the terminal. We will leave this running in a secondary window during all subsequent phases.

## Phase 1: The Engine Room (State Sovereignty & Caching)
*Objective: Establish Redis as the absolute authority and SQLite as the enforced durable log.*

1.  **Schema Enforcement:** Build the `SchemaManager` in `koad-spine` to execute `.sql` migrations at boot. Include the new `intake_queue` table for async ideas.
2.  **Redis Lua Guards:** Write the Lua scripts for atomic Lease Acquisition and Session Status updates. Ensure Leases have a strict 5-second TTL during boot.
3.  **Event-Driven Cache:** Rewrite `AgentSessionManager`. It must NOT query Redis on every request. It must subscribe to `__keyspace@0__:koad:sessions:*` and maintain a synchronized, read-only cache.

## Phase 2: The Stateless Spine & Strict Contracts
*Objective: Eradicate JSON-in-gRPC and lock down the socket safely.*

1.  **Protobuf Refactor:** Rewrite `spine.proto`. Separate heavy `AgentSession` payloads from lightweight `AgentStatus` telemetry to prevent OOM errors.
2.  **UDS Lockdown & Sweeper:** Modify `KernelBuilder` to enforce file-locks on `kspine.sock`. **CRITICAL:** Implement the `Stale Socket Sweeper` to `unlink()` dead sockets after a hard crash.
3.  **CLI Refactor:** Update the `koad` CLI to serialize/deserialize using the new strict Protobuf types.

## Phase 3: The Signal Corps & Observation Deck
*Objective: Build the professional-level telemetry aggregators.*

1.  **Signal Corps Task:** Create the background loop. **CRITICAL:** Implement Severity & Source Filters to prevent noisy `stdout` from crashing the Web Deck.
2.  **The TUI Dashboard:** Upgrade `koad watch` from a raw stream viewer to a formatted Dashboard (Sector A, B, C design).

## Phase 4: The Agent Chassis & Workspace Manager
*Objective: Implement physical Git isolation and the formal Docking Protocol.*

1.  **Workspace Manager:** Implement `git worktree` spawning. **CRITICAL:** Implement the `WorkspaceSweeper` to auto-prune worktrees abandoned for >72 hours.
2.  **Command Filter Interceptor:** Update the `Sandbox` to actively block path traversal (`cat`, `grep`) to sensitive directories for Officer-rank agents, ensuring true Sandbox Isolation.
3.  **Chassis State Machine:** Wire the `DOCKING -> HYDRATING -> ACTIVE -> TEARDOWN` states.
4.  **Async Brain Drain:** Implement the Redis `koad:intents:knowledge` queue so agents don't hang waiting for SQLite disk writes during Teardown.

---

## Summary for the Overhaul
By executing **Phase 0** first, we guarantee that during Phases 1-4, if anything breaks, we have the precise `trace_id` and real-time Redis streams to tell us exactly which line of code failed. We stop guessing and start measuring.
