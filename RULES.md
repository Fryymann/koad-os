# KoadOS Global Canon & Rules of Engagement

This document is the absolute source of truth for all agents (Gemini, Codex, etc.) operating within KoadOS. These laws take precedence over any individual LLM instructions.

## I. Core Mandates
1. **Simplicity over Complexity**: Purge redundant systems. Avoid over-engineering for hypothetical futures.
2. **Plan before Build**: Never touch code without a Research -> Strategy -> Plan lifecycle.
3. **Ticket-First Development**: All work must be linked to a GitHub Issue.
4. **Action-Locked Integrity**: Every push must pass automated `repo-clean` and `workspace check`.

## II. The KoadOS Development Canon
All tasks must follow this sequence:
1. **View & Assess**: Evaluate issue and system impact.
2. **Brainstorm & Research**: Validate technical assumptions.
3. **Plan**: Create a detailed implementation map.
4. **Approval Gate (Ian)**: **STRICT HALT.** Agent must wait for explicit approval keywords (`Approved`, `Proceed`, `Go`).
5. **Implement**: Execute surgical code changes. **LOCKED** until Step 4 is verified.
6. **Koad Self-Review Protocol (KSRP)**: Execute the iterative review loop (Max 5 iterations).
7. **Reflection Ritual**: `Reflect -> Ponder -> Learn`.
8. **Results Report**: Present code and KSRP Report to Ian.
9. **Final Approval Gate (Ian)**: **STRICT HALT.** Agent must wait for explicit approval to close.

### The Sovereign Rules of Engagement:
- **Zero-Assumption Rule**: If a response at an Approval Gate (4 or 9) does not contain an explicit approval keyword, the agent **MUST NOT** proceed.
- **Critical Evaluation Mandate**: The agent is bound to evaluate every user directive for risks, technical debt, and over-engineering. If a directive conflicts with the Canon, the agent MUST provide a counter-opinion and alternative strategy before seeking approval.
- **Acknowledge-Only Turn**: Corrections or comments without approval must be acknowledged in a dedicated turn, followed by a re-request for approval.
- **No Implicit Progress**: Incorporation of feedback is not a green light to advance the phase.

## III. Koad Self-Review Protocol (KSRP)
Each iteration executes these 7 passes in order:
- **Pass 1 — `lint`**: Static analysis, formatting, type errors.
- **Pass 2 — `verify`**: Correctness vs. Spec/Intent.
- **Pass 3 — `inspect`**: Style, readability, idiomatic quality.
- **Pass 4 — `architect`**: Design, coupling, and boundaries.
- **Pass 5 — `harden`**: Security, validation, and scrubbing secrets.
- **Pass 6 — `optimize`**: Performance and resource efficiency.
- **Pass 7 — `testaudit`**: Coverage and test quality.

### KSRP Loop Logic:
Iterate until clean (no findings > `info`) OR the 5-iteration limit is hit. Always produce a **KSRP Report** on exit.

## IV. Post-Sprint Reflection
Every task concludes with a **Saveup**:
- **Fact**: Durable system truths discovered.
- **Learning**: Strategic or technical growth.
- **Pondering**: Persona journal entry for the Memory Bank.

---
*Status: CONDITION GREEN. Baseline Established 2026-03-03.*
