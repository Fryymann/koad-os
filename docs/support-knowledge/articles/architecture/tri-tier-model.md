# The Tri-Tier Model

> The foundational architecture of KoadOS, splitting the system into three specialized layers — Body, Brain, and Link — that communicate exclusively via gRPC.

**Complexity:** basic
**Related Articles:** [The Body/Ghost Model](./body-ghost-model.md), [koad-citadel](../core-systems/koad-citadel.md), [koad-cass](../core-systems/koad-cass.md), [koad-agent boot](../core-systems/koad-agent-boot.md)

---

## Overview

The Tri-Tier Model is the architectural foundation of the KoadOS rebuild. It replaces the previous monolithic "Spine" binary — which combined sessions, security, memory, and orchestration into a single process — with three clearly separated layers, each with a distinct responsibility and a well-defined gRPC interface.

The three tiers are:

- **The Link** (`koad-agent` / `koad-cli`): The identity and entry point. Gets an agent into the system.
- **The Citadel** (`koad-citadel`): The "Body". Manages sessions, security, and physical infrastructure.
- **CASS** (`koad-cass`): The "Brain". Manages memory, intelligence, and context.

The core insight behind this model is that "existing" (having a session) is a different concern from "thinking" (having memory and context), and both are different from "entering" (the boot process). The old Spine entangled all three, making the system brittle. The Tri-Tier model enforces hard boundaries through network-level contracts (gRPC), making each tier independently deployable, testable, and maintainable.

The architecture was born out of real problems with the legacy Spine: two agents couldn't share the same AI driver, secrets were scattered, and a bug in memory management could crash the session layer. The Tri-Tier model eliminates these failure cascades by design.

## How It Works

Data flows left to right: Link → Citadel → CASS. Each hop is a gRPC call.

### Tier 1: The Link (`koad-cli`)

The Link is the agent's entry point into the system. When a developer or AI agent runs `eval $(koad-agent boot --agent Tyr)`, the `koad-cli` binary:

1. Reads the agent's identity from `config/identities/tyr.toml`
2. Makes a `CreateLease` gRPC call to the Citadel with the agent's name and a fresh `body_id`
3. Receives a session token from the Citadel on success
4. Prints `export` statements to stdout (captured and executed by `eval`)

The Link doesn't maintain any persistent state of its own. It's a stateless bootstrapper — its only job is to fuse a Ghost (identity) with a Body (shell session) and exit. After boot, the agent communicates with the Citadel directly using the `KOAD_SESSION_TOKEN` injected into the environment.

### Tier 2: The Citadel (`koad-citadel`)

The Citadel is the "operating system kernel" of KoadOS. It's a long-running `tonic` gRPC server that acts as the single source of truth for all session state. Every gRPC call into the Citadel passes through a `tonic` interceptor that validates the session token before the request reaches any service logic — this is the Zero-Trust enforcement point.

The Citadel's primary services are:

- **`CitadelSession`**: Issues and manages session leases. Enforces the "One Body, One Ghost" rule.
- **`Sector`**: Manages shared locks (via Redis) and routes commands to the sandbox for validation.
- **`Signal`**: The event bus, backed by Redis Streams, allowing agents to broadcast and subscribe to system events.
- **`PersonalBay`**: Provisions agent-specific storage and manages git worktrees for isolated task execution.

The Citadel stores hot state (active sessions, locks) in Redis and delegates all cognitive requests (memory, context) to CASS.

### Tier 3: CASS (`koad-cass`)

CASS — the Citadel Agent Support System — is the "brain". It's a separate long-running gRPC server that handles everything requiring intelligence or persistent memory:

- **`MemoryService`**: Commits and queries `FactCard` (long-term knowledge) and `EpisodicMemory` (session summaries) to a SQLite database (`cass.db`).
- **`HydrationService`**: Builds the initial context packet an agent receives at boot via Temporal Context Hydration (TCH). Respects token budgets and workspace level.
- **`SymbolService`**: Provides code graph queries — look up where a function or struct is defined across the codebase.
- **`EndOfWatchPipeline`**: A background task that auto-summarizes sessions when they close, using `koad-intelligence` to generate AI-powered retrospectives.

Agents generally do not call CASS directly. The Citadel orchestrates CASS on their behalf.

### The `TraceContext`

All gRPC requests carry a `TraceContext` message (defined in `proto/`) that includes the `session_id` and a per-request `request_id`. This is threaded through all three tiers, producing a complete audit chain for every operation. It is the primary observability primitive in KoadOS.

```
koad-agent boot
    │
    ▼ CreateLease (gRPC + TraceContext)
koad-citadel  ──── session token ──► shell env ($KOAD_SESSION_TOKEN)
    │
    ▼ Hydrate (gRPC + TraceContext, routed by Citadel)
koad-cass  ──── context packet ──► agent receives on boot
```

## Configuration

| Key | Location | Description |
|-----|----------|-------------|
| `citadel_grpc_addr` | `config/kernel.toml` | Address the Citadel listens on (e.g., `127.0.0.1:50051`) |
| `cass_grpc_addr` | `config/kernel.toml` | Address CASS listens on (e.g., `127.0.0.1:50052`) |
| `KOAD_AGENT_NAME` | Shell environment | Set by `koad-agent boot`; identifies the active agent |
| `KOAD_SESSION_ID` | Shell environment | The public session identifier |
| `KOAD_SESSION_TOKEN` | Shell environment | The private credential used to authenticate all gRPC calls |

> **Note:** The `config/kernel.toml` file currently uses the field name `spine_grpc_addr` in some places — this is legacy naming from the old Spine architecture and refers to the Citadel's gRPC address. It will be renamed to `citadel_grpc_addr` in a future cleanup.

## Failure Modes & Edge Cases

**What happens if the Citadel crashes?**
Active agents lose the ability to validate future gRPC calls — the interceptor will reject them. Existing shell sessions remain alive (the shell process doesn't die), but agents can't make new privileged requests. On Citadel restart, sessions already in Redis are recovered and agents can resume by sending a heartbeat. Sessions that were mid-operation when the crash occurred may be left in an inconsistent state; the reaper task will eventually mark them dark and purge them.

**What happens if CASS crashes?**
The Citadel continues to function normally — session management, security, and the event bus are all unaffected. Agents lose access to memory queries, context hydration, and the EndOfWatch pipeline. Sessions close without being summarized until CASS is restored and can process the backlog from the Redis stream.

**Can the Citadel and CASS run in the same process?**
Technically yes — both are `tonic` gRPC services and could be hosted in the same binary. They communicate over the network stack even when on the same machine, so combining them is a deployment detail, not an architectural violation. In practice, they run as separate processes to allow independent restarts and resource management.

## FAQ

### Q: What's the difference between the Citadel and CASS?
The Citadel is infrastructure — it manages whether an agent is alive, authenticated, and allowed to act. CASS is cognition — it manages what an agent knows, remembers, and has as context when it starts up. The Citadel handles fast, low-latency questions ("Is this session valid?"). CASS handles slower, complex work ("Summarize this session" or "What do I know about Redis?"). Think of the Citadel as the OS kernel and CASS as the intelligence layer running on top of it.

### Q: Where does the agent's session actually "live"?
The session lives in Redis, managed by the Citadel. When `koad-agent boot` succeeds, the Citadel writes a `SessionRecord` to `koad:state` in Redis and creates a TTL-keyed lease at `koad:session:<session_id>`. The shell holds the `KOAD_SESSION_TOKEN` that references this record. The session "exists" as long as the TTL key is alive (kept fresh by periodic heartbeats).

### Q: How do the three tiers communicate?
Exclusively via gRPC, with protobuf message definitions in `proto/`. The Link calls the Citadel; the Citadel calls CASS. No tier reaches into another's internal state directly. All cross-tier communication carries a `TraceContext` for observability.

### Q: Why was the old "Spine" model retired?
The Spine was a monolithic binary that combined all responsibilities. This meant a memory bug could crash the session layer, two agents couldn't use the same AI driver simultaneously, and the security model was porous. The Tri-Tier model separates these concerns into independent processes with explicit, versioned gRPC contracts between them.

### Q: Can I run the Citadel without CASS?
Yes. The Citadel has no hard startup dependency on CASS. Sessions, security, locking, and the event bus all work without CASS. You lose context hydration at boot, memory commit/query, and EndOfWatch summaries — but the core session infrastructure remains operational.

### Q: What happens if the gRPC link between an agent and the Citadel breaks?
The agent's shell continues to run, but any operation requiring Citadel validation will fail (the interceptor rejects requests without a valid session token). The agent's heartbeat will stop reaching the Citadel, and after the `lease_duration_secs` TTL expires, the session will be marked dark and eventually purged by the reaper. The agent would need to `eval $(koad-agent boot ...)` again in a new shell to establish a fresh session.

## Source Reference

- `crates/koad-cli/src/main.rs` — Entry point for the Link tier; handles the `boot` command and initiates the Citadel handshake
- `crates/koad-cli/src/handlers/boot.rs` — Core boot logic: identity loading, gRPC call, shell script generation
- `crates/koad-citadel/src/kernel.rs` — The `Kernel` struct that assembles and starts all Citadel gRPC services
- `crates/koad-citadel/src/auth/interceptor.rs` — The Zero-Trust gRPC interceptor; validates session tokens on every call
- `crates/koad-cass/src/main.rs` — Entry point for the CASS tier; assembles MemoryService, HydrationService, SymbolService
- `proto/citadel.proto` — gRPC contract between Link and Citadel
- `proto/cass.proto` — gRPC contract between Citadel and CASS
