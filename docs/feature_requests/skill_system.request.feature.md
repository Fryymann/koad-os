## Purpose

Research and design a **KoadOS-native Skill System** — a structured framework for packaging reusable agent behaviors, workflows, and domain knowledge into discoverable, composable units that KoadOS agents can trigger, load, and execute.

This request originates from a deep analysis of Anthropic's Claude Code "Skill Creator" skill. Noti performed a pattern extraction identifying eight architectural patterns that map directly onto existing KoadOS concepts (CASS memory tiers, Body/Ghost model, pipeline pattern, agent taxonomy). This feature request captures the *what* and *why* — implementation specs, templates, and code are deferred to codebase agents (Tyr, Sky) with direct access.

---

## Source Material

- **Analyzed artifact:** Claude Code's `skill-creator` skill (full instruction set captured in [Claude Code Skill Creator](https://www.notion.so/Claude-Code-Skill-Creator-323fe8ecae8f80229472c10a78116191?pvs=21))
- **Analysis session:** Noti × Ian, 2026-03-14
- **Related prior art:** Agent Boot Research — CLI Context Injection Patterns, Context Hydration Architecture research

---

## Problem Statement

KoadOS agents currently lack a standardized mechanism for:

1. **Packaging reusable behaviors** — workflows, domain expertise, and operational patterns are embedded in agent instructions or scattered across docs. There's no portable, composable unit.
2. **Dynamic capability loading** — agents load their full identity at boot but can't selectively acquire new capabilities mid-session based on task context.
3. **Skill discovery and triggering** — no semantic routing layer exists to match user intent against available agent capabilities and load only what's relevant.
4. **Iterative skill improvement** — no eval/feedback loop to validate that a packaged behavior actually works well across diverse inputs.

The Claude Code Skill Creator solves all four problems within its ecosystem. KoadOS should solve them within ours — but natively, using our architecture.

---

## Extracted Patterns — Research Summary

The following patterns were identified from the Claude Code skill system. Each includes a mapping to KoadOS architecture.

### Pattern 1: Progressive Disclosure (Three-Tier Context Loading)

**What Claude does:** Skills load in three tiers — metadata (~100 words, always in context), [SKILL.md](http://SKILL.md) body (<500 lines, on trigger), bundled resources (unlimited, on demand).

**KoadOS mapping:** Maps directly to CASS memory tiers. Skill metadata → L1 Redis (hot, always available). Skill body → L2 SQLite (warm, loaded on activation). Deep references → L3 Qdrant or filesystem (cold, loaded on explicit need). The context hydration pipeline already has a three-tier model — skills should plug into it.

**Key insight:** Context window is a scarce resource. Every skill should have a *context cost profile* declared upfront.

### Pattern 2: Phased Convergence Loops

**What Claude does:** Skills aren't linear procedures — they're iterative loops with named phases and re-entry logic. The agent orients to wherever the user is in the process and jumps in.

**KoadOS mapping:** This is the pipeline pattern from the Agent Taxonomy. Skills should define phases with entry conditions, not just sequential steps. An agent should be able to resume a skill mid-workflow after a session break (via EndOfWatch + CASS session restore).

### Pattern 3: Description as Semantic Router

**What Claude does:** The skill description field is the *sole trigger mechanism*. The system matches user intent against description text to decide whether to load the skill body. Descriptions are written slightly "pushy" to avoid under-triggering.

**KoadOS mapping:** This is a routing layer problem. KoadOS would need a skill registry (likely a CASS-managed index) where each skill has an intent signature. On user input, the routing layer scores available skills against the input and loads the highest-match skill's body into context. Could leverage Qdrant semantic search for fuzzy matching.

### Pattern 4: Environment Adaptation / Graceful Degradation

**What Claude does:** Skills declare capability dependencies and provide fallback behavior per runtime context (full subagents vs. no subagents vs. no browser).

**KoadOS mapping:** Maps to the Body/Ghost model and Citadel disconnect states. A skill should declare what it needs (`requires: [cass, redis, mcp_tools]`) and define degraded behavior when dependencies are unavailable. If CASS is offline, the skill operates from local ghost config only. If subagents aren't available, the skill falls back to inline execution.

### Pattern 5: Subagent Delegation via Role References

**What Claude does:** Complex skills decompose into role-specific sub-skills (`agents/grader.md`, `agents/comparator.md`) loaded lazily when the workflow needs them.

**KoadOS mapping:** This is the pipeline pattern + Personal Bay model. A skill can reference other skills or agent roles by name. The orchestrating agent holds the workflow graph; delegates load their own skill context on activation. Cross-agent skill invocation could route through A2A-S signals.

### Pattern 6: Eval-Driven Development

**What Claude does:** Skills ship with test cases, assertion frameworks, grading agents, and human-in-the-loop review. Eval is baked into the skill lifecycle, not bolted on.

**KoadOS mapping:** This is new territory for KoadOS. A skill's `_eval/` directory would contain test prompts and expected behaviors. The Integrity Audit Protocol could be extended to cover skill validation. KSRP could include a "skill review" pass.

### Pattern 7: Anti-Overfitting / Mentorship Tone

**What Claude does:** Instructions explain *why* before *what*. Explicitly warns against rigid ALWAYS/NEVER patterns. Treats the consuming agent as a smart collaborator, not a mechanical executor.

**KoadOS mapping:** This is a canon-level principle. Skills should be written for class-level coverage, not instance-level scripting. Aligns with the Contributor & Coding Manifesto's philosophy. Should be codified in a "Skill Writing Guide" that ships with the system.

### Pattern 8: Bundled Reusable Scripts

**What Claude does:** When test runs reveal that agents independently write the same helper scripts, those scripts get bundled into the skill's `scripts/` directory. DRY principle applied to agent-generated code.

**KoadOS mapping:** Skills can bundle executable scripts (bash, Python, Rust) in a `scripts/` directory. These run without being loaded into context — agents invoke them by path. Aligns with KoadOS's programmatic-first communication principle.

---

## Proposed Skill Anatomy (Conceptual)

This is a *starting point* for Tyr to evaluate, not a final spec.

```
skill-name/
├── SKILL.md              # Required. Frontmatter (name, description, requires) + instructions
├── scripts/              # Optional. Executable code for deterministic tasks  
├── references/           # Optional. Deep-load docs (ToC if >300 lines)
├── agents/               # Optional. Role-specific sub-skill instructions
└── _eval/                # Optional. Test prompts, assertions, grading schema
```

**Frontmatter fields (candidate):**

```yaml
name: skill-name
description: "Intent signature — when to trigger, what it does"
requires: [cass, redis]           # Capability dependencies
tier: officer | crew | micro      # Which agent tier can use this
context_cost: small | medium | large
```

---

## Integration Points with Existing KoadOS Architecture

| KoadOS System | Skill System Touchpoint |
| --- | --- |
| **CASS Memory Tiers** | Skill metadata (L1), body (L2), deep refs (L3) |
| **Body/Ghost Boot** | Skills loaded during context hydration at ghost prepare |
| **Agent Taxonomy** | Skills tagged by tier (Officer, Crew, Micro) |
| **Pipeline Pattern** | Skills define phased workflows with re-entry |
| **A2A-S Signals** | Cross-agent skill invocation via signal protocol |
| **KSRP** | Skill review pass added to code review protocol |
| **EndOfWatch** | Skill state persisted for session resume |
| **Signal Corps** | `skill:loaded`, `skill:completed` events on event bus |
| **KoadConfig TOML** | Skill registry in `~/.koad-os/config/skills/` |
| **Token Audit** | Skills declare context_cost; audit tracks skill token burn |

---

## Open Questions for Tyr

1. **Registry location** — Should skills live in `~/.koad-os/skills/`, in the project station, or both? Global skills vs. station-local skills?
2. **Discovery mechanism** — Qdrant semantic search against descriptions, or simpler keyword/tag matching? What's the right complexity for v1?
3. **Loading mechanism** — Does `koad-agent` load skills at boot (static), or does CASS inject them mid-session (dynamic)? Or both (boot-loaded defaults + dynamic on-demand)?
4. **Skill authoring** — Who writes skills? Only agents with codebase access? Can Ian author skills via Notion and have them synced?
5. **Relationship to existing [AGENTS.md](http://AGENTS.md)** — Are skills a *replacement* for the three-tier context file hierarchy, a *supplement*, or a completely separate system?
6. **Eval infrastructure** — Is skill eval a Phase 6 feature (Memory System Advanced), or should a minimal eval framework ship earlier?
7. **Cross-agent skill sharing** — If Sky writes a skill, can Tyr use it? Permission model? Aligns with TCH sharing permissions?

---

## Recommended Investigation Protocol

<aside>
📋

**For Tyr:** Follow the same Phase A–D structure used in the Context Hydration feature request. Inventory what exists → gap analysis → usage audit → proposal with tradeoffs.

</aside>

**Phase A — Inventory:** Audit existing KoadOS constructs that already function as proto-skills ([AGENTS.md](http://AGENTS.md) files, agent instruction pages, bundled scripts in `.koad-os/`, any `koad scan`/`koad analyze` tools).

**Phase B — Gap Analysis:** Compare current state against the eight patterns above. Which patterns are already partially covered? Which are net-new?

**Phase C — Architecture Fit:** Evaluate where the skill system sits in the Citadel Refactor phasing. Is this a Phase 4 (koad-agent) feature? Phase 5 (Integration)? Phase 6 (Memory Advanced)? Or does it span multiple phases?

**Phase D — Proposal:** Draft a scoped v1 proposal. Recommend what ships first (minimal viable skill system) vs. what's gated on CASS/Qdrant maturity.

---

## Rules of Engagement

1. **No implementation.** This is research and proposal. No feature code.
2. **Noti's role:** Deep research, external pattern analysis, knowledge synthesis. This page is Noti's deliverable.
3. **Tyr's role:** Codebase audit, architecture fit, implementation proposal, phasing recommendation.
4. **Dood approves** the path forward after Tyr's proposal.
5. **KSRP self-review** on the final report before delivery.

---

## Delivery

- Tyr writes the investigation report to `.koad-os/docs/features/skill-system-architecture.md`
- Summarize top findings and key recommendation in chat or KoadStream.
- Await Dood approval before any follow-up action.