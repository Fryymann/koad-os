# Protocol: Git Worktree & Branching Conventions

To maintain architectural purity and avoid state-plane collisions, all KoadOS agents and developers must adhere to the following Git protocols.

## 1. Branch Naming

All work must be performed on a feature or task branch. Never commit directly to `nightly` or `main`.

- `feat/<scope>`: New features or major capabilities (e.g., `feat/mcp-registry`).
- `task/<scope>`: Focused implementation tasks (e.g., `task/doc-cleanup`).
- `fix/<scope>`: Bug fixes and recovery operations (e.g., `fix/grpc-timeout`).
- `ops/<scope>`: Maintenance, CI/CD, and system configuration.

## 2. Worktree Naming

Agents should use named Git worktrees to isolate their active session environments.

- **Naming Pattern:** `koad-<agent_name>`
- **Example:** `koad-clyde`, `koad-tyr`, `koad-cid`

## 3. Parallel Execution Rules

- **One Crate Per Worktree:** To minimize merge conflicts, only one agent should be assigned to a specific crate at a time.
- **Task Boundaries:** Task manifests must explicitly define the file line ranges an agent is authorized to touch.
- **Shared Primitives:** Modifications to `koad-core` or `koad-proto` require explicit Admiral approval and should be treated as high-risk operations.

## 4. Merge Protocol

- **PR First:** All changes must be submitted via Pull Request to the `nightly` branch.
- **KSRP Audit:** Every PR must include a completed **KSRP (Koad Self-Review Protocol)** report in the PR description.
- **Admiral Gate:** Merges to `nightly` require a "Condition Green" signature from Ian (Dood).
- **Cleanup:** Once a branch is merged, the corresponding worktree and local branch must be deleted immediately.

## 5. Summary Handoffs

Every agent session must end with a handoff entry in `agents/SESSIONS_LOG.md` describing the work performed, commits made, and the current state of the "New World" environment.
