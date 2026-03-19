## Purpose
Research and design a KoadOS-native Skill System — a structured framework for packaging reusable agent behaviors, workflows, and domain knowledge into discoverable, composable units that KoadOS agents can trigger, load, and execute.
This request originates from a deep analysis of Anthropic's Claude Code "Skill Creator" skill. Noti performed a pattern extraction identifying eight architectural patterns that map directly onto existing KoadOS concepts (CASS memory tiers, Body/Ghost model, pipeline pattern, agent taxonomy). This feature request captures the what and why — implementation specs, templates, and code are deferred to codebase agents (Tyr, Sky) with direct access.
---
## Source Material
- Analyzed artifact: Claude Code's skill-creator skill (full instruction set captured in Claude Code Skill Creator)
- Analysis session: Noti × Ian, 2026-03-14
- Related prior art: Agent Boot Research — CLI Context Injection Patterns, Context Hydration Architecture research
---
## Problem Statement
KoadOS agents currently lack a standardized mechanism for:
1. Packaging reusable behaviors — workflows, domain expertise, and operational patterns are embedded in agent instructions or scattered across docs. There's no portable, composable unit.
1. Dynamic capability loading — agents load their full identity at boot but can't selectively acquire new capabilities mid-session based on task context.
1. Skill discovery and triggering — no semantic routing layer exists to match user intent against available agent capabilities and load only what's relevant.
1. Iterative skill improvement — no eval/feedback loop to validate that a packaged behavior actually works well across diverse inputs.
The Claude Code Skill Creator solves all four problems within its ecosystem. KoadOS should solve them within ours — but natively, using our architecture.
---
## Extracted Patterns — Research Summary
The following patterns were identified from the Claude Code skill system. Each includes a mapping to KoadOS architecture.
### Pattern 1: Progressive Disclosure (Three-Tier Context Loading)
What Claude does: Skills load in three tiers — metadata (~100 words, always in context), SKILL.md body (<500 lines, on trigger), bundled resources (unlimited, on demand).
KoadOS mapping: Maps directly to CASS memory tiers. Skill metadata → L1 Redis (hot, always available). Skill body → L2 SQLite (warm, loaded on activation). Deep references → L3 Qdrant or filesystem (cold, loaded on explicit need). The context hydration pipeline already has a three-tier model — skills should plug into it.
Key insight: Context window is a scarce resource. Every skill should have a context cost profile declared upfront.
### Pattern 2: Phased Convergence Loops
What Claude does: Skills aren't linear procedures — they're iterative loops with named phases and re-entry logic. The agent orients to wherever the user is in the process and jumps in.
KoadOS mapping: This is the pipeline pattern from the Agent Taxonomy. Skills should define phases with entry conditions, not just sequential steps. An agent should be able to resume a skill mid-workflow after a session break (via EndOfWatch + CASS session restore).
### Pattern 3: Description as Semantic Router
What Claude does: The skill description field is the sole trigger mechanism. The system matches user intent against description text to decide whether to load the skill body. Descriptions are written slightly "pushy" to avoid under-triggering.
KoadOS mapping: This is a routing layer problem. KoadOS would need a skill registry (likely a CASS-managed index) where each skill has an intent signature. On user input, the routing layer scores available skills against the input and loads the highest-match skill's body into context. Could leverage Qdrant semantic search for fuzzy matching.
### Pattern 4: Environment Adaptation / Graceful Degradation
What Claude does: Skills declare capability dependencies and provide fallback behavior per runtime context (full subagents vs. no subagents vs. no browser).
KoadOS mapping: Maps to the Body/Ghost model and Citadel disconnect states. A skill should declare what it needs (requires: [cass, redis, mcp_tools]) and define degraded behavior when dependencies are unavailable. If CASS is offline, the skill operates from local ghost config only. If subagents aren't available, the skill falls back to inline execution.
### Pattern 5: Subagent Delegation via Role References
What Claude does: Complex skills decompose into role-specific sub-skills (agents/grader.md, agents/comparator.md) loaded lazily when the workflow needs them.
KoadOS mapping: This is the pipeline pattern + Personal Bay model. A skill can reference other skills or agent roles by name. The orchestrating agent holds the workflow graph; delegates load their own skill context on activation. Cross-agent skill invocation could route through A2A-S signals.
### Pattern 6: Eval-Driven Development
What Claude does: Skills ship with test cases, assertion frameworks, grading agents, and human-in-the-loop review. Eval is baked into the skill lifecycle, not bolted on.
KoadOS mapping: This is new territory for KoadOS. A skill's _eval/ directory would contain test prompts and expected behaviors. The Integrity Audit Protocol could be extended to cover skill validation. KSRP could include a "skill review" pass.
Self-improvement binding: Eval is not just a ship gate — it is the mechanism by which skills improve. Each eval run produces a graded result. Failed scenarios generate targeted improvement tasks. The skill's instruction body is revised, re-tested, and versioned. The cycle is: Author → Eval → Grade → Gap Analysis → Revise → Re-eval → Promote. Skills that cannot pass their own eval at ≥ 80% are not deployed. Skills that regress below 80% after an update are rolled back automatically.
### Pattern 7: Anti-Overfitting / Mentorship Tone
What Claude does: Instructions explain why before what. Explicitly warns against rigid ALWAYS/NEVER patterns. Treats the consuming agent as a smart collaborator, not a mechanical executor.
KoadOS mapping: This is a canon-level principle. Skills should be written for class-level coverage, not instance-level scripting. Aligns with the Contributor & Coding Manifesto's philosophy. Should be codified in a "Skill Writing Guide" that ships with the system.
### Pattern 8: Bundled Reusable Scripts
What Claude does: When test runs reveal that agents independently write the same helper scripts, those scripts get bundled into the skill's scripts/ directory. DRY principle applied to agent-generated code.
KoadOS mapping: Skills can bundle executable scripts (bash, Python, Rust) in a scripts/ directory. These run without being loaded into context — agents invoke them by path. Aligns with KoadOS's programmatic-first communication principle.
---
## Proposed Skill Anatomy (Conceptual)
This is a starting point for Tyr to evaluate, not a final spec.
```plain text
skill-name/
├── SKILL.md              # Required. Frontmatter (name, description, requires) + instructions
├── scripts/              # Optional. Executable code for deterministic tasks
├── references/           # Optional. Deep-load docs (ToC if >300 lines)
├── agents/               # Optional. Role-specific sub-skill instructions
└── _eval/                # Required for all non-trivial skills
    ├── test-prompts.md       # ≥ 12 test scenarios grouped by operation type
    ├── grading-schema.md     # Weighted criteria + pass thresholds (target: ≥ 80%)
    ├── results/              # Versioned eval run outputs (auto-generated)
    │   └── YYYY-MM-DD.json   # Graded result per eval run
    └── improvement-log.md    # Gap analysis notes + revision history
```
Frontmatter fields (candidate):
```yaml
name: skill-name
description: "Intent signature — when to trigger, what it does"
requires: [cass, redis]           # Capability dependencies
tier: officer | crew | micro      # Which agent tier can use this
context_cost: small | medium | large
version: 1.0.0                    # Semantic version — incremented on each eval-driven revision
eval_pass_threshold: 0.80         # Minimum weighted score required to deploy or retain
author: tyr                       # Agent responsible for implementation + maintenance
```
---
## Integration Points with Existing KoadOS Architecture
---
---
## Blueprint / Instance Model — Skill Ownership
### Why Blueprint/Instance?
A single shared skill definition creates problems at scale:
- Improvement conflicts. If Sky and Tyr both improve the same skill, whose changes win? Merging AI-generated instruction refinements is not like merging code — there's no clean diff.
- Context drift. Sky uses notion-sync for SLE work. Tyr uses it for Citadel docs. Over time, their usage patterns diverge. A single shared skill would overfit to whichever agent used it last.
- Specialization is the goal. KoadOS agents are specialists. The same skill should evolve differently in different hands — Sky's version of code-review should reflect SLE conventions; Vigil's version should reflect security audit patterns.
The Blueprint/Instance model solves all three by making skill ownership explicit and per-agent.
### How It Works
```plain text
~/.koad-os/skills/                    # BLUEPRINTS — system-level canonical definitions
  notion-sync/
    SKILL.md                          # Blueprint v1.0.0
    _eval/
    scripts/

~/.koad-os/config/identities/
  sky/
    skills/                           # SKY'S INSTANCES — her personal copies
      notion-sync/
        SKILL.md                      # Instance v1.2.1 (forked from blueprint v1.0.0)
        _eval/                        # Sky's eval results, not the blueprint's
        improvement-log.md            # Sky's improvement history
  tyr/
    skills/                           # TYR'S INSTANCES
      notion-sync/
        SKILL.md                      # Instance v1.0.3 (forked from blueprint v1.0.0)
        _eval/
        improvement-log.md
```
### Blueprint Lifecycle
1. Authored — A skill blueprint is created in ~/.koad-os/skills/ by an agent or by Ian. It defines the canonical starting point: instructions, eval suite, scripts, dependencies.
1. Published — The blueprint is registered in the system skill catalog (a CASS-managed index). It becomes available for granting.
1. Granted — When an agent acquires the skill (via any acquisition channel: innate, rank, role, at-will), the system forks the blueprint into the agent's personal skills/ directory. The agent now owns an instance.
1. Versioned independently — The blueprint may receive updates (new eval scenarios, bug fixes). Agent instances are NOT auto-updated. The agent's instance evolves on its own trajectory.
1. Upstream sync (optional) — An agent can choose to pull updates from the blueprint (koad skill sync <skill-name> --from blueprint), but this is opt-in. It merges new eval scenarios and script updates without overwriting the agent's customized instructions.
### Instance Lifecycle
1. Forked — Created from a blueprint at grant time. The instance's SKILL.md frontmatter records blueprint_version (the version it was forked from) and instance_version (the agent's own version counter).
1. Used — The agent invokes the skill during sessions. Usage is logged.
1. Evaluated — Eval runs use the instance's _eval/ suite, not the blueprint's. The agent's eval results reflect their specific usage patterns.
1. Improved — The self-improvement loop (Author → Eval → Grade → Gap Analysis → Revise → Re-eval → Promote) operates on the instance. The agent proposes refinements to their own copy.
1. Diverged — Over time, the instance may diverge significantly from the blueprint. This is expected and healthy — it means the agent has specialized the skill to their domain.
### Frontmatter Changes
The SKILL.md frontmatter gains fields to track the blueprint/instance relationship:
```yaml
# Blueprint (in ~/.koad-os/skills/notion-sync/SKILL.md)
name: notion-sync
type: blueprint
version: 1.0.0
author: tyr
description: "Bidirectional sync between Notion pages and local SQLite databases"

# Instance (in ~/.koad-os/config/identities/sky/skills/notion-sync/SKILL.md)
name: notion-sync
type: instance
blueprint_version: 1.0.0           # Version of blueprint this was forked from
instance_version: 1.2.1             # Agent's own version counter
owner: sky                           # Agent who owns this instance
forked_at: 2026-03-20
last_blueprint_sync: 2026-03-20     # Last time upstream changes were pulled
max_tokens: 800                      # Token budget constraint for this skill
```
### Cross-Agent Skill Sharing (Instance-to-Instance)
The Blueprint/Instance model also clarifies cross-agent sharing:
- Blueprint sharing is always allowed — blueprints are system-level and available to all agents (subject to tier restrictions).
- Instance sharing is controlled — if Sky wants to share her improved notion-sync with Tyr, she can publish her instance as a new blueprint variant (koad skill publish <skill-name> --as <variant-name>). Tyr can then fork from Sky's variant instead of the original blueprint.
- No direct instance access — agents cannot read or write each other's instances. This preserves the memory isolation model from CASS (same principle as private Qdrant collections).
### Promoting Instance Improvements Back to Blueprint
When an agent's instance improvements are broadly valuable, the improvement can be promoted back to the system blueprint:
1. Agent (or operator) runs koad skill promote <skill-name> — proposes the instance's changes as a blueprint update.
1. The diff between the instance and the current blueprint is generated.
1. Ian reviews and approves (per Development Canon — no auto-merge to blueprints).
1. Blueprint version is bumped. Other agents can opt-in to sync.
This creates a bottom-up improvement flow: agents improve their own copies → best improvements bubble up to the blueprint → other agents benefit on their next sync. The system gets smarter through agent specialization, not through centralized updates.
### Relationship to CASS Memory
- Blueprints are stored on the filesystem (~/.koad-os/skills/) and indexed in CASS L4 (procedural memory). They are system knowledge.
- Instances are stored in the agent's identity directory and indexed in the agent's private CASS L4 partition. They are agent-specific procedural memory.
- Instance eval data is stored in the agent's CASS L2 (episodic memory) — it's usage history, not permanent knowledge.
- Usage tracking (invocation logs, outcome records) goes to CASS L2 and feeds the self-improvement eval loop.
---
## Self-Improvement Safeguards
