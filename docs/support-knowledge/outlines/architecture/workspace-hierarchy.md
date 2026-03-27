# The Workspace Hierarchy

## Metadata
- Category: ARCHITECTURE & CONCEPTS
- Complexity: intermediate
- Related Topics: tri-tier-model, personal-bays, temporal-context-hydration
- Key Source Files: `crates/koad-core/src/hierarchy.rs`, `crates/koad-core/src/config.rs`, `crates/koad-citadel/src/services/session.rs`
- Key Canon/Doc References: `AGENTS.md`, `DRAFT_PLAN_3.md`

## Summary
The KoadOS Workspace Hierarchy is a four-tier mapping system that provides logical and physical separation for agent operations. It applies a "Game Map Metaphor" to the filesystem, defining scopes from the entire machine down to a single repository. This structure is critical for security, context management, and efficient tool use, forming the basis of the Sanctuary Rule and Level-Awareness.

## How It Works
The system uses the `HierarchyManager` in `koad-core` to determine the current level based on the agent's working directory and the `koad.toml` configuration files found within the project structure.

The four levels are:
1.  **Level 4: System:** The entire machine (`/`). This level is generally off-limits except for the `Admiral` or `Dood` for maintenance. An agent's personal vault (e.g., `~/.tyr`) resides here.
2.  **Level 3: Citadel:** The core KoadOS platform directory (`~/.koad-os/`). This contains the running services, shared configuration, and canonical documentation. Only high-ranking agents like Scribe or Vigil operate primarily at this level.
3.  **Level 2: Station:** A project hub or domain containing multiple related repositories (e.g., `/home/ideans/data/skylinks/agents/sky/`). This allows an agent to have context over a whole suite of applications.
4.  **Level 1: Outpost:** A single git repository. This is the most common operational level for "Crew" agents, as it strictly jails their operations to the current codebase.

When an agent boots, `koad-agent` passes the current path to the Citadel's `CreateLease` RPC. The Citadel uses its `HierarchyManager` to resolve the level and embeds this information in the session token. Every subsequent gRPC call from the agent carries this level, allowing services like CASS to make level-aware decisions. For example, the **Temporal Context Hydrator (TCH)** will load *only* local files at the Outpost level but may include high-level pointers from the Station level, preventing token-costly context pollution.

## Key Code References
- **File**: `crates/koad-core/src/hierarchy.rs`
  - **Element**: `HierarchyManager` struct, `resolve_level()` method
  - **Purpose**: Contains the core logic for walking up the directory tree from a given path, looking for `.koad` markers or specific directory names to determine the current `WorkspaceLevel`.
- **File**: `crates/koad-core/src/config.rs`
  - **Element**: `KoadConfig`, `ProjectConfig` structs
  - **Purpose**: Defines how `koad.toml` files are structured, which is what the `HierarchyManager` looks for to identify Stations and Outposts.
- **File**: `crates/koad-cass/src/services/hydration.rs`
  - **Element**: `CassHydrationService::hydrate()` method
  - **Purpose**: This is a key consumer of the hierarchy. It checks the `level` from the `HydrationRequest` to decide how much context to load (e.g., the "Hierarchy Walk").

## Configuration & Environment
- `.koad`: A marker file or directory that can be placed in a project root to explicitly define it as a `Station` or `Outpost`.
- `koad.toml`: A configuration file within a project that can define its name and relationship to a parent Station.

## Common Questions a Human Would Ask
- "How does the system know if I'm in a repository vs. a larger project?"
- "What prevents an agent from accessing files outside its designated 'Outpost'?"
- "How does the hierarchy help save money on tokens?"
- "Can I define my own 'Station' for a new project?"
- "What is the `HierarchyManager` and where does it run?"

## Raw Technical Notes
- The hierarchy is a logical concept enforced by the Citadel. It's not a true OS-level chroot jail, but a gRPC-level one. An agent with direct shell access could, in theory, `cd` out of its workspace. The `koad-sandbox` is the next line of defense against this.
- The `Hierarchy Walk` performed by the TCH is a key optimization. It avoids loading every file in a large monorepo by default, instead prioritizing local `agents/` folders and high-level readme files.
- This system is the technical implementation of the "Locality of Reference" principle mentioned in the canon docs.
