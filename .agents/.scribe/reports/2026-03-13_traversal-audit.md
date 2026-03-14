# Traversal Audit Report — 2026-03-13

## Key Facts
- **Date:** 2026-03-13
- **Author:** Scribe
- **Workspace:** `~/.koad-os/`
- **Current Status:** Phase 1 — Citadel MVP Construction

## Friction Assessment

| **Dimension** | **Score (1-5)** | **Observation** |
| --- | --- | --- |
| **Discoverability** | 2 | Naming is generally logical (e.g., `config/`, `crates/`), but the split between `new_world/`, `docs/`, and `legacy/` creates initial ambiguity for task-specific context. |
| **Depth Cost** | 3 | Average depth of 3 hops to reach active source/config. Frequent needs (identities, proto) are buried under subdirectories. |
| **Signposting** | 1 | Excellent use of `AGENTS.md`, `CITADEL.md`, and TOML headers. The "onboarding" path is clear if an agent starts at the root. |
| **Cross-Reference Clarity** | 2 | Internal links in docs are generally valid. Relative paths are used consistently. |
| **Cold-Start Token Cost** | 4 | High risk of token burn without a map. Understanding the `legacy/` graveyard vs. active `crates/` requires multiple `ls` and `cat` calls. |
| **Dead Weight** | 4 | `legacy/` contains mirror structures (`agents/`, `crates/`, `docs/`) that are easily mistaken for active files by naive search tools. |

## Traversal Efficiency Score (TES)
**Score: 14/30**
*Interpretation: Significant friction. Agents are burning meaningful tokens on orientation. System Map is essential.*

## Flagged Items
- **Shadow Paths:** `legacy/agents/` vs `.agents/`. Naive agents might read stale personae.
- **Plan Drift:** `new_world/` contains multiple versions of "DRAFT_PLAN". Only `DRAFT_PLAN_3.md` is active.
- **Orphaned Logs:** Large log files in root (`redis.log`, `kspine.log.2026-03-13`) clutter traversal.

## Recommendations
1. **Promote the Map:** All agents MUST read `SYSTEM_MAP.md` on boot.
2. **Prune Legacy:** Consider moving `legacy/` out of the primary worktree or adding a `.geminiignore` entry to prevent accidental scouting.
3. **Log Rotation:** Move active logs to a dedicated `logs/` directory.
