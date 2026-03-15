# The Tri-Tier Model

## Metadata
- Category: ARCHITECTURE & CONCEPTS
- Complexity: basic
- Related Topics: body-ghost-model, koad-citadel, koad-cass, koad-cli
- Key Source Files: `crates/koad-citadel/src/lib.rs`, `crates/koad-cass/src/lib.rs`, `crates/koad-cli/src/main.rs`
- Key Canon/Doc References: `AGENTS.md`, `.agents/CITADEL.md`

## Summary
The Tri-Tier Model is the foundational architecture of the KoadOS rebuild, replacing the monolithic "Spine". It separates concerns into three distinct layers: the Body (infrastructure), the Brain (cognition), and the Link (identity). This model provides clear boundaries, enhances security, and allows for independent development and scaling of each system component.

## How It Works
The system is designed as a hierarchy of services that communicate via gRPC.

1.  **The Link (`koad-agent` / `koad-cli`):** This is the user/agent entry point. The `koad-agent boot` command initiates a session. It reads the agent's identity from a TOML file, prepares the shell environment with necessary variables, and performs the initial gRPC handshake with the Citadel to acquire a session lease. It acts as the "Link" between the agent's "Ghost" (identity) and its "Body" (the shell session).

2.  **The Citadel (The "Body"):** This is the core infrastructure layer, a persistent gRPC service (`koad-citadel`). It is the operating system's kernel. Its primary responsibilities include:
    - **Session Management:** Issuing, tracking, and reaping session leases via Redis. Enforces the "One Body, One Ghost" rule.
    - **Security & Jailing:** Validating all incoming gRPC requests with an auth interceptor. The `koad-sandbox` is integrated here to enforce command and path policies.
    - **Orchestration:** Manages personal agent bays, worktrees, and routes requests to the correct subsystem (like CASS).

3.  **CASS (The "Brain"):** The Citadel Agent Support System is the cognitive layer, another persistent gRPC service (`koad-cass`). It provides high-level services that agents need to reason and remember.
    - **Memory:** Provides RPCs to commit and query long-term memory (`FactCard`, `EpisodicMemory`) from a SQLite database.
    - **Intelligence:** Uses the `koad-intelligence` crate to perform tasks like session summarization (distillation) and semantic scoring of facts.
    - **Context Hydration (TCH):** Prepares the initial context packet an agent receives upon boot, respecting token budgets.

Data flows from the Link (`koad-agent`) to the Body (`koad-citadel`), which then orchestrates and delegates cognitive tasks to the Brain (`koad-cass`).

## Key Code References
- **File**: `crates/koad-cli/src/main.rs`
  - **Element**: `main()` function, `Commands::Boot` match arm
  - **Purpose**: Entry point for the agent's interaction with the system. Initiates the boot handshake.
- **File**: `crates/koad-citadel/src/kernel.rs`
  - **Element**: `KernelBuilder`
  - **Purpose**: Assembles and starts all the gRPC services that form the "Body" (CitadelSession, Sector, Signal, etc.).
- **File**: `crates/koad-cass/src/main.rs`
  - **Element**: `main()` function
  - **Purpose**: Assembles and starts all the gRPC services that form the "Brain" (MemoryService, HydrationService, SymbolService).

## Configuration & Environment
- `KOAD_AGENT_NAME`: Injected by `koad-agent boot` to inform the shell which agent is active.
- `KOAD_SESSION_ID`: Injected by `koad-agent boot` with the lease ID from the Citadel.
- `config/kernel.toml`: Contains the gRPC addresses (`spine_grpc_addr`, which should be `citadel_grpc_addr`, and the CASS address) that link the tiers.

## Common Questions a Human Would Ask
- "What's the difference between the Citadel and CASS?"
- "Where does the agent's session actually 'live'?"
- "How do the three main parts of KoadOS talk to each other?"
- "Why was the old 'Spine' model retired in favor of this?"
- "Can I run the Citadel without CASS?"
- "What happens if the gRPC link between the agent and the Citadel breaks?"

## Raw Technical Notes
- The previous "Spine" architecture combined all these responsibilities into a single monolithic binary, which was difficult to maintain and secure. The Tri-Tier model is a direct response to the fragility of that design.
- Communication is entirely over gRPC, defined in `proto/`. This enforces a strict API contract between the layers.
- The `TraceContext` message is the key to observability, as it's passed through all three tiers for every request, allowing for a complete audit trail.
- While logically separate, `koad-citadel` and `koad-cass` are currently run as separate processes, but could in theory be hosted in the same binary if needed, since they communicate over the network stack.
