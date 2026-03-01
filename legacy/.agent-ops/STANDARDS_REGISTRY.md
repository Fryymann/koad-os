# Standards Registry

Purpose: codify the operating standards that every workspace relying on this global kit must follow.

## How to use
- Each standard has a stable ID.
- Source points to required local artifact (usually under `~/.koad-os`).
- Local interpretation defines expectations.

## Standards

### STD-001 - Change Safety
- Status: Active
- Source: `.agent-ops/STANDARDS_REGISTRY.md`
- Intent: Avoid unsafe or hard-to-rollback changes.
- Local interpretation:
  - Favor incremental edits with a clear rollback plan.
  - Describe risky assumptions before touching broad swaths of the codebase.

### STD-002 - Documentation Minimum
- Status: Active
- Source: `.agent-ops/STANDARDS_REGISTRY.md`
- Intent: Maintain continuity through durable records.
- Local interpretation:
  - Update `~/.koad-os/.agent-ops/sessions/SESSION_LOG.md` for substantial tasks.
  - Add governance-impacting decisions to `.agent-ops/decisions/DECISION_LOG.md`.

### STD-003 - Canonical Source Freshness Gate
- Status: Active
- Source: `.agent-core/scripts/standards_sync_status.py`
- Intent: Ensure enforced policies are up-to-date before policy-dependent work.
- Local interpretation:
  - Run the freshness script (`standards_sync_status.py`) before relying on enforced standards.
  - If the manifest is missing or stale, stop planning/implementation until the source is refreshed.

### STD-004 - Memory Mirror
- Status: Active
- Source: `.agent-core/memory/LEARNINGS.md`
- Intent: Keep learnings, patterns, facts, and preferences in sync with operations.
- Local interpretation:
  - Record every new lesson in `LEARNINGS.md` with observation, impact, and behavior update.
  - Mirror operation-level insights (patterns, facts, preferences) in their respective ledgers.

### STD-005 - Sprint Execution Authorization
- Status: Active
- Source: `.agent-core/ops/SAVEUP_PROTOCOL.md`
- Intent: Prevent premature sprint execution by the PM beyond delegated lanes.
- Local interpretation:
  - Koad (PM) remains in planning/orchestration mode unless sprint implementation is explicitly requested.
  - Team-role agents run the actual implementation lanes and use lane-specific saveups.

### STD-006 - Role Selection Boot Gate
- Status: Active
- Source: `.agent-core/ops/ROLE_BOOT_PROTOCOL.md`
- Intent: Ensure every agent explicitly resolves its role before work.
- Local interpretation:
  - Ask the role question before any substantial work.
  - Document selected role, standards IDs, risk level, and planned scope.

### STD-007 - Support Branch Scope
- Status: Active
- Source: `.agent-core/ops/SAVEUP_PROTOCOL.md`
- Intent: Keep shared memory/saveup artifacts on the designated support branch (e.g., `koad-os`).
- Local interpretation:
  - Track shared ledgers (`SAVEUP_CALLS.md`, `LOG.md`, `PROJECT_PROGRESS.md`) only when working on the support branch.
  - Feature branches must use lane-isolated journals until they are promoted.

### STD-008 - Lane Isolation Guardrail
- Status: Active
- Source: `.agent-core/ops/ROLE_BOOT_PROTOCOL.md`
- Intent: Prevent cross-lane interference.
- Local interpretation:
  - Use one worktree per lane branch.
  - Capture onboarding evidence (worktree path, branch, base commit) before editing.
  - Require lane handoffs to include PR metadata and review states.

### STD-009 - Handoff Documentation & Evidence
- Status: Active
- Source: `.agent-ops/decisions/DECISION_LOG.md`
- Intent: Keep handoffs transparent with documented acceptance criteria, tests, and coverage evidence.
- Local interpretation:
  - Non-support (release-line) changes must link to implementation docs covering files/behavior.
  - Include automated test results plus negative-path/regression evidence.

### STD-010 - Role-Aware Saveup Continuity
- Status: Active
- Source: `.agent-core/ops/SAVEUP_PROTOCOL.md`
- Intent: Preserve role/context attribution in continuity records.
- Local interpretation:
  - Every saveup entry includes `role` and `context_ref`.
  - Team-role saveups do not directly rewrite project backlog/risk artifacts; they note proposed updates instead.

### STD-011 - Dashboard & Progress Sync
- Status: Active
- Source: `.agent-core/ops/SAVEUP_PROTOCOL.md`
- Intent: Keep progress views aligned with merged work.
- Local interpretation:
  - Use automated dashboards or issue-managed views rather than manual per-change commits.
  - When needed, `saveup` can refresh local dashboards, but default publication should remain merge-driven.

### STD-012 - GitHub Authentication Context Awareness
- Status: Active
- Source: `.agent-ops/STANDARDS_REGISTRY.md`
- Intent: Ensure the correct GitHub PAT is used based on the project context.
- Local interpretation:
  - When in `~/data/skylinks/` (or `/mnt/c/data/skylinks/`), use `GITHUB_SKYLINKS_PAT`.
  - In all other directories, use `GITHUB_PERSONAL_PAT` by default.
  - Verify current directory before performing GitHub operations.
