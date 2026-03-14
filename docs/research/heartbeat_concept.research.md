Here's a solid breakdown of the **heartbeat pattern** as it's used across software and distributed systems — framed so you can map it directly onto your Spine ↔ agent session model.

---

## What a Heartbeat Is

A heartbeat is a **periodic, lightweight signal** sent between components to indicate "I'm still alive and functioning." It's the distributed systems equivalent of a pulse check. If the signal stops arriving within a defined window, the monitoring side assumes the sender has failed or become unreachable and can take corrective action (cleanup, failover, alerting, etc.).

## Two Core Models: Push vs. Pull

**Push model** — The monitored node (your agent) actively sends a heartbeat message to the monitor (Spine) at a regular interval. The agent is responsible for broadcasting its own liveness. This is the most common pattern for daemon-style heartbeats.

**Pull model** — The monitor (Spine) periodically pings/polls the agent to check if it's alive. The agent just needs to respond. This shifts responsibility to the central system.

**For your use case** — push makes the most sense. The agent daemon emits a signal; Spine listens and tracks. If Spine has to poll every agent, you're coupling Spine's workload to agent count and adding latency to failure detection.

## Key Components of a Heartbeat System

1. **Heartbeat interval** — How often the signal is sent (e.g., every 5–30 seconds). Shorter = faster detection but more overhead.
2. **Timeout window** — How long the receiver waits before declaring the sender dead. Should always be a multiple of the interval to absorb transient misses (e.g., interval = 5s, timeout = 15s means 3 missed beats before declared dead).
3. **Failure detection logic** — Rules the receiver applies. Best practice: require **multiple consecutive misses** before declaring failure. A single missed beat could be a network hiccup, not a real crash. This avoids false positives.
4. **Acknowledgment (optional)** — Bidirectional heartbeats where the monitor ACKs receipt. This verifies both the sender's liveness *and* the network path between them. Useful if you also need the agent to know Spine is reachable.

## Common Pitfalls (Likely Relevant to Your Issues)

- **Timeout too tight** — If your timeout ≈ your interval, any momentary delay (GC pause, CPU spike, network jitter) triggers a false death declaration. Rule of thumb: timeout should be 2–3× the interval minimum.
- **Heartbeat thread not isolated** — If the heartbeat runs on the same thread/event loop as heavy agent work, a blocking operation can starve the heartbeat, making Spine think the agent died when it's just busy. The heartbeat sender should be on its **own isolated thread or process**.
- **No distinction between "dead" and "unreachable"** — A missed heartbeat could mean the agent crashed, OR the network between agent and Spine is flaky. Without bidirectional ACKs or a secondary channel, you can't distinguish these.
- **No graceful shutdown signal** — If the agent shuts down cleanly but never sends a "I'm going offline" message, Spine has to wait for the full timeout to notice. Best practice: send an explicit **deregistration/goodbye** message on clean shutdown, and reserve timeout-based detection for actual crashes.
- **Stale state after restart** — If an agent crashes and reboots fast, Spine might still be holding the old session's heartbeat state. You need a way to **invalidate the old session** and register a fresh one (session IDs help here).

## Best Practices Summary

| Principle | Detail |
| --- | --- |
| **Lightweight payload** | Just a session ID + timestamp. Don't pack business data into heartbeats. |
| **Isolated sender** | Dedicated thread/process for emitting heartbeats, decoupled from main agent workload. |
| **Tuned intervals** | Start with 5–10s interval, 15–30s timeout. Tune based on your tolerance for detection latency vs. overhead. |
| **Consecutive miss threshold** | Require 2–3 missed beats before declaring dead. |
| **Explicit deregistration** | Send a "shutting down" signal on clean exit. |
| **Session-aware** | Tie each heartbeat to a unique session ID so Spine can differentiate reboots from continuations. |
| **Monotonic sequencing** | Include a counter or generation number so Spine can detect out-of-order or duplicate beats. |

## How This Maps to Spine

Conceptually, if Spine is your central coordination layer and agents are ephemeral session-bound processes:

- Each agent boots → registers with Spine (gets or sends a session ID) → starts emitting heartbeats on interval.
- Spine maintains a **liveness table**: `{ sessionId, lastBeatTimestamp, missCount }`.
- On each interval tick, Spine checks the table. Any session past the timeout threshold gets marked inactive → triggers cleanup/rebalancing.
- On clean agent shutdown, the agent sends a deregistration message → Spine immediately marks it inactive without waiting.

This gives you a clean, centralized view of which agents are alive at any moment — which is exactly what you're after.

---

If the heartbeat daemon you've built is "not working," the most likely culprits based on the patterns above are: the heartbeat thread getting starved by agent workload, the timeout being too tight, or the session state not being properly invalidated/refreshed on agent restarts. Happy to dig into any of those angles if you want to go deeper.