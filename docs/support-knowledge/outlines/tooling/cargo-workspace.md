# Cargo Workspace (koad-os)

## Metadata
- Category: TOOLING & DEVELOPER WORKFLOW
- Complexity: intermediate
- Related Topics: rust-canon, koad-core
- Key Source Files: `Cargo.toml`, `crates/`
- Key Canon/Doc References: `docs/protocols/RUST_CANON.md`

## Summary
The KoadOS project is structured as a Rust [Cargo Workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html). This means the entire system is composed of multiple, individual crates that live in the `crates/` directory but are compiled together and share a single `Cargo.lock` file. This approach promotes modularity, clean boundaries, and efficient compilation.

## How It Works
The workspace is defined by the `[workspace]` table in the root `Cargo.toml` file.

1.  **`[workspace]` Table**:
    - The `members` array in this table lists all the individual crates that are part of the workspace (e.g., `"crates/koad-core"`, `"crates/koad-citadel"`). Any new crate must be added to this list to be recognized.

2.  **`[workspace.dependencies]` Table**:
    - This is a key feature for ensuring consistency. Common dependencies used by many crates (like `tokio`, `serde`, `anyhow`, `tracing`) are defined **once** here.
    - Individual crates can then inherit this dependency by specifying `dep-name.workspace = true` in their own `Cargo.toml`. This guarantees that every crate in the entire project uses the exact same version of `tokio`, preventing version conflicts and code bloat.

3.  **Shared `Cargo.lock`**:
    - All crates share a single `target/` directory and a single `Cargo.lock` file at the root of the workspace. This means that when you build one crate, any of its local dependencies are also recompiled if they have changed, and all transitive dependency versions are locked for the entire project.

4.  **Crate Boundaries**:
    - The workspace is organized by function:
        - `koad-core`: The foundation. Contains shared structs, traits, and utilities that almost every other crate needs. It has very few dependencies.
        - `koad-proto`: Auto-generated gRPC code from the `.proto` files.
        - `koad-citadel`, `koad-cass`: "Binary" crates that represent the main, long-running services. They depend on library crates like `koad-core` and `koad-intelligence`.
        - `koad-intelligence`, `koad-sandbox`: "Library" crates that provide a specific piece of functionality but don't run on their own. They are consumed by other crates.
        - `koad-cli`: A binary crate for the user-facing command-line tool.

Running `cargo` commands from the root directory (e.g., `cargo check`, `cargo build`) will operate on all crates in the workspace, which is the standard workflow.

## Key Code References
- **File**: `Cargo.toml` (root of the project)
  - **Element**: `[workspace]` table
  - **Purpose**: This is the master definition of the workspace, its members, and its shared dependencies.
- **File**: `crates/*/Cargo.toml` (e.g., `crates/koad-citadel/Cargo.toml`)
  - **Element**: `[dependencies]` table
  - **Purpose**: Shows how individual crates declare dependencies on other workspace crates (e.g., `koad-core = { path = "../koad-core" }`) and inherit from the workspace (e.g., `serde.workspace = true`).

## Configuration & Environment
- This is a build-time configuration managed entirely by Cargo. There are no runtime environment variables for the workspace itself.

## Common Questions a Human Would Ask
- "Why are there so many `Cargo.toml` files?"
- "How do I add a new crate to the project?"
- "Where should I define the version for a new dependency like `regex`?"
- "What's the difference between `koad-core` and `koad-citadel`?"
- "How do I build or check just a single crate instead of the whole project?" (Answer: `cargo check -p koad-citadel`)

## Raw Technical Notes
- The workspace structure is a direct enforcement of the "Clean Crate Boundaries" rule from `RUST_CANON`. It prevents circular dependencies and forces a clear architectural hierarchy.
- For example, `koad-core` cannot depend on `koad-citadel`, because `koad-citadel` depends on `koad-core`. This enforces `koad-core`'s role as the foundational, dependency-light base of the system.
- The use of `path` dependencies (`{ path = "../koad-core" }`) is what links the local crates together into a graph.
- This setup dramatically speeds up incremental builds. If you only change a file in `koad-citadel`, Cargo is smart enough to not recompile `koad-core` or `koad-intelligence`.
