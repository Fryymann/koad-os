# Design Deep Dive — Sweep 07: Workspace Manager & Git Orchestration

> [!IMPORTANT]
> **Status:** PLAN MODE (Filesystem & Version Control)
> **Goal:** Design a robust, isolated workspace environment for multiple agents. Use Git Worktrees to allow parallel development on the same repository without physical path collisions.

---

## 1. The Problem: Physical Path Collisions
Currently, agents operate in the same physical directory. If Tyr is refactoring the Spine while Sky is updating the SLE logic in the same repo, they will fight over the same `.git` index and local file state.

## 2. The Solution: Git Worktree Orchestration
We will implement a `WorkspaceManager` inside the Spine that manages a dedicated pool of **Git Worktrees**.

### **A. Path Strategy**
Workspaces will be namespaced by Agent and Task ID:
`~/.koad-os/workspaces/{agent_name}/{task_id}/`

### **B. The "Worktree Spawn" Lifecycle**
When a task is dispatched (`koad assign <agent> <issue_id>`):
1. **Validation:** Spine verifies the target repo and branch.
2. **Creation:** Spine executes `git worktree add -b {agent}/{issue_id} ~/.koad-os/workspaces/{agent}/{issue_id} origin/main`.
3. **Mounting:** The `AgentSession` is updated with the unique `root_path` pointing to this isolated worktree.
4. **Isolation:** The agent is "jailed" to this path. Any tool calls (`read_file`, `write_file`) are prepended with this path.

## 3. Workspace State Management (Redis)
The `WorkspaceManager` will track active worktrees in Redis to prevent orphaned folders.

- **Key:** `koad:workspaces:{path_hash}`
- **Values:**
    - `agent`: Name of the assigned agent.
    - `issue_id`: The GitHub Issue link.
    - `created_at`: Timestamp.
    - `trace_id`: The ID of the command that spawned it.

## 4. Teardown & PR Orchestration
Agents no longer push code directly.
1. **Submission:** Agent signals task completion.
2. **Review:** The platform (or a Lead Agent like Sky) reviews the diff within the isolated worktree.
3. **Commit:** Upon approval, the Spine handles the `git commit` and `gh pr create` from that specific worktree.
4. **Cleanup:** Once the PR is merged, the Spine executes `git worktree remove` and nukes the directory.

## 5. Security: The "Isolation Mandate"
For the **SLE**, the `WorkspaceManager` will enforce a specific constraint:
- **Sandbox Worktrees:** SLE worktrees are created with a pre-configured `.env.sandbox` that contains only mock credentials. The Spine will block any attempt to mount a `worktree` without a verified sandbox configuration for Officer-rank agents.

---

## **Refined Implementation Strategy (v5.0)**
1.  **Crate Update:** Add a `git2` or `tokio::process::Command` wrapper to `koad-core` for safe git operations.
2.  **Spine Integration:** Add `workspace_manager` module to the Spine Engine.
3.  **Path Resolution:** Update all agent tools to use `session.context.root_path` dynamically.
4.  **Auto-Cleanup:** Implement a background "Debris Sweep" task that prunes worktrees whose sessions have gone DARK for > 24 hours.

---
*Next Sweep: The Idea Pipeline & Autonomous Event Routing.*
