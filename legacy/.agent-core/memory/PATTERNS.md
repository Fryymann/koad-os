# Patterns

Track recurring workflows, signal-response pairs, or reliable playbooks so future sessions can avoid reinventing the wheel.

## Pattern: Role Boot Routine
- **Trigger**: Starting a new Codex thread or work session in any repository.
- **Steps**:
  1. Bootstrap global kit (`AGENTS.md`, identity, mission).
  2. Load working memory, learnings, facts, and preferences from the global kit.
  3. Ask the role question defined in `ROLE_BOOT_PROTOCOL.md` and follow the resulting path.
- **Validation**: Substantial work never starts with an unresolved role question.

## Pattern: Standards Freshness Gate
- **Trigger**: Any standards-dependent planning or execution task.
- **Steps**:
  1. Run `standards_sync_status.py` with the manifest and required-sources documents.
  2. Confirm the returned status is `FRESH` before relying on enforced policies.
  3. If stale or missing, capture blocker details in the session log and pause work.
- **Validation**: Work streams only resume once the freshness gate passes.

## Pattern: Knowledge Capture Loop
- **Trigger**: Finish of a meaningful change, review, or decision point.
- **Steps**:
  1. Run the `SAVEUP_PROTOCOL` to capture session summary and role metadata.
  2. Add new lessons to `LEARNINGS.md`, facts to `FACTS_LEDGER.md`, and patterns when the workflow recurs.
  3. Mirror operational decisions into `.agent-ops/sessions/SESSION_LOG.md` and `.agent-ops/decisions/DECISION_LOG.md` if policies or scope shifted.
- **Validation**: Future agents find the lessons, facts, and decisions in the global kit without re-asking the same questions.
