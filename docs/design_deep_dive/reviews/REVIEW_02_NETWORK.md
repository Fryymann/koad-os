# Architect Review 02: Network & Boundary Integrity

**Reviewer:** Senior Systems Architect (Tyr)
**Target:** Phase 2 (Stateless Spine) & Phase 3 (Signal Corps)
**Verdict:** **YELLOW (Security & Routing Revisions Required)**

## 1. The UDS Single Point of Failure
**The Plan:** Deprecate TCP port 50051. Enforce `/home/ideans/.koad-os/kspine.sock` file-locking to prevent ghost processes.
**The Vulnerability:** UDS sockets in Linux do not automatically clean up when a process crashes (e.g., `kill -9`). If the Spine crashes hard, the `.sock` file and the `.pid` file remain on disk. When the Spine restarts, it will see the `.sock` and `.pid`, assume another process is running, and refuse to boot. This creates a "Bricked Station" scenario requiring manual intervention by the Admiral.
**The Fix:** Implement a robust "Stale Socket Sweeper" in `KernelBuilder`. At boot, it must:
1. Check if the `.pid` corresponds to a running process (`kill -0 <pid>`).
2. If the process is dead, forcefully `unlink()` the `.sock` and `.pid` files before attempting to bind.

## 2. The Protobuf Payload Bloat
**The Plan:** Replace `identity_json` with strict `AgentSession` Protobuf messages.
**The Vulnerability:** `AgentSession` contains `ProjectContext`, which contains `repeated HotContextChunk`. If an agent has a 50k token context, and `koad status` calls `GetSystemState`, we are serializing and transmitting megabytes of context data just to see who is online. This will crash the CLI with OOM or max payload limits.
**The Fix:** We must separate **Telemetry** from **Payload**. 
- `GetSystemState` should return an `AgentStatus` message (just the Name, Rank, and Heartbeat).
- A new RPC, `GetAgentContext(session_id)`, should be used *only* when the agent is actively processing a prompt, preventing massive payloads from clogging the routing layer.

## 3. The Broadcast Firehose
**The Plan:** Signal Corps aggregates everything into 500ms packets and pushes to Redis Streams for the Web Deck/TUI.
**The Vulnerability:** If Sky is executing a `npm install` and streaming stdout, and the Signal Corps blindly aggregates this, the Redis Stream will bloat massively, consuming RAM and crashing the WebSocket Gateway.
**The Fix:** Implement a **Severity & Source Filter** at the Aggregator level. `stdout` logs should be sent to a low-priority, highly-truncated stream (`koad:stream:stdout`). The main `koad:stream:events` must be strictly reserved for high-signal architectural events (Spine boot, Agent leased, Task created).
