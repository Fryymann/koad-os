## 16. State of the Union & Structural Refactor (The Great Decoupling)

**Current State Analysis:**
*   **The Monolith:** `core/rust/src/main.rs` is currently a ~1,500 line monolith. It handles everything from Airtable syncs to database queries to CLI parsing. This is fragile and violates the concept of modular "Ship Systems."
*   **Type Duplication:** `kspine.rs` (The Kernel) and `kbooster.rs` (The Sidecar) are currently duplicating JSON state definitions (e.g., `KoadOSState`, `AgentState`). 
*   **Skill Fragmentation:** Python skills (`vault.py`, `blueprint_engine.py`) are robust but loosely coupled. The Rust Kernel executes them blindly without a formal RPC or callback interface.

**Proposed Big Design Changes:**

### A. The Cargo Workspace Refactor (Modular Ship Systems)
We must break the Rust core into a true `Cargo Workspace` consisting of specialized crates. This mirrors the "Spaceship" metaphor physically in the codebase:
1.  **`koad-core` (The Hull):** A library crate containing shared types (`AgentProfile`, `Task`, `CommMessage`), SQLite interfaces, and error handling. Used by all binaries.
2.  **`koad-spine` (The Engine Room):** The background Kernel daemon. Depends on `koad-core`. Handles WebSockets, Redis (future), and background loops.
3.  **`koad-cli` (The Bridge):** The thin-client terminal interface (`main.rs`). It contains *zero* business logic; it only parses arguments and sends requests to the `kspine` API or executes Python skills.
4.  **`koad-tui` (The Viewscreen):** Extracted from `main.rs`, dedicated entirely to the Ratatui dashboard.

### B. The Unified Skill API (The Intercom)
Currently, `kspine` executes Python skills via shell commands. This is brittle.
*   **Change:** Implement a local gRPC or UNIX Domain Socket (UDS) interface for Skills. When a Python skill runs, it communicates back to the Kernel via an SDK (e.g., `import koados`). This allows skills to report progress, emit logs to the Comm-Array, and update the Task Graph dynamically.

### C. The Agent "Memory Page" (Paging Architecture)
*   **Current Issue:** When I (Koad) boot, the CLI dumps the entire `PROJECT_PROGRESS.md` and recent database rows into the terminal. This consumes massive context tokens upfront.
*   **Change:** Implement a "Paging" memory architecture. The `kbooster` compiles a highly compressed "Index" of the current state. If I need deep context on a specific file or task, I use a specific tool to "page" that memory into my context window. This shifts me from a "read-all" to a "query-on-demand" intelligence model, drastically reducing token waste.

*Status: Awaiting Creator review on The Great Decoupling.*