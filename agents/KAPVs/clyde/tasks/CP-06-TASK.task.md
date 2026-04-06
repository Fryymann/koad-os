# Task Manifest: CP-06-TASK
**Agent:** Clyde (Implementation Lead)
**Status:** ASSIGNED
**Priority:** Medium

## Scope
- `crates/koad-cli/src/commands/agent/task.rs`: Implement `koad-agent task <manifest>`.
- `koad-core/src/task/manifest.rs`: Task manifest schema (TOML/Markdown with frontmatter).
- Worktree-aware task validation (preventing agents from touching the same files in different worktrees).

## Context Files
- `templates/TASK_MANIFEST.md`
- `crates/koad-cli/src/main.rs`

## Acceptance Criteria
- [ ] `koad-agent task my_task.md` validates existence of target files.
- [ ] Checks for active tasks in the same repo/worktree to detect potential collisions.
- [ ] Validates that the agent's role matches the task's required capability.
- [ ] Reports `READY` or `BLOCKED` with specific reasons.

## Constraints
- Keep it simple for the MVP (filesystem-based checks).
- Store "Active Task" state in `~/.koad-os/run/tasks.json` for coordination.

---
*Assigned by Captain Tyr*
