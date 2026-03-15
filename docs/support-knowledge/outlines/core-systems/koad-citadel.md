# koad-citadel: The Core OS Kernel

## Metadata
- Category: CORE SYSTEMS & SUBSYSTEMS
- Complexity: advanced
- Related Topics: tri-tier-model, agent-session-lifecycle, zero-trust-security, personal-bays
- Key Source Files: `crates/koad-citadel/src/main.rs`, `crates/koad-citadel/src/kernel.rs`, `proto/citadel.proto`
- Key Canon/Doc References: `.agents/CITADEL.md`

## Summary
The `koad-citadel` crate is the "Body" of the KoadOS Tri-Tier model. It's a persistent, multi-service gRPC server that acts as the central operating system kernel. It is responsible for orchestrating agent sessions, managing shared state, enforcing security policies, and providing core infrastructure services that all other parts of the system rely on.

## How It Works
The `koad-citadel` binary starts up and listens for gRPC connections on a configured port (e.g., `127.0.0.1:50051`). It is assembled by the `KernelBuilder` in `kernel.rs`, which initializes and injects all necessary dependencies into its various sub-services.

Its primary gRPC services include:
1.  **`CitadelSession`**: Manages the entire agent lifecycle.
    - **`CreateLease`**: Authenticates an agent via `koad-agent boot` and issues a session token.
    - **`Heartbeat`**: Allows an active agent to maintain its session lease.
    - **`CloseSession`**: Terminates a session and triggers the EndOfWatch pipeline.
    - **Reaper Task**: A background task that automatically cleans up "dark" or "zombie" sessions that have timed out.

2.  **`Sector`**: Manages shared resources and locks.
    - **`AcquireLock` / `ReleaseLock`**: A Redis-based distributed locking mechanism to prevent multiple agents from modifying the same resource simultaneously.
    - **`ValidateIntent`**: The entry point for the `koad-sandbox`, allowing agents to have their commands pre-validated against security policies.

3.  **`Signal`**: Manages the event bus (Koad Stream).
    - **`Subscribe` / `Broadcast`**: Allows agents to listen for and send events across the system, using Redis Streams as the backend.

4.  **`PersonalBay`**: Manages agent-specific storage and workspaces.
    - **`Provision`**: Creates the necessary directory structure and database files for a new agent.
    - **`ProvisionWorkspace`**: Creates temporary `git worktree` environments for agents to perform tasks in isolation.

All incoming requests are first processed by a `tonic` gRPC interceptor (`auth/interceptor.rs`) which enforces Zero-Trust security by validating the session token on every single call.

## Key Code References
- **File**: `crates/koad-citadel/src/main.rs`
  - **Element**: `main()` function
  - **Purpose**: The binary entry point. It loads the `KoadConfig`, initializes all the manager and service structs (e.g., `RedisClient`, `Sandbox`, `CitadelSessionService`), and starts the gRPC server.
- **File**: `crates/koad-citadel/src/kernel.rs`
  - **Element**: `KernelBuilder`
  - **Purpose**: Provides a fluent interface for constructing and launching the Citadel. This is used by `main.rs` and allows for easier testing and configuration.
- **File**: `crates/koad-citadel/src/auth/interceptor.rs`
  - **Element**: `build_citadel_interceptor()`
  - **Purpose**: This is the heart of the Zero-Trust security model. It creates a `tonic::service::Interceptor` that checks the `KOAD_SESSION_ID` from the gRPC metadata against the active session cache in Redis *before* the request is passed to the actual service logic.

## Configuration & Environment
- `config/kernel.toml`: The primary configuration file for the Citadel. Defines gRPC ports, database paths, and session timeouts.
- `config/identities/*.toml`: Used by the `PersonalBayService` to know which agents to provision bays for.

## Common Questions a Human Would Ask
- "What is the Citadel and what does it do?"
- "How does the Citadel keep agents from doing dangerous things?"
- "What's the difference between the `koad-citadel` binary and the `koad-agent` CLI?"
- "What happens if the Citadel server crashes?"
- "How do I configure the session timeout or the gRPC port?"

## Raw Technical Notes
- The Citadel is designed to be the single source of truth for all "Body"-related state. It is the only component that should have direct write access to core Redis state like session leases.
- It is heavily asynchronous, built entirely on `tokio` and `tonic`. Background tasks like the session reaper and the Redis state drain loop are spawned as independent, long-running futures.
- The use of `Arc` for all shared services (e.g., `Arc<RedisClient>`, `Arc<Sandbox>`) is critical for safely sharing these resources across the multiple concurrent tasks and gRPC service handlers.
