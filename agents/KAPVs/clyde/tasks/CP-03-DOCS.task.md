# Task Manifest: CP-03-DOCS
**Agent:** Scribe (Context Distillation)
**Status:** ASSIGNED
**Priority:** Low

## Scope
- `scripts/sync-status.sh`: Script to parse `updates/` and update `MISSION.md`.
- `MISSION.md`: Update "Active Phase" and "Status" sections.
- `AGENTS.md` (root): Update agent team awareness.

## Context Files
- `updates/*.md`
- `MISSION.md`
- `AGENTS.md`

## Acceptance Criteria
- [ ] `sync-status.sh` correctly updates `MISSION.md` based on latest update files.
- [ ] Core documentation reflects the "Active Phase" accurately (currently Phase 7.5).
- [ ] Distilled summaries are concise and high-signal.

## Constraints
- Use `grep`, `sed`, or a small JS script.
- Do NOT delete existing history.
- Ensure formatting remains consistent.

---
*Assigned by Captain Tyr*
