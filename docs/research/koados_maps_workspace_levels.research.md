A conceptual exploration of **agent navigation systems** and the **Citadel → Station → Outpost workspace hierarchy** for KoadOS. This document is a knowledge resource — it focuses on principles, trade-offs, and patterns rather than implementation specifics.

---

## Part 1: Agent Navigation — The Map Metaphor

### Why maps work in games

Open-world games solve a fundamental cognitive problem: a player exists in a vast space but can only perceive a tiny slice of it at any moment. Maps solve this by providing **layered abstractions** of the environment:

- **World map** — coarse overview, major regions, rough distances. Answers: *"What exists and where is it relative to everything else?"*
- **Region map** — intermediate detail, points of interest, roads/paths. Answers: *"What's around me and how do I move between things?"*
- **Local/minimap** — immediate surroundings, real-time orientation. Answers: *"Where am I right now and what's adjacent?"*

Critically, the player never needs to hold the full world map in working memory. They zoom to the level of detail their current goal requires, extract a route or reference, and then operate locally. The map is an **external cognitive scaffold** — it offloads spatial reasoning from the player's mind to a persistent, queryable artifact.

### The agent parallel

AI agents face the same fundamental problem but in a **filesystem and information topology** rather than a physical one. An agent's "world" is the full KoadOS environment — system configs, the Citadel, multiple Stations, many Outposts, other agents' workspaces, documentation, data stores. No agent should (or can efficiently) hold all of that in context.

The game map metaphor translates directly:

| **Game concept** | **Agent equivalent** | **What it provides** |
| --- | --- | --- |
| World map | System-level topology map | All Citadels, Stations, Outposts — their existence, purpose, and relationships |
| Region map | Level-specific map (Station or Outpost) | Contents of a specific workspace level — key directories, agents present, data stores, entry points |
| Minimap / HUD | Agent's current context awareness | Where the agent is right now, what's immediately accessible, what scope it's operating in |
| Waypoints / quest markers | Task-relevant pointers | Direct references to the specific resources an agent needs for its current objective |
| Fog of war | Scope boundaries / access tiers | What an agent *can't* or *shouldn't* see at its current level — enforced or conventional |

### Core principles for agent map systems

**1. Maps are not the territory**

A map is a *compressed, purpose-built representation* of the environment — not the environment itself. An agent reading a map should get just enough information to navigate, not a full dump of every file and folder. This is the most important design tension: **fidelity vs. cognitive cost**. A map that contains everything is just the filesystem itself. A map that's too sparse forces the agent to explore blindly.

**2. Maps should be queryable, not just readable**

A static text file listing directories is a *chart*, not a map. A true navigation tool lets the agent ask questions: *"Where is the nearest `.agents/` folder above me?"* or *"Which Station owns this Outpost?"* or *"What agents are active at this level?"* The distinction matters because agents don't browse — they resolve. They have a goal and need the shortest path to the relevant resource.

**3. Maps should be level-aware**

Just as a game doesn't render dungeon interiors on the world map, an agent map should respect the workspace hierarchy. When an agent is operating at the Outpost level, the map it consults should emphasize Outpost-level detail and provide *pointers upward* to Station and Citadel resources — not inline their full contents. This is the **zoom principle**: detail increases as scope narrows.

**4. Maps should be self-maintaining (or cheaply maintained)**

The biggest risk with any metadata artifact is staleness. If maps require manual updates every time a file moves or an agent is added, they rot quickly. Ideal map systems are either:

- **Generated** — computed from the actual filesystem/environment on demand or on a schedule
- **Convention-derived** — if the workspace structure follows strict conventions (like `.agents/` always existing), the map is partially implicit and only needs to track deviations or additions
- **Event-updated** — agents that modify the environment update the relevant map as a side effect of their work

**5. Maps should support both orientation and wayfinding**

*Orientation* answers: "Where am I? What level is this? What's my scope?"

*Wayfinding* answers: "How do I get to X from here?"

Both are necessary. An agent that knows it's at an Outpost but can't resolve the path to its Station's shared config is oriented but lost. An agent that can find any file but doesn't know what level it's operating at might violate scope boundaries.

---

## Part 2: The Citadel → Station → Outpost Pattern

### The hierarchy as an information architecture

The Citadel → Station → Outpost pattern is fundamentally an **information scoping hierarchy** — each level defines a boundary of relevance:

| **Level** | **Scope** | **Contains** | **Agents typically present** |
| --- | --- | --- | --- |
| **System** | The entire machine / user environment | Global configs, system-level agent vaults (e.g., `~/.tyr`) | System administrators only |
| **Citadel** | KoadOS itself — the platform core | Core protocols, shared libraries, platform-level agent data, canonical documentation | Platform officers (Scribe, Vigil, etc.) |
| **Station** | A project hub — a domain with multiple related projects | Station-level configs, shared resources across projects, Station officer vault, inter-project coordination data | Station commander + any agents operating across projects in this domain |
| **Outpost** | A single project | Project source code, project-level agent data, local configs | Project-level crew agents + any higher-rank agents doing project work |

The key architectural insight: **each level is self-contained enough to be useful in isolation, but connected enough to inherit from above.** An Outpost doesn't need the Citadel's full protocol library loaded — it just needs to know it exists and how to reach it when needed.

### Design properties of the hierarchy

**Locality of reference**

Most of an agent's work at any given moment is local. A crew agent working on a project at the Outpost level will spend 90%+ of its operations within that Outpost's file tree. The hierarchy exploits this by keeping the most relevant data closest to the point of work. This is the same principle behind CPU cache hierarchies — L1 is small and fast (Outpost), L2 is larger and slower (Station), L3/main memory is the full system.

**Inheritance vs. isolation**

The hierarchy creates a natural tension between two needs:

- **Inheritance** — lower levels should benefit from higher-level standards, configs, and shared resources without duplicating them
- **Isolation** — lower levels should be able to operate independently without requiring full knowledge of or connectivity to higher levels

This tension is healthy and should be *designed into* the system rather than resolved entirely in one direction. Some resources genuinely need to cascade downward (e.g., canonical protocols, naming conventions). Others should be strictly local (e.g., project-specific build configs, local agent state).

**The `.agents/` folder as a universal interface**

The `.agents/` convention is arguably the most powerful element of this architecture. It provides:

- **Discoverability** — any agent arriving at any level knows exactly where to look for agent-relevant data
- **Namespace isolation** — each agent has its own subfolder (`.<agent>/`), preventing state collisions
- **Shared space** — data outside personal subfolders is communal, enabling coordination without direct agent-to-agent communication
- **Level-appropriate context** — the same agent can have different data at different levels, scoped to what's relevant there

This pattern works because it's **invariant across levels**. The `.agents/` folder at the Citadel and the `.agents/` folder at an Outpost serve the same structural purpose with the same layout conventions — only the *contents* differ based on scope.

### Resolution and escalation patterns

A critical design decision for any hierarchical system is: **when an agent can't find what it needs at its current level, what does it do?**

Several patterns exist in other domains that are worth considering:

**Upward resolution (DNS-style)**

The agent walks up the hierarchy: Outpost → Station → Citadel → System. At each level, it checks for the resource. First match wins. This is simple and predictable but can be slow for deep hierarchies and creates implicit coupling between levels.

**Manifest/registry lookup**

Each level maintains a manifest that declares what it provides and what should be resolved elsewhere. The agent consults the manifest first, which either satisfies the query or provides an explicit pointer to the correct level. Faster than blind walking but requires manifest maintenance.

**Delegation**

The agent doesn't resolve upward itself — it signals its need to a higher-level agent (e.g., the Station officer) who has the broader context to resolve the request. This keeps lower-level agents simple but introduces communication latency and dependency on the higher agent's availability.

**Cached context injection**

When an agent is initialized at a level, it receives a pre-computed "context packet" from the level above — a curated subset of higher-level information deemed relevant to that scope. The agent operates with this injected context and only escalates if it encounters something outside the packet. This minimizes runtime resolution but requires good prediction of what the agent will need.

Each of these has trade-offs in **latency**, **accuracy**, **maintenance cost**, and **agent complexity**. The right choice likely varies by use case — a crew agent doing routine project work might use cached context injection, while a system administrator agent debugging a cross-Station issue might use upward resolution.

---

## Part 3: Maps + Levels — The Combined System

### How maps serve the hierarchy

The Citadel → Station → Outpost pattern defines *what exists and where*. Maps define *how agents navigate it*. Together, they form a complete navigation system:

- The **hierarchy** is the *terrain* — the actual structure of workspaces, files, agents, and data
- The **map** is the *representation* — a queryable, compressed view of that terrain optimized for agent consumption

### Map types aligned to levels

**System Map (Atlas)**

The highest-level view. Shows the Citadel, all known Stations, and their Outposts. For each, it provides: name/identifier, purpose, active agents, and entry point paths. This is what an agent consults when it needs to understand the full KoadOS topology or find a resource that could live anywhere.

**Station Map (Chart)**

Mid-level view of a single Station. Shows the Station's Outposts, shared resources, active agents and their roles, data stores, and key configuration locations. This is what a Station-level agent consults for intra-Station navigation and coordination.

**Outpost Map (Local)**

Ground-level view of a single project. Shows the project structure, local agent presence, relevant configs, and pointers upward to Station and Citadel resources the project depends on. This is what a crew agent consults for day-to-day work.

**Agent Passport (Personal)**

Agent-specific view. Shows *this agent's* presence across levels — where its vaults are, what levels it's active at, what permissions/roles it holds at each level, and its current assignment. This is what an agent consults to understand its own position in the system.

### The bootstrap problem

When an agent first activates (or activates in a new context), it faces a cold-start problem: it doesn't yet know where it is, what level it's at, or what's available. This is the navigation equivalent of spawning in a game with no map.

The solution is a **bootstrap sequence** — a minimal, deterministic set of steps the agent always performs on activation:

1. **Orient** — determine current level (System, Citadel, Station, Outpost) from environmental cues (directory structure, presence of known marker files)
2. **Locate local map** — find the `.agents/` folder at the current level and read the local map or manifest
3. **Identify self** — locate own vault/folder at the current level, load personal context
4. **Resolve upward pointers** — if the local map references higher-level resources, note them for lazy loading
5. **Ready** — the agent now has enough context to begin work without needing the full system topology

This sequence is cheap (a handful of filesystem reads), deterministic (same steps every time), and level-agnostic (works identically whether the agent is at an Outpost or the Citadel).

### Information flow patterns

Maps don't just help agents find things — they also define how information *flows* through the hierarchy:

**Downward flow (directives & standards)**

Higher levels publish standards that lower levels inherit. The Citadel defines canonical protocols; Stations adapt them for their domain; Outposts consume them for project work. Maps facilitate this by pointing downward consumers to the canonical source.

**Upward flow (reports & signals)**

Lower levels generate information that higher levels aggregate. Outpost agents produce project status; Station officers compile cross-project summaries; Citadel officers track system-wide health. Maps facilitate this by giving higher-level agents visibility into what lower-level outputs exist and where to find them.

**Lateral flow (peer coordination)**

Agents at the same level may need to coordinate — two Outposts in the same Station sharing a dependency, or two crew agents in the same Outpost dividing work. Maps facilitate this by making peer entities discoverable within a shared scope.

---

## Part 4: Design Considerations & Trade-offs

### Static vs. dynamic maps

| **Approach** | **Strengths** | **Weaknesses** |
| --- | --- | --- |
| **Static** (file-based, manually or periodically generated) | Simple, fast to read, no runtime cost, works offline | Can become stale, requires maintenance discipline or generation tooling |
| **Dynamic** (computed on demand from filesystem) | Always accurate, zero maintenance | Runtime cost on every query, requires filesystem access, slower for large hierarchies |
| **Hybrid** (static base + dynamic overlay for volatile data) | Balances freshness and speed, static core is fast, dynamic overlay catches changes | More complex to implement, must define what's static vs. dynamic |

### Map granularity

How much detail should a map contain? Too little and agents still need to explore. Too much and the map itself becomes a cognitive burden (consuming context window for an LLM agent, processing time for a programmatic one).

A useful heuristic from cartography: **a map should contain enough information to answer the questions it's designed for, and nothing more.** A system map doesn't need file-level detail. An outpost map doesn't need to list every source file — just the structurally significant ones (entry points, configs, agent folders).

### Staleness and trust

If an agent follows a map pointer and finds the target missing or changed, it needs a fallback. Options include:

- **Graceful degradation** — the agent notes the discrepancy, falls back to exploration or upward resolution, and optionally flags the stale reference for map maintenance
- **Map refresh** — the agent triggers a map regeneration for its current level before retrying
- **Error escalation** — the agent reports the stale map to a higher-level agent responsible for maintenance

The choice depends on how critical accuracy is for the agent's current task and how expensive regeneration is.

### Agent rank and map access

Not all agents need the same map. A crew agent at an Outpost rarely needs the full system atlas. A system administrator needs all of it. Map access can follow the same scoping principle as the hierarchy itself:

- **Crew agents** → local map + upward pointers (Station, Citadel) for escalation
- **Station officers** → station chart + system map reference for cross-Station awareness
- **Citadel officers** → full system atlas + detailed maps for their domain
- **System administrators** → everything

This isn't about *restricting* access — it's about **reducing default cognitive load**. An agent can request a broader map if needed, but its default view matches its operational scope.

### The map as a coordination primitive

Beyond navigation, maps can serve as a **lightweight coordination mechanism**. If each level's map includes a section on active agents and their current focus, agents can use the map to:

- Avoid duplicate work ("Agent X is already working on this at the Station level")
- Find collaborators ("Which agent at this Outpost handles documentation?")
- Understand dependencies ("This Outpost's build depends on a shared library managed at the Station level by Agent Y")

This turns the map from a passive reference into an active coordination surface — closer to a game's multiplayer minimap than a static world chart.