<aside>
📜

**Scribe Task Prompt.** Direct command from Ian. Scribe scouts the `~/.koad-os` workspace from scratch, audits traversal efficiency for agents, produces a scored assessment, and delivers the canonical **System Map** that all agents reference for navigation.

</aside>

---

## Objective

Perform a **fresh, ground-truth traversal** of the entire `~/.koad-os` directory. Do not rely on any prior maps, layout docs, or architectural summaries produced by other agents — those may be stale. You are the source of truth now.

Deliver two artifacts:

1. **Traversal Audit Report** — a scored assessment of how much friction an agent encounters navigating the workspace
2. **KoadOS System Map** — the canonical index/manifest of folder structures, key file locations, and navigation hints that all agents will use for orientation

**You own the System Map going forward.** When the workspace changes, you update it. Other agents should never need to `ls` their way through the tree when the System Map exists.

---

## Context

Agent token burn during workspace traversal is a top-priority concern. Every `ls`, `cat`, and `find` an agent runs to figure out *where things are* costs tokens and requests that should be spent on actual work. A centralized, maintained System Map eliminates redundant discovery across all agents and sessions.

This task aligns with the DRAFT_PLAN_2 Realignment priority: **reduce agent token consumption and request overhead.**

---

## Phase 1 — Fresh Directory Scout

Traverse `~/.koad-os` recursively. **Start from zero assumptions.** Use `ls`, `tree`, `find`, or whatever tooling is available to map the full directory structure.

For every directory and notable file, record:

- **Path** — full path from `~/.koad-os/`
- **Type** — directory, config, source, doc, data, binary, socket, template, log, unknown
- **Purpose** — one-line description of what this file/directory does
- **Owner** — which agent or system component is the primary consumer/maintainer (if determinable)
- **Staleness signal** — last modified date or any indicator of whether this is active, stale, or deprecated

### How to determine purpose

- **Read signpost files first:** `README.md`, `AGENTS.md`, `IDENTITY.md`, `CLAUDE.md`, `Cargo.toml`, and TOML headers in each directory. These are your primary evidence.
- **Read doc directories:** If a `docs/` folder exists anywhere in the tree, read its contents — these are likely the most current architectural references.
- **For Rust crates (`crates/`):** Read only `Cargo.toml`, `mod.rs`, and `main.rs` (or `lib.rs`) entry points to determine crate purpose. Do **not** deep-read full source files.
- **For config files (`config/`):** Read headers and key names only. Respect the **Sanctuary Rule** — read-only, no edits.
- **If purpose is unclear:** Mark it `UNKNOWN — needs annotation` and flag it in the audit. Do not guess.
- **Do not read:** database files (`*.db`), socket files (`*.sock`), binary artifacts, or `target/` build directories.

### Repo and external docs

If there is a linked GitHub repo, documentation hub, or `new_world/` planning directory within `~/.koad-os`, scout those too. They may contain the most current architectural plans, specs, and layout definitions. Include them in the map with a clear annotation that they are planning/reference docs vs. runtime artifacts.

---

## Phase 2 — Traversal Friction Audit

Using the directory map from Phase 1, evaluate traversal friction for a **cold-starting Tier 1 agent** (e.g., Tyr booting fresh with no cached context). Score each dimension on a **1–5 scale** (1 = minimal friction, 5 = severe friction):

| **Dimension** | **What to Measure** |
| --- | --- |
| **Discoverability** | Can an agent find a file/directory by name or convention alone, without reading docs first? Are naming conventions consistent and predictable? |
| **Depth Cost** | How many directory levels deep must an agent traverse to reach commonly needed files? Count average and worst-case hops from root to frequently needed resources. |
| **Signposting** | Are there README/[AGENTS.md](http://AGENTS.md) files at key decision points? Do directory names self-describe their contents? Can an agent navigate the tree by reading names alone? |
| **Cross-Reference Clarity** | When a file references another file (TOML paths, [AGENTS.md](http://AGENTS.md) links, spec references), are those references resolvable without guessing? Are paths absolute, relative, or broken? |
| **Cold-Start Token Cost** | Estimate how many tool calls (`ls`  • `cat`) and approximate tokens a Tier 1 agent would burn to go from zero context to "I know where everything is and what it does." Compare to the ideal: reading one System Map file. |
| **Dead Weight** | How much of the workspace is stale, deprecated, duplicated, or orphaned? Dead weight increases traversal noise — agents waste tokens reading things that don't matter. |

### Composite Score

After scoring all dimensions, compute a **Traversal Efficiency Score (TES)**:

`TES = 30 - (sum of all dimension scores)`

Scale: **30** = frictionless workspace, **0** = agents cannot self-orient.

Interpretation:

- **25–30:** Workspace is well-organized. System Map is a convenience, not a necessity.
- **18–24:** Workspace is navigable but has friction points. System Map adds clear value.
- **10–17:** Significant friction. Agents are burning meaningful tokens on orientation. System Map is essential.
- **0–9:** Workspace is disorganized. Agents cannot reliably self-orient. Restructuring recommended before the System Map can be effective.

---

## Phase 3 — Produce the System Map

Write the canonical **KoadOS System Map** as a standalone file. This is the file that every agent reads at boot (or on-demand) instead of traversing the workspace themselves.

### System Map format

```
# KoadOS System Map
# Generated: [ISO date] | Author: Scribe | TES: [score]/30
# This file is the canonical workspace index. All agents should
# reference this instead of traversing ~/.koad-os directly.
# Maintained by Scribe. Notify Scribe when workspace structure changes.

## Quick Reference — Most Accessed Paths
[Table: Path | What It Is | When You Need It]
(Top 10-15 most commonly needed files/dirs for agent operations)

## Full Directory Tree
[Annotated tree output with one-line purpose annotations]

## Config Files Index
[All TOML/config files with: path, purpose, which agents use them]

## Agent Bays Index
[All agent personal bays with: path, agent name, bay status]

## Crate/Module Index
[All Rust crates with: path, binary/library, purpose, entry point]

## Documentation Index
[All docs, specs, plans, research with: path, subject, staleness]

## Cross-Reference Map
[Key files that reference other files, with the dependency direction]

## Stale/Deprecated Items
[Anything flagged as stale, orphaned, or deprecated in Phase 1]

## Navigation Tips
[Short list of "if you need X, go to Y" hints for common agent tasks]
```

### System Map rules

- **Output < Input.** The System Map must be significantly shorter than the raw traversal data. If you read 100K tokens of workspace, the map should be 3–8K tokens. Compress aggressively.
- **No opinions.** The map says what *is* and where it *is*. Assessment and recommendations belong in the Audit Report, not the map.
- **Stable paths only.** Do not index `target/`, temp files, build artifacts, or anything that changes on every compile.
- **Agent-addressed navigation tips.** The "Navigation Tips" section should be written from the perspective of "you are an agent that just booted and needs to find X."

---

## Deliverables

| **Artifact** | **Location** | **Format** |
| --- | --- | --- |
| **Traversal Audit Report** | `~/.koad-os/.agents/.scribe/reports/[DATE]_traversal-audit.md` | Scout Report template (Key Facts, dimension scores, composite TES, flagged items) |
| **KoadOS System Map** | `~/.koad-os/SYSTEM_MAP.md` | System Map format defined above |

The System Map lives at the **root of `~/.koad-os/`** so every agent can find it with a single read — no traversal required.

The Audit Report lives in your reports bay per standard Scribe bay layout.

---

## Operating Constraints

- **Read-only traversal.** Do not create, modify, or delete any files during Phase 1 and Phase 2. You only write during Phase 3 (the two deliverables).
- **Sanctuary Rule in effect.** `config/*.toml`, `koad.db`, `*.sock` — read-only, no exceptions.
- **CIP boundaries apply.** Do not read raw cognitive state, session keys, or lease data belonging to other agents.
- **Token discipline.** You are running on Flash-Lite. Be efficient. Do not `cat` entire source files when `head` or reading the first 20 lines gives you enough to determine purpose.
- **No assumptions from prior maps.** If you find a `SYSTEM_MAP.md` or prior layout doc already in the workspace, read it for comparison but do **not** trust it as current. Your traversal is the ground truth.

---

## Success Criteria

- The System Map, when read by a cold-starting Tier 1 agent, provides enough information to navigate to any file in `~/.koad-os` without additional `ls` or `find` calls.
- The Traversal Audit Report gives Ian a clear picture of where navigation friction exists and how severe it is.
- Both artifacts are complete, honest (UNKNOWN items are flagged, not guessed), and follow the token efficiency mandate (output significantly shorter than input).

---

<aside>
⚡

**Workflow:** Ian hands this prompt to Scribe in Gemini CLI → Scribe executes Phases 1–3 sequentially → Scribe writes both deliverables → Scribe runs a saveup.

</aside>