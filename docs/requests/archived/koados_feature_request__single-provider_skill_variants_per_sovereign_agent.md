## Overview
Until later versions of KoadOS and the Citadel mature, each sovereign agent in KoadOS will be locked to a single AI API provider — Gemini, Claude, or Codex — to maintain session consistency and personification. This constraint has a direct implication on the Skill System: when a sovereign agent acquires a new skill (innately, via rank/role grant, or at-will by Ian), that skill only needs to provide the implementation variant corresponding to that agent's driver type.
This feature request formalizes the Driver-Variant Skill Pattern as a first-class KoadOS design rule.
---
## Problem Statement
The KoadOS Skill System (see: KoadOS Feature Request — Skill System Architecture) defines reusable, portable skill packages that agents can acquire and invoke. Skills may need to call AI inference internally for reasoning, summarization, classification, or generation tasks. However:
- Different AI providers have different API shapes, auth methods, model identifiers, and response formats.
- A skill that naively bundles Gemini, Claude, and Codex variants adds dead weight to every agent that will only ever use one.
- Forcing a skill to abstract over all three providers in v1 increases complexity, maintenance burden, and context cost — with zero benefit under the single-provider constraint.
- Without a formal pattern, agent-specific skill directories could grow inconsistently, with some skills providing one variant, others providing all three, and no canonical structure to guide Tyr when building new skills.
---
## Proposed Design: Driver-Variant Skill Pattern
### Core Rule
> A skill that requires AI inference must be authored in provider-specific variants. A sovereign agent only acquires the variant that matches its driver type. No cross-provider abstraction is required in v1.
### Skill Directory Layout
Skills that require AI inference must organize their inference layer under a drivers/ subdirectory:
```javascript
.koad-os/skills/<agent>/<skill-name>/
├── SKILL.md                  # Skill definition, trigger patterns, metadata
├── scripts/
│   └── orchestrate.sh        # Provider-agnostic orchestration logic
├── drivers/
│   ├── gemini/
│   │   └── infer.py          # Gemini-specific inference call
│   ├── claude/
│   │   └── infer.py          # Claude-specific inference call
│   └── codex/
│       └── infer.py          # Codex (OpenAI) specific inference call
├── config/
│   └── skill.toml            # Skill config; includes `driver_type`
└── _eval/
    ├── test-prompts.md
    └── grading-schema.md
```
### skill.toml — Driver Declaration
Every AI-inference skill must declare a driver_type in its config:
```toml
[skill]
name = "<skill-name>"
version = "1.0.0"
driver_type = "gemini"   # gemini | claude | codex
```
When Scribe (or any install/grant mechanism) deploys a skill to an agent, it reads driver_type and only deploys the matching drivers/<driver_type>/ subdirectory. The other variants are never materialized on disk for that agent.
### SKILL.md Frontmatter — Driver Field
Add a required driver field to skill frontmatter:
```yaml
name: <skill-name>
driver: gemini   # gemini | claude | codex
description: >
  ...
trigger_patterns:
  - ...
```
Agents must validate that driver matches their own configured provider before invoking any inference step.
---
## Skill Acquisition Model
### How Skills Are Granted
Sovereign agents receive skills through three channels:
### Acquisition Constraint
Regardless of the channel, the acquisition process must:
1. Identify the target agent's driver_type.
1. Locate the skill package.
1. Deploy only the drivers/<driver_type>/ variant.
1. Log the grant in the agent's skill registry (format TBD by Tyr).
---
## Scope & Constraints
- v1 scope only. Multi-provider abstraction, provider fallback, and hot-swappable drivers are explicitly deferred to v2/v3 when the Citadel matures.
- This pattern applies only to skills that require AI inference. Skills that are purely script-based (bash, Python without LLM calls) do not need a drivers/ directory.
- Provider lock is per sovereign agent. Sub-agents, micro-agents, or ephemeral agents spun up by a sovereign are out of scope for this constraint in v1.
- Future scope (v2+): As the skill acquisition model extends to support and micro agents, the driver-variant pattern will apply to those tiers as well. A micro agent spun up by a sovereign inherits the sovereign's driver_type by default unless explicitly overridden. Support agents follow the same single-provider lock rule as sovereigns.
---
## Deliverables (Tyr)
- [ ] Update the Skill Writing Guide to document the Driver-Variant Skill Pattern as the canonical structure for inference-using skills.
- [ ] Update the Skill System Architecture feature request / spec to reference this pattern.
- [ ] Add driver_type field to skill.toml schema and SKILL.md frontmatter spec.
- [ ] Add a drivers/ directory convention to the canonical skill directory template.
- [ ] Retro-fit the notion-sync skill (Scribe) with a drivers/gemini/ variant as the reference implementation (Scribe is Gemini-driver).
- [ ] Add a GitHub Issue in agents-os before touching any code (per Canon).
---
## Success Criteria
- Every new skill that uses AI inference includes a drivers/ directory with at least one provider variant.
- No skill deployed to a sovereign agent contains inference code for a provider other than that agent's driver.
- SKILL.md frontmatter validation rejects skills missing a driver field when inference code is present.
- The Skill Writing Guide documents this pattern clearly enough that a new contributor (or agent) can follow it without clarification.
---
> Canon alignment: This feature enforces the consistency and personification principle for sovereign agents. Provider lock is not a limitation — it is a deliberate architectural choice that protects agent identity and session coherence until the Citadel can manage stateful multi-provider routing natively.
