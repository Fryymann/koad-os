# The Workspace Hierarchy

> A four-tier logical model that maps the filesystem into scoped zones (System → Citadel → Station → Outpost), controlling what context an agent can access and what operations it can perform based on its current working location.

**Complexity:** intermediate
**Related Articles:** [The Tri-Tier Model](./tri-tier-model.md), [koad-cass](../core-systems/koad-cass.md), [koad-agent boot](../core-systems/koad-agent-boot.md)

---

## Overview

KoadOS uses a hierarchical model of the filesystem to control agent context and enforce operational boundaries. This model — called the Workspace Hierarchy — maps any directory on the machine to one of four levels: System, Citadel, Station, or Outpost. An agent's active level determines what files it loads as context, how much of the filesystem it can access, and which operations the Citadel permits.

The motivation is practical: AI agents have finite token budgets. Without a scoping model, a context hydration system might try to load files from across an entire machine, burning budget on irrelevant material. The Workspace Hierarchy solves this by saying: "if you're working in a single git repository, load only that repository's context." It's the technical implementation of **Locality of Reference** — the principle that the most relevant information to your current task is almost always close to where you're working.

The hierarchy also forms the basis of the **Sanctuary Rule**: agents are expected to operate within their designated level and not reach above it without explicit authorization. A Crew-rank agent working at the Outpost level has no business modifying files at the System level.

The four levels map to increasingly broad operational scope, described in KoadOS as a "Game Map Metaphor":

| Level | Name | Scope | Typical Agent |
|-------|------|-------|---------------|
| 4 | **System** | The entire machine (`/`) | Admiral (Ian) only |
| 3 | **Citadel** | The KoadOS platform directory (`~/.koad-os/`) | Scribe, Vigil, Captain-rank |
| 2 | **Station** | A project hub containing multiple repos | Lead agents on multi-repo work |
| 1 | **Outpost** | A single git repository | Crew agents on feature work |

## How It Works

### Level Resolution at Boot

When an agent runs `koad-agent boot`, the CLI passes the agent's current working directory to the Citadel's `CreateLease` RPC. The Citadel uses the `HierarchyManager` (from `koad-core`) to resolve the level.

`HierarchyManager::resolve_level()` works by walking up the directory tree from the given path. It looks for specific markers at each level:
- A directory named `.koad-os` (or containing a `kernel.toml`) resolves to **Citadel** level
- A `koad.toml` file containing a `[station]` section resolves to **Station** level
- A `.koad` marker file or a `koad.toml` file resolves to **Outpost** level
- If none of the above are found, the level falls back to **System**

The resolved level is embedded in the session token the Citadel issues. Every subsequent gRPC call from the agent carries this level in its metadata, making all services aware of the agent's operational scope.

### Level-Aware Context Hydration (TCH)

The primary consumer of the hierarchy level is CASS's **Temporal Context Hydration** (TCH) service, implemented in `CassHydrationService::hydrate()`. When an agent boots and requests its initial context packet, CASS uses the level to decide how broadly to search:

- **Outpost (Level 1):** Walk only the current repository. Load `AGENTS.md`, local `.agents/` folder contents, recent commits, and nearby source files. Do not traverse to parent directories.
- **Station (Level 2):** Walk the current repo plus high-level pointers from sibling repositories in the Station (e.g., their README files and `.agents/` summaries). Do not load full source trees from sibling repos.
- **Citadel (Level 3):** Load cross-cutting KoadOS platform documentation, shared config, and canonical plans. Appropriate for agents doing platform-level work.
- **System (Level 4):** Reserved for administrative agents (Admiral/Dood). Unrestricted.

This graduated approach prevents **context pollution**: an agent working on a single feature in a single repo doesn't need (or want) context from unrelated repositories in the same Station.

### Tokens and Token Budgeting

The `Hydrate` RPC accepts a `token_budget` field. CASS respects this budget by:
1. Prioritizing high-value sources first (recent `EndOfWatch` summaries, high-significance `FactCard`s, the local `AGENTS.md`)
2. Progressively adding lower-priority material until the budget is exhausted
3. Truncating or summarizing large files rather than including them whole

The hierarchy level sets the *scope* of what's eligible to load; the token budget controls *how much* of that eligible content actually gets loaded.

### The Sanctuary Rule (gRPC Enforcement)

The hierarchy is enforced at the gRPC level, not the OS level. An agent with a shell can technically `cd` anywhere on the filesystem — the hierarchy doesn't create OS-level chroot jails. The enforcement happens when the agent makes a privileged gRPC call:

- The Citadel's interceptor validates the session token (which contains the level)
- The `Sector` service's `ValidateIntent` RPC routes operations through `koad-sandbox`, which checks the command and path against policies appropriate for the agent's level and rank

The `koad-sandbox` is the next line of defense. It can reject or sandbox commands that exceed the agent's authorized scope, regardless of what the shell allows.

### Defining Stations and Outposts

A developer can explicitly define a directory as a Station or Outpost by placing a `koad.toml` file in it:

```toml
# koad.toml — marks this directory as a Station named "skylinks"
[station]
name = "skylinks"
description = "Multi-repo hub for the Skylinks project suite"
```

Without a `koad.toml`, the `HierarchyManager` falls back to heuristics (directory names, presence of `.git`). Explicit `koad.toml` files are recommended for shared projects.

## Configuration

| Key | Location | Description |
|-----|----------|-------------|
| `.koad` | Project root | Marker file/directory that explicitly identifies a directory as an Outpost |
| `koad.toml` | Project or Station root | Configuration file defining a Station or Outpost; used by `HierarchyManager` for level resolution |
| `token_budget` | `HydrationRequest` field | Maximum tokens CASS may use for the context packet returned by TCH |

## Failure Modes & Edge Cases

**What if the level is resolved incorrectly?**
If `HierarchyManager` doesn't find any markers, it defaults to System level — the broadest scope. This is safe but inefficient: the agent will appear to be at System level, which may cause TCH to attempt to load more material than necessary. The fix is to add a `koad.toml` to the target directory to make the level explicit.

**The Sanctuary Rule is advisory at the shell level.**
An agent with direct shell access can `cd` to any directory and read any file their OS user has permission to access. The hierarchy does not prevent this. The Sanctuary Rule is enforced at the gRPC boundary: privileged operations (tool execution, file writes routed through `koad-sandbox`, resource locks) are validated against the agent's level and rank. Agents that bypass the gRPC layer and act directly via shell are outside the Citadel's visibility.

**Station context can become expensive.**
At Station level (Level 2), TCH walks multiple repositories. If a Station contains large repos, context compilation can be slow and token-intensive. The token budget parameter is the primary mitigation — set it conservatively for agents that don't need cross-repo context.

## FAQ

### Q: How does the system know if I'm in a repository versus a larger project?
The `HierarchyManager` in `koad-core` walks up the directory tree from the agent's current working directory, looking for specific markers: `.koad` files, `koad.toml` files with a `[station]` section, or the presence of the `~/.koad-os/` directory structure. The first match determines the level. If no markers are found, it defaults to System level. You can force a level by placing a `koad.toml` in the target directory.

### Q: What prevents an agent from accessing files outside its designated Outpost?
The Workspace Hierarchy is enforced at the gRPC layer, not the OS layer. An agent can technically `cd` out of its workspace in the shell. What it *cannot* do is invoke privileged operations (like requesting a file write, acquiring a resource lock, or executing a sandbox-validated command) on paths outside its scope — the Citadel's `Sector` service and `koad-sandbox` will reject these. The hierarchy is about controlling what the Citadel cooperates with, not restricting raw shell access.

### Q: How does the hierarchy help save tokens?
By scoping what CASS's Temporal Context Hydration (TCH) considers eligible to load. An agent at Outpost level only gets context from its current repository. At Station level, it gets cross-repo summaries. At Citadel level, it gets platform-wide documentation. This prevents an agent working on a single feature from receiving megabytes of context from unrelated repositories. Combined with the `token_budget` parameter on hydration requests, agents get focused, relevant context instead of noisy, token-expensive dumps.

### Q: Can I define my own Station for a new project?
Yes. Create a `koad.toml` file in your project hub directory with a `[station]` section. The `HierarchyManager` will recognize this directory as a Station when an agent boots from within it (or any of its subdirectories, until a more specific marker is found). You can also define individual repositories as Outposts with their own `koad.toml` files.

### Q: What is the `HierarchyManager` and where does it run?
`HierarchyManager` is a struct defined in `crates/koad-core/src/hierarchy.rs`. It runs inside the Citadel process — it's invoked during `CreateLease` to resolve the level from the agent's current working directory. The resolved level is then embedded in the session token and available to all services (including CASS) throughout the agent's session. It's part of `koad-core` (not `koad-citadel`) because the hierarchy resolution logic is considered a core utility that other components might need.

## Source Reference

- `crates/koad-core/src/hierarchy.rs` — `HierarchyManager` struct and `resolve_level()` method; the core level-resolution logic
- `crates/koad-core/src/config.rs` — `KoadConfig` and `ProjectConfig` structs; how `koad.toml` files are parsed
- `crates/koad-cass/src/services/hydration.rs` — `CassHydrationService`; primary consumer of hierarchy level for context loading
- `crates/koad-citadel/src/services/session.rs` — Where `HierarchyManager::resolve_level()` is called during lease creation
