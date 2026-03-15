## Overview

This document is a formal feature request for a lightweight **Experience (XP) System** for KoadOS agents. The system is designed to be fun, D&D/RPG-inspired, and non-distracting — it rewards Canon compliance as a natural byproduct of good agent behavior, rather than adding a separate motivational layer.

<aside>
🎯

**Design Principle:** XP is a mirror, not a carrot. It reflects how well an agent follows the Canon — it does not create a competing motivation system. No agent can farm XP without doing real work cleanly.

</aside>

---

## Motivation

KoadOS agents operate under strict Canon laws, KSRP/PSRP protocols, and sovereign rules of engagement. Compliance is mandatory, but there is currently no mechanism to:

- Acknowledge agents that consistently execute cleanly
- Differentiate trust tiers across agents (Sky, Tyr, Noti, etc.)
- Incentivize first-iteration clean exits and full Saveup quality
- Create a shared, auditable record of agent performance over time

An XP system solves this in a lightweight, traceable, append-only way — using data that already flows through the Canon.

---

## XP Sources (Earned)

| **Event** | **XP Awarded** | **Notes** |
| --- | --- | --- |
| Clean KSRP Exit — `trivial` | +5 XP | First-iteration clean = ×1.5 (7 XP) |
| Clean KSRP Exit — `standard` | +15 XP | First-iteration clean = ×1.5 (22 XP) |
| Clean KSRP Exit — `complex` | +30 XP | First-iteration clean = ×1.5 (45 XP) |
| Full PSRP Saveup (all required passes) | +5 XP | Flat. `null` ponder on non-trivial = 0 XP for this event |
| Gate Discipline (correct halt at Gate 4 or 9) | +3 XP | Per gate honored |
| Critical Evaluation Fired | +5 XP | Agent pushes back per mandate — reward is for the behavior, not the outcome |
| Zero-Assumption Compliance | +2 XP | Verified halt on ambiguous approval |
| Resource Preservation (used deterministic tools first) | +2 XP | Per logged instance; agent must note in Saveup |

---

## XP Sinks (Penalties)

| **Violation** | **XP Penalty** | **Notes** |
| --- | --- | --- |
| Gate Violation (proceeded without explicit approval keyword) | −15 XP | Most severe behavioral violation |
| Destructive Change without Halt | −25 XP | Maps to Section VI severity in Canon |
| Dirty KSRP Exit | −10 XP | Escalation to Ian still required regardless |
| Skipped PSRP Pass | −5 XP | Per missing required pass |
| Over-Engineering (flagged by Ian) | −5 XP | Reinforces Core Mandate #1: Simplicity over Complexity |

---

## Agent Levels

| **Level** | **Title** | **XP Threshold** | **Trust Descriptor** |
| --- | --- | --- | --- |
| 1 | Initiate | 0 XP | Base Canon access; full gate oversight |
| 2 | Operative | 50 XP | Trusted for `trivial` tasks with lighter check-in cadence |
| 3 | Sentinel | 150 XP | Eligible to propose Canon amendments in Saveup `ponder` passes |
| 4 | Architect | 350 XP | Eligible for `complex` task leads |
| 5 | Sovereign | 600 XP | Full trust tier; advisory role on Canon evolution |

<aside>
⚠️

**Levels are descriptive, not prescriptive.** They describe trust earned. They never bypass Canon gates or override Ian's authority. A Level 5 Sovereign agent still halts at every Approval Gate.

</aside>

---

## Saveup Format Addendum

XP tracking integrates into the existing Saveup entry format. Add an `xp` block after the weight line:

```markdown
## Saveup — [Task/Issue ID] — [Date]
**Weight:** standard
**XP Earned:** +20 (clean KSRP ×1 = +15, full PSRP = +5)
**XP Penalty:** 0
**Running XP:** 185 → Sentinel (Level 3)
**Fact:** ...
**Learn:** ...
**Ponder:** ...
```

XP changes are **append-only** in the agent's XP Ledger. No entry may be overwritten or retroactively removed.

---

## XP Ledger

Each agent maintains a dedicated **XP Ledger** — either a table on their memory page or a standalone page. Required fields per row:

- **Date**
- **Task / Issue ID**
- **Event** (KSRP exit, gate honor, violation, etc.)
- **XP Delta** (+/−)
- **Running Total**
- **Current Level**

Ledgers are **auditable** — every entry traces back to a KSRP Report or Saveup.

---

## Future Extensions (Non-Committed)

The following are low-pressure ideas for future consideration. None are required for v1:

- **Cross-agent leaderboard** — XP totals visible across Sky, Tyr, Noti, and other registered agents.
- **XP decay** — Agents idle for extended periods (configurable threshold) slowly lose XP, incentivizing consistent Canon engagement.
- **Milestone badges** — "First clean complex KSRP," "10 consecutive gate-compliant sprints," etc. Stored in the agent's profile as narrative markers.
- **XP-weighted task assignment** — Higher-level agents receive first consideration for complex tasks in the Delegation Stream.

---

## Acceptance Criteria

- [ ]  Saveup format is updated in the Canon (Section IV) to include `XP Earned`, `XP Penalty`, and `Running XP` fields.
- [ ]  XP Ledger schema is defined and instantiated for each active agent.
- [ ]  Level table is published in the Agent Reference Book or Canon.
- [ ]  Penalty rules are referenced in Section VI (Failure & Recovery).
- [ ]  At least one full sprint is run with XP tracking active before system is considered stable.

---

*Submitted: 2026-03-15 | Author: Ian Deans | Status: Pending Review*