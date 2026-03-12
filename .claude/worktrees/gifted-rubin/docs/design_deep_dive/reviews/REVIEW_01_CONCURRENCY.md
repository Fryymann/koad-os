# Architect Review 01: Concurrency & State Integrity

**Reviewer:** Senior Systems Architect (Tyr)
**Target:** Phase 1 (Engine Room) & Phase 4 (Agent Chassis)
**Verdict:** **YELLOW (Proceed with Caution / Revisions Required)**

## 1. The "Stateless Spine" Fallacy
**The Plan:** The Spine drops its internal `HashMap` and queries Redis for every request.
**The Vulnerability:** We are replacing memory synchronization bugs with *network latency* and *connection pool exhaustion*. If Sky, Tyr, and the Web Deck are all polling the Spine simultaneously, and the Spine must query Redis for every single `GetSystemState` request, we will choke the Redis connection pool. 
**The Fix:** The Spine cannot be purely stateless. It must employ an **Event-Driven Cache**. The Spine subscribes to Redis keyspace notifications (`__keyspace@0__:koad:sessions:*`). It maintains a local read-only cache that is updated *only* when Redis pushes a change. This provides zero-latency reads while maintaining Redis as the ultimate authority.

## 2. The Atomic Lease Condition
**The Plan:** Lua scripts handle lease acquisition to prevent the `IDENTITY_LOCKED` bug.
**The Vulnerability:** What happens if the Spine crashes *after* acquiring the lease but *before* spawning the agent process? The Lua script acquired the lock, but the agent is dead. The next boot attempt will hit the exact same `IDENTITY_LOCKED` error.
**The Fix:** Leases must have a strict TTL (Time-To-Live) that is shorter than the boot sequence (e.g., 5 seconds). The agent process, once alive, must immediately take over the heartbeat to extend the TTL. If the boot fails, the lease auto-expires, preventing ghost locks.

## 3. The "Brain Drain" Deadlock
**The Plan:** An agent cannot release its lease until its memory is written to SQLite.
**The Vulnerability:** If the SQLite file is locked by another process (e.g., a backup script), the `TEARDOWN` phase hangs indefinitely. The agent process stays alive, holding the Redis lease, waiting for SQLite.
**The Fix:** Implement an asynchronous "Write-Ahead" queue in Redis (`koad:intents:knowledge`). The agent dumps its learnings into Redis and dies immediately. A dedicated Spine background task (The Log Keeper) slowly drains this queue into SQLite. Never block an agent teardown on a slow disk write.