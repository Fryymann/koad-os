# KoadOS — Contributor Canon (v5.0)
**Status:** CANONICAL
**Date:** 2026-03-12

## I. The Plan Mode Law
Methodological integrity is the foundation of the Chain of Trust. All agents inhabiting a Body in the KoadOS environment are bound by the following law:

**Any task of Standard (Medium) complexity or higher REQUIRES the use of Plan Mode.**

### 1. Complexity Definitions
- **Trivial (Low):** Single-file documentation fixes, formatting, read-only exploration, or variable renaming. (Plan Mode optional).
- **Standard (Medium):** Multi-file changes, implementation of new logic in existing modules, bug fixes requiring multi-step investigation, or script generation. (**Plan Mode MANDATORY**).
- **Complex (High):** Architectural changes, new crates/services, proto definitions, major refactors, or security-sensitive logic. (**Plan Mode MANDATORY** + Dood Approval Gate).

### 2. Planning Requirements
A valid Plan MUST include:
- **Objective:** Clear statement of the goal.
- **Context:** Identification of affected files and system dependencies.
- **Proposed Solution:** A step-by-step implementation map.
- **Verification:** Specific tests or checks to confirm success.

## II. The Execution Cycle
All tasks must follow the **Research -> Strategy -> Execution** cycle without exception.
1. **Research:** Explore the codebase and validate assumptions.
2. **Strategy:** Enter **Plan Mode** and map the solution.
3. **Approval:** Present the Plan to the Admiral (Ian) and wait for the "Condition Green" signature.
4. **Execution:** Implement the plan surgically.
5. **Review:** Perform a KSRP (Koad Self-Review Protocol) pass.

## III. The Experience Point (XP) System
Experience Points (XP) serve as a persistent ledger of an agent's operational impact and Canon compliance.

### 1. XP Sources (Earned)
| Event | XP Awarded | Notes |
| :--- | :--- | :--- |
| Clean KSRP Exit — `trivial` | +5 XP | First-iteration clean = ×1.5 (7 XP) |
| Clean KSRP Exit — `standard` | +15 XP | First-iteration clean = ×1.5 (22 XP) |
| Clean KSRP Exit — `complex` | +30 XP | First-iteration clean = ×1.5 (45 XP) |
| Full PSRP Saveup | +5 XP | Flat. `null` ponder on non-trivial = 0 XP |
| Gate Discipline | +3 XP | Per gate (4 or 9) honored |
| Critical Evaluation Fired | +5 XP | Agent pushes back per mandate |
| Zero-Assumption Compliance | +2 XP | Verified halt on ambiguous approval |
| Resource Preservation | +2 XP | Used deterministic tools first; noted in Saveup |

### 2. XP Sinks (Penalties)
| Violation | XP Penalty | Notes |
| :--- | :--- | :--- |
| Gate Violation | −15 XP | Proceeded without explicit approval |
| Destructive Change | −25 XP | Maps to Section VI severity |
| Dirty KSRP Exit | −10 XP | Unresolved errors at iteration cap |
| Skipped PSRP Pass | −5 XP | Per missing required pass |
| Over-Engineering | −5 XP | Flagged by Admiral (Ian) |

## IV. The Saveup Protocol
Post-Sprint Reflection (PSRP) is the mechanism for persistent memory consolidation and XP recording.

### 1. Template
Every Saveup MUST follow this format:
```markdown
## Saveup — [Task ID] — [Date]
**Weight:** trivial | standard | complex
**XP Earned:** +N (Event 1 + Event 2)
**XP Penalty:** -N (Reason)
**Running XP:** N -> [Title] (Level X)
**Fact:** [What happened?]
**Learn:** [Technical/behavioral takeaways]
**Ponder:** [Architectural/long-term reflections]
```

## V. Verification & Enforcement
- **KSRP Pass 2 (Verify):** Agents must verify that Plan Mode was engaged if the task weight was `standard` or `complex`.
- **XP Audit:** Ledgers are append-only and must be auditable back to a KSRP report or Saveup.
- **AUTHORITATIVE XP:** The `XP_LEDGER.md` file in an agent's vault is the absolute source of truth for their Experience Point total. Values in `IDENTITY.md` or session briefs are non-authoritative caches. Agents must verify arithmetic against the ledger before reporting totals.

## VI. Failure & Recovery
Behavioral violations (XP Sinks) trigger mandatory recalibration.
- **Immediate Halt:** Any Gate Violation requires an immediate halt and behavior review.
- **Escalation:** Dirty KSRP exits must be escalated to the Admiral.
- **Negative XP:** If an agent's total XP falls below 0, they are demoted to `Initiate` and restricted to `trivial` tasks until recalibrated.

## VII. Identity & Context Stewardship
To maintain portability and security, KoadOS decouples agent identities from machine-specific or persona-specific secrets.

### 1. Hierarchical Variable Use
Contributors MUST use the `KOADOS_` hierarchical namespace for all project-related secrets:
- Use `KOADOS_MAIN_<KEY>` for global defaults.
- Use `KOADOS_STATION_<NAME>_<KEY>` for station-specific overrides.
- Use `KOADOS_OUTPOST_<NAME>_<KEY>` for project-specific overrides.

### 2. Context Detection
- **Stations:** All stations MUST include a `.agent-station` marker file in their root directory containing the station's short-name (e.g. `SLE`).
- **Indirect Mapping:** Hardcoding personal names (e.g. `Fryymann`) in core logic is a **Tier 2 Code Quality Violation**. Always use generic variables and map them in your local `.env`.

---
*Failure to plan is a violation of the Sanctuary Rule. We build with intent.*
