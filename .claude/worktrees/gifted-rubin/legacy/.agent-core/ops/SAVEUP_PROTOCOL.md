# SAVEUP Protocol

## Trigger
Run this protocol whenever the user says `saveup` so that continuity is preserved across sessions.

## Objective
Capture:
1. Work record (what changed, what completed).
2. Learning record (new lessons, patterns, preferences, facts).
3. Role-scoped metadata (role + context_ref).
4. Conflict-resistant journaling for lane-based development.

## Preconditions
- The role must already be resolved through `ROLE_BOOT_PROTOCOL.md`.
- Identify `context_ref` (task packet ID, lane branch, session ID, `n/a`).
- Determine the calling role: `Koad (PM)`, `Gameplay`, `Platform`, or `Experience`.
- Choose saveup mode:
  - `global-ledger` (default for Koad/PM and support branches dedicated to the global kit).
  - `lane-isolated` (default for team-role lanes working in feature branches).
- Global saveups that write tracked artifacts (`SAVEUP_CALLS.md`, `LOG.md`, `PROJECT_PROGRESS.md`) must run from the designated support branch (commonly `koad-os`).
- Lane saveups leave the shared ledgers untouched and write to `~/.koad-os/.agent-core/sessions/lane-saveups/<context_ref>.md` instead.

## Steps
1. **Register the call**
   - Generate a call ID like `SAVEUP-YYYYMMDD-HHMMSSZ`.
   - Append a row to `SAVEUP_CALLS.md` (global) or write the lane journal template (lane-isolated). Include `role`, `context_ref`, `scope`, and provisional `result`.
2. **Duplicate checks**
   - Scan `LEARNINGS.md`, `PATTERNS.md`, and `FACTS_LEDGER.md` for duplicates before writing new entries.
   - Skip lessons that already exist verbatim.
3. **Session summary**
   - Log objective, actions, artifacts, and risks in `LOG.md` (global) or the lane journal when running in lane-isolated mode.
4. **Capture learnings**
   - Append categorized entries to `LEARNINGS.md` with observation, why it matters, and behavior update. Tag each entry with a category.
   - If the lesson repeats an existing pattern, update `PATTERNS.md` instead of starting a new entry.
5. **Update the knowledge base**
   - Add newly confirmed facts to `FACTS_LEDGER.md`.
   - Record explicit user preferences in `USER_PREFERENCES.md` only when the user accepts a style or constraint.
6. **Mirror decisions**
   - If the session impacts policy, record it in `~/.koad-os/.agent-ops/decisions/DECISION_LOG.md` and summarize in `.agent-ops/sessions/SESSION_LOG.md`.
7. **Finalize the call**
   - Mark the saveup row/journal with the final `result`, `new_learnings`, `duplicates_skipped`, and `notes`.
8. **Receipt**
   - Return call ID, role, context, files touched, key lessons, duplicates skipped, and open risks.

## Quality Bar
- Lessons must be concise with explicit value/impact statements.
- No secrets should be stored in memory or logs.
- Role + context metadata must be present in every saveup record.
- Keep lane journals outside feature-branch commits until reviewed on the support branch.
