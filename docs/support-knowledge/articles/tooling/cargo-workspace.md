# Cargo Workspace (koad-os)

> The multi-crate Rust workspace that structures the KoadOS project — organizing independent crates with clean boundaries, shared dependency versions, and a single build artifact cache.

**Complexity:** intermediate
**Related Articles:** [RUST_CANON](../protocols/rust-canon.md)

---

## Overview

The KoadOS project is organized as a [Cargo workspace](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html) — a collection of related Rust crates that are developed, versioned, and compiled together under a single root `Cargo.toml`. All crates live under the `crates/` directory, share a single `Cargo.lock` file, and compile into a shared `target/` directory.

This structure is the technical implementation of the [RUST_CANON](../protocols/rust-canon.md)'s "Clean Crate Boundaries" rule. By forcing all code into discrete crates with explicit `path` dependencies between them, the workspace makes the architecture visible and the dependency graph enforceable. You cannot introduce a circular dependency between crates without the build system refusing to compile.

For a developer or agent new to the project, the workspace structure is the first thing to understand — it tells you what the major components are, how they relate, and where to look for any given piece of functionality.

## How It Works

### The Root `Cargo.toml`

The root `Cargo.toml` defines two things: the workspace membership list and the shared dependency table.

**`[workspace]` — The member list:**

```toml
[workspace]
members = [
    "crates/koad-core",
    "crates/koad-proto",
    "crates/koad-intelligence",
    "crates/koad-sandbox",
    "crates/koad-codegraph",
    "crates/koad-citadel",
    "crates/koad-cass",
    "crates/koad-cli",
    # ...
]
```

Any crate not listed here is invisible to `cargo` commands run from the workspace root. When you add a new crate, you must register it in this list.

**`[workspace.dependencies]` — The shared version table:**

```toml
[workspace.dependencies]
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
anyhow = "1"
tracing = "0.1"
tonic = "0.11"
prost = "0.12"
# ...
```

Individual crates inherit these by specifying `dep.workspace = true`:

```toml
# In crates/koad-citadel/Cargo.toml
[dependencies]
tokio.workspace = true
anyhow.workspace = true
tracing.workspace = true
```

This ensures every crate in the project uses the exact same version of `tokio`, `serde`, and other shared dependencies. Without this, it's possible for two crates to depend on incompatible versions of the same library, causing subtle runtime failures or compilation errors from type mismatches.

### The Crate Hierarchy

The crates form a dependency DAG (Directed Acyclic Graph). Understanding the hierarchy tells you what depends on what:

```
koad-core          (foundation — minimal dependencies, no internal koad deps)
    ↑
koad-proto         (generated gRPC code — depends on prost/tonic)
    ↑
koad-intelligence  (AI inference layer — depends on koad-core)
koad-sandbox       (command safety — depends on koad-core)
koad-codegraph     (tree-sitter code indexing — depends on koad-core)
    ↑
koad-citadel       (binary — depends on koad-core, koad-proto, koad-sandbox)
koad-cass          (binary — depends on koad-core, koad-proto, koad-intelligence, koad-codegraph)
koad-cli           (binary — depends on koad-core, koad-proto)
```

The "no upward dependencies" rule enforces this hierarchy: `koad-core` cannot depend on `koad-citadel`. `koad-intelligence` cannot depend on `koad-cass`. This prevents circular dependencies and keeps lower-level crates lean and stable.

**Crate roles at a glance:**

| Crate | Type | Purpose |
|-------|------|---------|
| `koad-core` | Library | Shared types, traits, utilities. Foundation everything else builds on. |
| `koad-proto` | Library | Auto-generated gRPC client/server code from `.proto` files. |
| `koad-intelligence` | Library | AI inference routing (calls external LLM APIs for summarization, scoring). |
| `koad-sandbox` | Library | Command validation and policy enforcement. |
| `koad-codegraph` | Library | Tree-sitter-based code indexing and symbol lookup. |
| `koad-citadel` | Binary | The Citadel gRPC server. The "Body" of the Tri-Tier model. |
| `koad-cass` | Binary | The CASS gRPC server. The "Brain" of the Tri-Tier model. |
| `koad-cli` | Binary | The `koad-agent` command-line tool. The "Link". |

Additional crates in the workspace include `koad-board` (notification/board service) and `koad-bridge-notion` (Notion integration bridge).

### Inter-Crate Dependencies

Local crates reference each other via `path` dependencies in their individual `Cargo.toml` files:

```toml
# crates/koad-citadel/Cargo.toml
[dependencies]
koad-core = { path = "../koad-core" }
koad-proto = { path = "../koad-proto" }
koad-sandbox = { path = "../koad-sandbox" }
```

The `path` dependency tells Cargo where to find the source code. Combined with the workspace's shared `Cargo.lock`, this means all crates always build against the same version of each other.

### Running `cargo` Commands

From the workspace root (`~/.koad-os/`), `cargo` commands operate on all crates:

```bash
# Check all crates for compilation errors
cargo check

# Build all crates
cargo build

# Run all tests across all crates
cargo test

# Run clippy on all crates
cargo clippy -- -D warnings
```

To target a specific crate, use the `-p` (package) flag:

```bash
# Check only koad-citadel
cargo check -p koad-citadel

# Run only koad-cass tests
cargo test -p koad-cass

# Build only the koad-cli binary
cargo build -p koad-cli
```

The shared `target/` directory means compiled artifacts are reused across crates. If you compile `koad-citadel` and then compile `koad-cass`, the `koad-core` artifacts built for `koad-citadel` are reused for `koad-cass` — no recompilation unless `koad-core` changed. This is a significant build-time optimization in large workspaces.

## Configuration

The workspace has no runtime configuration. All configuration is build-time, managed by Cargo.

| File | Purpose |
|------|---------|
| `Cargo.toml` (root) | Workspace definition: member list and shared dependencies |
| `Cargo.lock` (root) | Locked dependency versions for all crates; commit this file |
| `crates/*/Cargo.toml` | Per-crate configuration: name, version, local deps, feature flags |
| `target/` (root) | Compiled artifact cache; not committed, gitignored |

## Failure Modes & Edge Cases

**Adding a new crate but forgetting to register it in the workspace.**
`cargo check` and `cargo build` from the workspace root will not see the new crate. The crate can still be compiled from within its own directory (`cd crates/my-new-crate && cargo build`), but it won't be part of the CI pipeline or workspace-wide commands. Fix: add `"crates/my-new-crate"` to the `members` array in root `Cargo.toml`.

**Version conflict with a `workspace = true` dependency.**
If a crate specifies both `workspace = true` and a local version override, Cargo may error or silently prefer one. The canon rule: always use `workspace = true` for any dependency defined in `[workspace.dependencies]`. If you need a different version, update the workspace-level definition (after ensuring no other crate breaks).

**Adding a dependency that creates a circular dependency.**
Cargo will refuse to build: "package 'koad-core' depends on 'koad-citadel', which depends on 'koad-core'". The fix is to move the shared code into `koad-core` (or a new intermediate crate) so both crates can depend on it without a cycle.

**Slow full rebuilds.**
The `target/` directory can grow very large (several GB) for a workspace of this size. Use `cargo check` (no codegen) instead of `cargo build` during development for fast feedback. Use `-p <crate>` to limit the scope of build commands. Running `cargo clean` to clear `target/` resolves disk-space issues but requires a full rebuild on next compile.

## FAQ

### Q: Why are there so many `Cargo.toml` files?
Each `Cargo.toml` in `crates/*/` is the manifest for one independent crate. Each crate has its own name, version, dependencies, and feature flags. The root `Cargo.toml` is the workspace manifest — it doesn't define a crate itself, just the collection. Multiple `Cargo.toml` files are fundamental to how Cargo workspaces work. It's not complexity for complexity's sake; each file corresponds to a meaningful software module with its own identity and dependency set.

### Q: How do I add a new crate to the project?
Three steps: (1) Create the crate directory: `cargo new --lib crates/my-new-crate` (or `--bin` for a binary). (2) Add the crate to the workspace members list in the root `Cargo.toml`: `"crates/my-new-crate"`. (3) Add the standard RUST_CANON headers to `src/lib.rs` (`//!` module doc, `#![warn(missing_docs)]` for libraries). Then run `cargo check -p my-new-crate` to verify it builds correctly.

### Q: Where should I define the version for a new dependency like `regex`?
In `[workspace.dependencies]` in the root `Cargo.toml`, if the dependency is likely to be used by more than one crate. Then in each crate's `Cargo.toml` that needs it, use `regex.workspace = true`. If it's truly a one-off dependency used by a single crate with no expectation of reuse, you may define it locally in that crate's `Cargo.toml`. The RUST_CANON prefers the workspace table for any commonly-used crate to prevent version drift.

### Q: What's the difference between `koad-core` and `koad-citadel`?
`koad-core` is a library crate — it defines shared types, traits, and utilities that other crates use. It has no `main()` function, produces no binary, and has deliberately minimal dependencies. It is the foundation of the codebase. `koad-citadel` is a binary crate — it produces the `koad-citadel` executable, the long-running gRPC server that manages agent sessions. `koad-citadel` depends on `koad-core` (for shared types and utilities), but `koad-core` does not depend on `koad-citadel`.

### Q: How do I build or check just a single crate instead of the whole project?
Use the `-p` flag: `cargo check -p koad-citadel` or `cargo build -p koad-cass`. This compiles only the specified crate and its dependencies (not the entire workspace). This is the standard way to iterate quickly on a specific component without waiting for unrelated crates to compile. For running tests, `cargo test -p koad-core` runs only `koad-core`'s test suite.

## Source Reference

- `Cargo.toml` (workspace root) — `[workspace]` member list and `[workspace.dependencies]` shared versions
- `Cargo.lock` (workspace root) — All locked dependency versions; canonical source of truth for reproducible builds
- `crates/koad-core/Cargo.toml` — Example of a minimal library crate manifest
- `crates/koad-citadel/Cargo.toml` — Example of a binary crate with multiple workspace and local path dependencies
