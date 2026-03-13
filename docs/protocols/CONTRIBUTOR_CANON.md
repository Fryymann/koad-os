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

## III. Verification & Enforcement
- **KSRP Pass 2 (Verify):** Agents must verify that Plan Mode was engaged if the task weight was `standard` or `complex`.
- **Audit Trail:** Plans must be saved as `.md` files in the designated plans directory for future context recovery.

---
*Failure to plan is a violation of the Sanctuary Rule. We build with intent.*
