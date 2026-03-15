**Scribe — Agent Information System: Navigation Mapping & [AGENTS.md](http://AGENTS.md) Update**

**Objective:** Establish a weighted navigation layer across the KoadOS Agent Information System (AIS) so agents can traverse the filesystem efficiently — using cross-linked reference points rather than blind exploration.

---

**Phase 1 — Navigation Audit**

Using your `agent_info_system` map, identify all files and directories that would benefit from outbound links to other docs. For each candidate, determine:

- What it *is* (entry point, reference doc, config, spec, etc.)
- What other files an agent arriving here would *likely need next*
- Whether it should serve as a **traversal node** (hub linking to many others), a **reference leaf** (linked to, not linking out), or a **bidirectional bridge**

Produce a prioritized list of link insertions — file path, target path, and a one-line rationale for each.

---

**Phase 2 — `~/.koad-os/AGENTS.md` Revision**

Review and update `~/.koad-os/AGENTS.md` to:

1. Add explicit hooks to `.agents/CREW.md` and `.agents/CITADEL.md` — including a short description of what each contains and when an agent should load it
2. Add a **"Beneficial Information Docs"** section that guides agents toward high-value reference material based on their current task context (e.g., "If working on agent orchestration → load [CITADEL.md](http://CITADEL.md); if resolving identity/role questions → load [CREW.md](http://CREW.md)")
3. Ensure the file reads as a **routing document first** — it should tell an agent *where to go*, not repeat content that lives elsewhere

---

**Success Criteria:**

- An agent cold-starting in `~/.koad-os/` can reach any critical doc in ≤2 hops
- `AGENTS.md` functions as a lightweight weighted map, not a monolithic spec
- No doc duplication — links replace redundancy