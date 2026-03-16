# Plan: KoadOS XP System Integration

## Objective
Implement a lightweight Experience (XP) system to reward Canon compliance and track agent trust levels.

## Key Files
- `docs/protocols/CONTRIBUTOR_CANON.md`: Update Saveup format and add penalty rules.
- `docs/agent_ref_book/agent_ref_book.md`: Publish the Agent Levels table.
- `~/.tyr/identity/XP_LEDGER.md`: Create the first XP ledger.
- `~/.tyr/memory/FACTS.md`: Record XP System implementation facts.

## Implementation Steps

### 1. Update CONTRIBUTOR_CANON.md
- **Section III (The Experience Point System):** Add detail about XP sources and sinks based on the feature request.
- **Section IV (Saveup Format):** Update the canonical Saveup template to include XP fields.
- **Add Section VI (Failure & Recovery):** Formalize the XP penalty rules for behavioral violations.

### 2. Update agent_ref_book.md
- Add a new section "Agent Levels & Trust" containing the XP threshold table and titles.

### 3. Initialize Agent Ledgers
- Create `~/.tyr/identity/XP_LEDGER.md` with an initial entry for the current XP (150).
- Format: Date | Task ID | Event | Delta | Running Total | Level.

### 4. Record implementation facts
- Update `~/.tyr/memory/FACTS.md` to reflect the transition to an XP-aware environment.

### 5. Retroactive XP calculation for current session
- Calculate XP for the "Agent Tools & Intelligence" task.
- Complexity: Complex.
- KSRP Exit: Clean (1st iteration).
- Full PSRP: Yes.
- Expected XP: +45 (KSRP) + +5 (PSRP) = +50 XP.

## Verification
- Verify all files are updated and correctly formatted.
- Confirm the new Saveup for this task uses the updated format.
