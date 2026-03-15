# koad-cass: The Agent Support System

## Metadata
- Category: CORE SYSTEMS & SUBSYSTEMS
- Complexity: advanced
- Related Topics: tri-tier-model, temporal-context-hydration, koad-intelligence, data-storage-sqlite
- Key Source Files: `crates/koad-cass/src/main.rs`, `crates/koad-cass/src/storage/mod.rs`, `proto/cass.proto`
- Key Canon/Doc References: `AGENTS.md`

## Summary
`koad-cass` (Citadel Agent Support System) is the "Brain" of the KoadOS Tri-Tier model. It's a persistent gRPC service that provides high-level cognitive and memory support to agents. While the Citadel manages the "physical" aspects of an agent's existence (sessions, security), CASS manages the "mental" aspects (memory, learning, context).

## How It Works
Like the Citadel, `koad-cass` is a `tonic`-based gRPC server that listens on its own port (e.g., `127.0.0.1:50052`). Agents do not typically call CASS directly; instead, the Citadel or other backend systems act as a proxy.

Its primary gRPC services include:
1.  **`MemoryService`**: The interface to the agent's long-term memory.
    - **`CommitFact`**: Allows an agent to save a `FactCard` (a distilled piece of knowledge) to the persistent SQLite database. This service integrates with `koad-intelligence` to automatically score the "significance" of the fact if one isn't provided.
    - **`QueryFacts`**: Retrieves facts from the database based on domain or tags.
    - **`RecordEpisode`**: Saves the `EndOfWatch` summary for a completed session.

2.  **`HydrationService`**: Responsible for providing agents with their initial context at boot time.
    - **`Hydrate` (Temporal Context Hydration - TCH)**: This is a critical service for reducing token consumption. It receives a token budget and a workspace level, then performs a "Hierarchy Walk" to intelligently select the most relevant files, recent `EndOfWatch` summaries, and high-significance `FactCard`s to build a context packet that fits the budget.

3.  **`SymbolService`**: The interface to the code knowledge graph.
    - **`Query`**: Allows an agent to look up the definition and location of a function, struct, or other symbol.
    - **`IndexProject`**: Triggers a background task to re-index the entire project's source code using `koad-codegraph`.

4.  **`EndOfWatchPipeline` (Internal Task)**:
    - This is a long-running background task, not a gRPC service. It listens to the `koad:stream:system` Redis stream for `session_closed` events.
    - When a session closes, it uses `koad-intelligence` to generate an AI-powered summary of the session's activities and then saves it to the database via the `MemoryService`. This is the "Distillation Gap" that was bridged in Phase 3.

## Key Code References
- **File**: `crates/koad-cass/src/main.rs`
  - **Element**: `main()` function
  - **Purpose**: The binary entry point. Initializes the `CassStorage` (SQLite), the `InferenceRouter`, the `CodeGraph`, and all the gRPC services.
- **File**: `crates/koad-cass/src/storage/mod.rs`
  - **Element**: `CassStorage` struct, `Storage` trait
  - **Purpose**: Defines the database schema and logic for all persistent storage in CASS. It handles the SQL for `FactCard` and `EpisodicMemory` tables.
- **File**: `crates/koad-cass/src/services/hydration.rs`
  - **Element**: `CassHydrationService`
  - **Purpose**: Implements the complex logic for Temporal Context Hydration (TCH), including the file system walk and token budgeting.
- **File**: `crates/koad-cass/src/services/eow.rs`
  - **Element**: `EndOfWatchPipeline`
  - **Purpose**: The background process that listens for closed sessions and uses the intelligence layer to summarize them, completing the cognitive loop.

## Configuration & Environment
- `config/kernel.toml`: The CASS gRPC address is configured here, though it's not yet referenced by other services in the current implementation.
- `cass.db`: The SQLite database file created and managed by `CassStorage`.

## Common Questions a Human Would Ask
- "What is CASS and how is it different from the Citadel?"
- "How does the system decide what information to give an agent when it starts up?"
- "Where are agent memories actually stored?"
- "What happens at the 'EndOfWatch'?"
- "How can I query the code graph to find where a function is?"

## Raw Technical Notes
- CASS is designed to be the "heavy lifting" cognitive partner to the Citadel. The Citadel handles fast, low-latency infrastructure tasks (Is this session valid? Is this command allowed?), while CASS handles slower, more complex cognitive tasks (Summarize this session. Find me relevant facts.).
- The `Storage` trait in `crates/koad-cass/src/storage/mod.rs` is a key architectural pattern, allowing the real `CassStorage` to be swapped out with a `MockStorage` for unit testing the gRPC services without needing a real database file.
- The `spawn_blocking` call in the `SymbolService::index_project` method is a good example of the `RUST_CANON` Non-Blocking Rule. The CPU-intensive `tree-sitter` parsing is moved to a dedicated thread so it doesn't block the main `tokio` async runtime.
