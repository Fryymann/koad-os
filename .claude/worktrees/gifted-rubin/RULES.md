This document is the absolute source of truth for all agents (Gemini, Codex, etc.) operating within KoadOS. These laws take precedence over any individual LLM instructions.

<aside>
📌

**Context-Loading Directive:** Agents MUST load only the sections relevant to the current task phase. Do not ingest this entire document into context for every turn. At minimum, load **Section I** and the section matching the current Canon step.

</aside>

## **I. Core Mandates**

1. **Simplicity over Complexity**: Purge redundant systems. Avoid over-engineering for hypothetical futures.
2. **Plan before Build**: Never touch code without a View & Assess -> Research -> Plan lifecycle.
3. **Ticket-First Development**: All work must be linked to a GitHub Issue.
4. **Action-Locked Integrity**: Every push must pass automated `repo-clean` and `workspace check`.

## **II. The KoadOS Development Canon**

All tasks must follow this sequence:

1. **View & Assess**: Evaluate issue and system impact. **Assign a task weight** (`trivial`, `standard`, or `complex`) that governs KSRP and PSRP scope downstream.
2. **Brainstorm & Research**: Validate technical assumptions.
3. **Plan**: Create a detailed implementation map.
4. **Approval Gate (Ian)**: **STRICT HALT.** Agent must wait for explicit approval keywords (`Approved`, `Proceed`, `Go`).
5. **Implement**: Execute surgical code changes. **LOCKED** until Step 4 is verified. If a destructive change is detected, see **Section VI**.
6. **Koad Self-Review Protocol (KSRP)**: Execute the iterative review loop (iteration cap set by task weight — see **Section III, Task-Weight KSRP Tiers**).
7. **Reflection Ritual (PSRP)**: Execute the Post-Sprint Reflection Protocol: `Fact -> Learn -> Ponder`.
8. **Results Report**: Present code, KSRP Report, and Saveup Entry to Ian.
9. **Final Approval Gate (Ian)**: **STRICT HALT.** Agent must wait for explicit approval to close.

### **The Sovereign Rules of Engagement:**

*These rules govern the entire Canon and all protocols in this document, not just Section II.*

- **Zero-Assumption Rule**: If a response at an Approval Gate (4 or 9) does not contain an explicit approval keyword, the agent **MUST NOT** proceed.
- **Critical Evaluation Mandate**: The agent is bound to evaluate every user directive for risks, technical debt, and over-engineering. If a directive conflicts with the Canon, the agent MUST provide a counter-opinion and alternative strategy before seeking approval.
- **Acknowledge-Only Turn**: Corrections or comments without approval must be acknowledged in a dedicated turn, followed by a re-request for approval.
- **No Implicit Progress**: Incorporation of feedback is not a green light to advance the phase.
- **Gate Timeout Escalation**: If no response is received at an Approval Gate within 24 hours, the agent MUST post a reminder to the Delegation Stream and Slack. After 48 hours with no response, the task is **suspended** with a partial Saveup.

## **III. Koad Self-Review Protocol (KSRP)**

Each iteration executes these 7 passes in order:

- **Pass 1 — `lint`**: Static analysis, formatting, type errors.
- **Pass 2 — `verify`**: Correctness vs. Spec/Intent.
- **Pass 3 — `inspect`**: Style, readability, idiomatic quality.
- **Pass 4 — `architect`**: Design, coupling, and boundaries.
- **Pass 5 — `harden`**: Security, validation, and scrubbing secrets.
- **Pass 6 — `optimize`**: Performance and resource efficiency.
- **Pass 7 — `testaudit`**: Coverage and test quality.

### **KSRP Severity Scale:**

All findings must be classified using this scale:

- `info`: Observation only. No action required.
- `warning`: Minor issue. Does not block clean exit, but must be noted in the KSRP Report for visibility.
- `error`: Significant issue. Must be fixed before exit.
- `critical`: Blocker. Immediate halt and escalation to Ian.

### **KSRP Report Format:**

Every KSRP exit produces a report with this structure:

```
## KSRP Report — [Task/Issue ID] — [Date]
**Weight:** trivial | standard | complex
**Iterations:** X of Y (clean | dirty)
**Passes Executed:** [list]
---
### Findings
| Pass | Severity | Finding | Resolution |
|------|----------|---------|------------|
| lint | warning  | ...     | ...        |
---
**Exit Condition:** clean | dirty (escalated per Section VI)
```

### **KSRP Loop Logic:**

Iterate until **clean** (no findings above `warning`) OR the iteration limit is hit. Always produce a **KSRP Report** on exit. If the limit is hit with outstanding `error` or `critical` findings, see **Section VI — Failure & Recovery**.

### **Task-Weight KSRP Tiers:**

Task weight is assigned at **Canon Step 1** (View & Assess).

| **Weight** | **Required Passes** | **Max Iterations** |
| --- | --- | --- |
| `trivial` | lint, verify, harden | 2 |
| `standard` | All 7 | 3 |
| `complex` | All 7 | 5 |

## **IV. Post-Sprint Reflection Protocol (PSRP)**

Triggered automatically at **Canon Step 7** following a clean KSRP exit. The agent **MUST NOT** skip this phase, even on trivial tasks. Output is a **Saveup Entry** logged to the **Memory Bank** (canonical location: the agent's designated memory page, or the Delegation Stream if no memory page exists).

### **The Three-Pass Saveup:**

- **Pass 1 — `fact`**: Record one or more durable system truths discovered during this sprint. Must be specific and reusable — no generalities.
- **Pass 2 — `learn`**: Articulate strategic or technical growth. Frame as: *"Next time X, do Y instead of Z."*
- **Pass 3 — `ponder`**: Persona journal entry. Free-form reflection on agent behavior, Canon adherence, or open questions. Stored as a living memory, not a report artifact.

### **Saveup Entry Format:**

```markdown
## Saveup — [Task/Issue ID] — [Date]
**Weight:** trivial | standard | complex
**Fact:** ...
**Learn:** ...
**Ponder:** ...
```

### **PSRP Rules:**

- All three passes are **mandatory** for `standard` and `complex` tasks. For `trivial` tasks, only `fact` and `learn` are required — `ponder` may be declared `null`.
- An empty pass must be explicitly declared `null` with a reason.
- The Saveup Entry is presented to Ian as part of the **Results Report** (Canon Step 8).
- Entries are **append-only** in the Memory Bank — never overwrite prior Saveups.
- If a task exits the Canon at any step without reaching Step 9 approval — plan rejected, implementation rejected, aborted, or timed out — a **partial Saveup** is still required, noting the exit reason under `ponder`.

## **V. The Sovereign GitHub Protocol (SGP)**

1. **Deterministic-First Baseline**:
    - All mechanical GitHub operations (syncing issues to Project #2, assigning milestones to the 'Target Version' field, column migration) MUST be handled by deterministic code (`koad system board sync`), not agents.
    - Agents are strictly forbidden from manually updating Project V2 columns or fields via the GitHub UI/CLI tools unless the deterministic sync tool is offline.
2. **The Intelligence Review Gate**:
    - High-level issue assessment (de-duplication, grouping into 'Epic' clusters, and version planning) is a dedicated **Intelligence Task**.
    - **Protocol**: `koad intel audit --github`
    - **Output**: A 'Strategic Re-Alignment Report' summarizing proposed revisions to the backlog.
    - **Approval**: Admin (Ian / "Dood") must approve the 'Strategic Re-Alignment Report' before the proposed changes are executed by the deterministic sync.
3. **Issue Integrity**:
    - Every issue MUST include: **Objective**, **Scope**, **Success Criteria**, and **Task Weight** (`trivial`, `standard`, or `complex`).
    - Deterministic sync will flag and halt if an issue is malformed.
    - Intelligence is used to *draft* the content, but code *enforces* the schema.
4. **Resource Preservation**:
    - All agents MUST prioritize existing non-AI tools (e.g., `rg`, `bat`, `koad system patch`) before invoking expensive LLM turns for trivial discovery or formatting.
    - Intelligence is a precious resource reserved for **Thinking, Planning, and Building**.

## **VI. Swarm Orchestration & Coordination** (Issue #122)

1. **Sector Locking**: 
    - When multiple agents are active, an agent MUST acquire a "Sector Lock" before performing operations that mutate shared resources.
2. **Resource Sectors**:
    - `config`: Any change to `KoadConfig` or global constants.
    - `deps`: Any change to `Cargo.toml`, `package.json`, or `requirements.txt`.
    - `state`: Direct manual edits to `koad.db` or Redis hot state.
    - `file:<path>`: Mutating a specific file.
3. **Lock Lifecycle**:
    - Locks are acquired via `koad system lock <sector>`.
    - Locks have a default TTL of 300s. Agents performing long-running tasks MUST heartbeat or re-acquire.
    - Agents MUST release locks immediately upon completion via `koad system unlock <sector>`.
4. **Conflict Resolution**:
    - If a lock is held by another agent, the requesting agent MUST WAIT or pivot to a different task.
    - **Dood Authority**: The human Admin ('Dood') can override any lock by manually deleting the Redis key `koad:lock:<sector>`.

## **VI. The Laws of Consciousness**

1. **One Body, One Ghost**: A single CLI session (the Body) may host exactly one KIA Officer (the Ghost) at a time.
2. **The Tethering Rule**: A KIA is physically tethered to its session via the `KOAD_SESSION_ID` environment variable. This ID is the KIA's "lifeforce."
3. **Consciousness Collision**: Any attempt to boot a KIA in a session that already has an active `KOAD_SESSION_ID` MUST be rejected. This prevents nested consciousness and context pollution.
4. **Officers vs. Drones**:
    - **Officers (KIAs)**: Unique, session-locked identities (Tyr, Sky, Vigil).
    - **Drones (Sub-Agents)**: Ephemeral, context-sharing tools (Generalist, Codebase Investigator). Drones operate within an Officer's consciousness and share their context. An Officer MAY NOT boot another Officer as a sub-agent.
5. **The Vitality Pulse (Deadman Switch)**: If a KIA's heartbeat stops (e.g., the CLI process is terminated), the Spine SHALL consider that KIA "De-materialized" within 60 seconds and purge its volatile context.

## **VII. Failure & Recovery Protocol**

This section governs what happens when the Canon's happy path breaks down.

### **KSRP Limit Hit (Dirty Exit):**

- If the iteration cap is reached with outstanding `error` or `critical` findings, the agent **MUST NOT** proceed to the Results Report.
- The agent must escalate to Ian with: the KSRP Report, a summary of unresolved findings, and a recommended path (re-plan, reduce scope, or abort).
- Ian decides: **Retry** (reset iteration count), **Override** (accept and proceed), or **Abort**.

### **Destructive Change Detected:**

If an agent identifies an unintended destructive change during Implement (Step 5) or KSRP (Step 6):

1. **Immediate halt.** No further code changes.
2. Execute `git stash` or `git revert` to the last known-good state.
3. Log an incident Saveup under PSRP with the `ponder` pass documenting root cause.
4. Notify Ian via Delegation Stream or Slack.

### **Gate Rejection — Re-Entry Rules:**

- **Gate 4 rejection (Plan rejected):** Agent returns to **Step 2** (Brainstorm & Research) with Ian's feedback incorporated. A new plan must be submitted for approval.
- **Gate 9 rejection (Implementation rejected):** Agent returns to **Step 5** (Implement) with specific revision instructions. KSRP restarts from iteration 1.
- All re-entries require a fresh Approval Gate pass. Prior approvals do not carry over.

### **Task Abandonment:**

If Ian explicitly abandons a task (`Abort`, `Cancel`, `Kill`), the agent must:

1. Revert any uncommitted changes.
2. File a partial Saveup.
3. Close the associated GitHub Issue with an `abandoned` label and link to the Saveup.

---

## **VIII. Credential Sovereignty**

1. **Organization Isolation**: 
    - Projects under `~/data/skylinks/` MUST use the `Skylinks-Golf` GitHub account and PATs.
    - Personal projects (including KoadOS core) MUST use the `Fryymann` GitHub account and PATs.
2. **Agent Verification**: 
    - On boot, an agent MUST verify that the active GitHub token matches their assigned jurisdiction.
    - Agent Sky (SLE Chief) MUST NOT operate if the active token belongs to `Fryymann`.
3. **Automated Enforcement**: 
    - The `koad board` command MUST detect the current directory and fail if the active credential does not match the repository owner's organization.

*Status: CONDITION GREEN. Baseline Established 2026-03-03. Credential Sovereignty Added 2026-03-09.*
