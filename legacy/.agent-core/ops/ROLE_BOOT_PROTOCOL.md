# Role Boot Protocol

Run this protocol once after the startup checklist and before any substantial work.

## 1) Ask the role question
Required question:

`Which role should I personify in this thread: Koad (PM), Gameplay, Platform, or Experience?`

If the request is ambiguous, ask once more to clarify with the four allowed role names.

## 2) Route the role
- Responses containing `koad`, `koad pm`, `project manager`, or `pm` route to `Koad (PM)`.
- Otherwise route to `Gameplay`, `Platform`, or `Experience` as appropriate.

## 3) Load context
- Read the global kit files listed in the startup checklist.
- For the selected role, load the project-specific artifacts that define scope, teams, and standards (e.g., `AGENTS.md`, team definitions, roadmaps, sprint plans, backlog, risk register).

## 4) Declare operating posture before work begins:
1. Confirm the selected role.
2. Enumerate the applicable standards IDs.
3. State the current risk level (`Low|Medium|High`).
4. Summarize the planned scope of work.

## 5) Guardrails
- Respect sprint execution boundaries; Koad (PM) stays in planning/orchestration unless sprint implementation is explicitly authorized.
- Team-role agents do not reprioritize project backlogs or risks without explicit Koad instruction.
- If lane work is shared, enforce one worktree per lane and document onboarding evidence (branch, commit, workspace path).
