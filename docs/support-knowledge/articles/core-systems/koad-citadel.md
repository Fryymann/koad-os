# koad-citadel: The Core OS Kernel

> The persistent gRPC server that forms the "Body" of KoadOS — managing all agent sessions, enforcing security, controlling resource access, and orchestrating the operating environment.

**Complexity:** advanced
**Related Articles:** [The Tri-Tier Model](../architecture/tri-tier-model.md), [Agent Session Lifecycle](./agent-session-lifecycle.md), [koad-agent boot](./koad-agent-boot.md), [koad-cass](./koad-cass.md)

---

## Overview

`koad-citadel` is the kernel of KoadOS. In the [Tri-Tier Model](../architecture/tri-tier-model.md), it occupies the "Body" position: the infrastructure layer that all agents depend on for their physical existence within the system. It is a long-running, multi-service gRPC server built on `tonic` and `tokio`, and it is the single source of truth for all active session state.

Every action an agent takes that affects shared state passes through the Citadel. When an agent boots, the Citadel issues the session lease. When an agent sends a command, the Citadel validates it. When an agent acquires a lock, the Citadel manages it. When an agent's session expires, the Citadel purges it. Nothing happens in KoadOS without the Citadel knowing about it.

The Citadel's security model is **Zero-Trust**: every single gRPC call to any Citadel service is intercepted and validated before reaching the service handler. There are no "internal" calls that bypass authentication. An agent that has lost or forged its session token cannot do anything privileged.

The Citadel is deliberately focused on infrastructure concerns. It does not perform AI inference, does not manage long-term memory, and does not generate context. Those responsibilities belong to [CASS](./koad-cass.md). The Citadel is fast, low-latency, and operates primarily against Redis — a conscious design choice to keep the session and security path as lightweight as possible.

## How It Works

### Startup and Assembly

The `koad-citadel` binary starts in `main.rs`. It loads `config/kernel.toml`, initializes the shared infrastructure components (Redis client, SQLite storage bridge, Sandbox), and assembles the gRPC server using the `Kernel` struct in `kernel.rs`.

The `Kernel` initializes all service handlers and configures the `tonic` server with the gRPC interceptor. It also starts background tasks (the session reaper, the Redis state drain loop). Once started, it listens on the configured address (e.g., `127.0.0.1:50051`) until signalled to shut down.

The key shared resources, all wrapped in `Arc` for safe concurrent access:
- `RedisClient` — shared across all services for session state and locking
- `Sandbox` — shared by the Sector service for command validation
- `BayStore` — manages agent bay metadata
- `WorkspaceManager` — handles git worktree provisioning

### The Zero-Trust Interceptor

Before any request reaches a service handler, it passes through the `tonic` interceptor built by `build_citadel_interceptor()` in `auth/interceptor.rs`. The interceptor:

1. Extracts the `KOAD_SESSION_TOKEN` from the gRPC request metadata
2. Validates the token against the active session cache in Redis
3. If valid: injects the `SessionRecord` into the request context and forwards the call
4. If invalid or missing: immediately returns `Unauthenticated` — the service handler never runs

This is called on **every** gRPC call, including heartbeats. There is no way to reach Citadel service logic without a valid, active session token.

### Service: `CitadelSession`

The session service manages the full agent lifecycle:

```
CreateLease   ──► Validates agent identity, checks for duplicate sessions,
                  creates SessionRecord in Redis, returns session_id + token

Heartbeat     ──► Refreshes the TTL on the agent's Redis lease key,
                  keeping the session alive

CloseSession  ──► Removes SessionRecord from Redis, fires session_closed
                  event to the Koad Stream for EOW processing
```

A background **reaper task** runs on a configurable interval (`reaper_interval_secs`). It scans all sessions in Redis and marks any session `dark` if its lease key TTL has expired. Sessions that remain dark past `dark_timeout_secs` are purged and their `session_closed` events are fired, ensuring CASS processes their EndOfWatch summaries even for crashed sessions.

See [Agent Session Lifecycle](./agent-session-lifecycle.md) for a complete walkthrough of the state machine.

### Service: `Sector`

The Sector service manages shared resources and command safety:

**`AcquireLock` / `ReleaseLock`**: A Redis-backed distributed locking mechanism. Before two agents modify the same resource (e.g., a shared file or database record), one agent acquires a named lock. The other must wait or fail fast. This prevents concurrent mutation conflicts in a multi-agent environment.

**`ValidateIntent`**: The entry point for `koad-sandbox`. When an agent requests to run a command, it calls `ValidateIntent` with the command and target path. The Citadel passes this to `Sandbox::evaluate()`, which checks the command against a policy appropriate for the agent's rank and workspace level. The result is either `Approved`, `Denied`, or `Sandboxed` (run in a restricted environment).

### Service: `Signal`

The Signal service is the Citadel's event bus, backed by Redis Streams under the key prefix `koad:stream:`:

**`Broadcast`**: Publish an event to a named stream. Any agent or internal service can broadcast events (e.g., `session_created`, `task_complete`, `tool_result`).

**`Subscribe`**: Stream events from a named channel to the caller. This is a server-side streaming RPC — the Citadel continuously forwards new events from Redis to the subscribed agent.

The `koad:stream:system` channel is reserved for internal Citadel events (session lifecycle events). CASS's `EndOfWatchPipeline` subscribes to this channel to detect session closures.

### Service: `PersonalBay`

The PersonalBay service manages agent-specific, sovereign storage areas:

**`Provision`**: Creates the directory structure and database files for a new agent. Every named agent in `config/identities/` gets a "bay" — a dedicated area within `~/.koad-os/bays/<agent_name>/` containing their working files and state.

**`ProvisionWorkspace`**: Creates isolated `git worktree` environments for agents to perform tasks without contaminating the main branch. Each worktree gets a generated name (e.g., `claude/agitated-swartz`) and is scoped to the requesting agent.

## Configuration

| Key | Location | Default | Description |
|-----|----------|---------|-------------|
| `citadel_grpc_addr` | `config/kernel.toml` | `127.0.0.1:50051` | Address the Citadel listens on |
| `[sessions].lease_duration_secs` | `config/kernel.toml` | `90` | TTL for a session's Redis lease key |
| `[sessions].dark_timeout_secs` | `config/kernel.toml` | `300` | Time before a dark session is purged |
| `[sessions].reaper_interval_secs` | `config/kernel.toml` | `10` | How often the reaper scans for dark sessions |
| `config/identities/*.toml` | `config/identities/` | — | Agent identity files used by PersonalBayService for bay provisioning |

## Failure Modes & Edge Cases

**The Citadel crashes mid-session.**
Active agents lose the ability to make privileged gRPC calls — all requests will be rejected by the interceptor (which can't validate tokens without Redis). On Citadel restart, Redis state (sessions, leases) is recovered automatically. Agents that were mid-operation when the crash occurred may find their operations incomplete. The reaper task on restart will scan for dark sessions and clean them up. Agents need to send a heartbeat on reconnect to confirm their session is still live.

**Redis becomes unreachable.**
The Citadel's session and locking services depend entirely on Redis. If Redis becomes unreachable, `CreateLease`, `Heartbeat`, `AcquireLock`, and most other operations will fail. The Citadel will log errors and return `Unavailable` to callers. This is a critical single point of failure in the current architecture; Redis should be treated as a required dependency.

**Session reaper marks a live session as dark.**
This can happen if the agent's heartbeat is delayed (e.g., high system load) and the lease TTL expires before the heartbeat arrives. The reaper marks the session dark. If the agent sends a heartbeat before the `dark_timeout_secs` deadline, the Citadel can potentially recover the session — but this edge case is not explicitly handled in the current implementation. The safe fallback is for the agent to re-boot.

**Two agents simultaneously try to acquire the same lock.**
`AcquireLock` is implemented using Redis's atomic `SET NX` (set if not exists) with a TTL. The first caller wins; the second receives a `AlreadyExists` error. The calling agent must decide whether to retry, wait, or fail. The lock TTL prevents dead locks if the holding agent crashes before calling `ReleaseLock`.

## FAQ

### Q: What is the Citadel and what does it do?
The Citadel (`koad-citadel`) is the "OS kernel" of KoadOS — a persistent gRPC server that manages every aspect of an agent's operational existence: booting into sessions, maintaining session leases, validating security on every call, managing resource locks, running the event bus, and provisioning agent workspaces. It's the infrastructure layer that the rest of the system depends on. Think of it as the component that answers the question "can this agent act?" while CASS answers "what does this agent know?".

### Q: How does the Citadel keep agents from doing dangerous things?
Two mechanisms work in concert. First, the gRPC interceptor (`auth/interceptor.rs`) validates the session token on every call — an agent without a valid session can't do anything. Second, the `Sector` service's `ValidateIntent` RPC routes commands through `koad-sandbox`, which checks them against a policy based on the agent's rank and workspace level. A Crew-rank agent at Outpost level will have commands vetted more strictly than a Captain-rank agent at Citadel level.

### Q: What's the difference between the `koad-citadel` binary and the `koad-agent` CLI?
`koad-citadel` is the server — it runs continuously as a background process and manages all sessions. `koad-agent` (the CLI, in `koad-cli`) is the client — it runs briefly when an agent boots, makes a `CreateLease` gRPC call to the Citadel, and exits after printing the environment variables to stdout. The Citadel persists; the CLI is transient.

### Q: What happens if the Citadel server crashes?
Active sessions have their leases stored in Redis, which is separate from the Citadel process. When the Citadel restarts, it reconnects to Redis and picks up where it left off. Agents with live sessions will see gRPC failures during the restart window, then recover when they resume heartbeats. The reaper task on restart cleans up any sessions that went dark during the outage. Operations that were in flight during the crash may need to be retried by the agent.

### Q: How do I configure the session timeout or the gRPC port?
Edit `config/kernel.toml`. The `[sessions]` section controls `lease_duration_secs` (heartbeat interval budget), `dark_timeout_secs` (how long before a dark session is purged), and `reaper_interval_secs` (how often the reaper runs). The gRPC address is set via `citadel_grpc_addr` (may appear as `spine_grpc_addr` in current configs — legacy naming, same field).

## Source Reference

- `crates/koad-citadel/src/main.rs` — Binary entry point; loads config, initializes all components, starts the server
- `crates/koad-citadel/src/kernel.rs` — `Kernel` struct; assembles gRPC services and starts the server with the interceptor
- `crates/koad-citadel/src/auth/interceptor.rs` — `build_citadel_interceptor()`; Zero-Trust token validation on every call
- `crates/koad-citadel/src/services/session.rs` — `CitadelSessionService`; session lifecycle and the reaper task
- `crates/koad-citadel/src/services/sector.rs` — `SectorService`; distributed locking and command validation
- `crates/koad-citadel/src/services/signal.rs` — `SignalService`; Redis Streams-backed event bus
- `crates/koad-citadel/src/services/bay.rs` — `PersonalBayService`; agent bay and worktree provisioning
- `crates/koad-citadel/src/auth/session_cache.rs` — `SessionRecord`, `ActiveSessions`; Redis interaction for session state
- `proto/citadel.proto` — gRPC service and message definitions for all Citadel services
- `config/kernel.toml` — Runtime configuration for all Citadel parameters
