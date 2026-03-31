# KoadOS Worktree & Parallel Execution Conventions

## 🏗️ AGENT WORKTREE ARCHITECTURE
To enable high-throughput parallel development, every KAI (KoadOS Artificial Intelligence) is assigned a dedicated, persistent **Ghost Worktree**. This serves as the agent's personal operating environment, isolated from other agents' work and the `nightly` branch.

### Persistent Ghost Worktrees
- `~/.koad-os/` (Citadel Hub: **nightly** branch)
- `~/koad-clyde/` (Clyde: **ghost/clyde** branch)
- `~/koad-tyr/` (Tyr: **ghost/tyr** branch)
- `~/koad-cid/` (Cid: **ghost/cid** branch)

## 🌿 BRANCH MANAGEMENT PROTOCOL
1. **Sovereignty:** Each agent "owns" their ghost branch. The branch name is their identity.
2. **Synchronization:** Agents pull from `nightly` into their ghost branch daily.
3. **Delivery:** When a task is complete, the agent opens a PR from their ghost branch into `nightly`.
4. **Resets:** After a PR is merged, the agent should rebase or reset their ghost branch against the new `nightly` to start the next task.

## 🤝 PARALLEL COORDINATION
- **Task Manifests:** Every agent session MUST be governed by a Task Manifest (in their worktree's `tasks/` directory) that defines its current file scope.
- **Crate Locks:** Avoid working on the same crate in two different worktrees simultaneously. Check `koad system status` for active locks.
- **Inter-Agent Inbox:** Use `~/.koad-os/agents/inbox/` to notify others of cross-crate changes or upcoming merges.

---
*KoadOS Operational Standard | Tyr | 2026-03-28*
