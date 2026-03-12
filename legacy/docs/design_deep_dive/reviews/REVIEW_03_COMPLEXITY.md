# Architect Review 03: Operational Complexity

**Reviewer:** Senior Systems Architect (Tyr)
**Target:** Phase 4 (Workspace Manager) & The Idea Pipeline
**Verdict:** **RED (Significant Process Overhaul Required)**

## 1. The Worktree Cleanup Paradox
**The Plan:** The Spine creates a Git Worktree for every task (`~/.koad-os/workspaces/sky/SG-142`). When the PR is merged, the Spine deletes it.
**The Vulnerability:** If the agent fails, or the PR is abandoned, or the Admiral decides to delete the issue manually on GitHub, the Spine never receives the "merge" event. Over a month, the `workspaces/` directory will bloat with dozens of orphaned worktrees, each containing a full checkout of the repository, eventually consuming all available disk space. Furthermore, Git becomes extremely unhappy if a worktree is deleted via `rm -rf` without running `git worktree prune`.
**The Fix:** 
1. **Garbage Collection:** We must implement an asynchronous `WorkspaceSweeper` task in the Spine. It periodically scans `~/.koad-os/workspaces/`. If a folder hasn't been touched in 72 hours, it executes `git worktree prune` and forcefully removes it.
2. **Ephemeral Scopes:** Re-evaluate if *every* task needs a worktree. For read-only tasks (like "analyze this module"), the agent should use the primary checkout. Worktrees should only be spawned upon the *first write operation*.

## 2. The Sandbox Isolation Failure
**The Plan:** Inject `.env.sandbox` into Sky's worktree to protect the SLE from production SCE credentials.
**The Vulnerability:** KoadOS executes tools as the `ideans` user. Even if we inject a sandbox `.env`, Sky can simply execute `run_shell_command("cat ~/.config/gcloud/credentials.db")` or `cat ~/.koad-os/config/` and extract the production API keys. The "Isolation Mandate" is currently security theater.
**The Fix:** True isolation requires containerization, which is too heavy. To maintain our lightweight model, we must build a **Command Filter Interceptor** in the `Sandbox` module. It must actively block `cat`, `grep`, `less`, and `vim` targeting sensitive directories (`~/.config`, `~/.ssh`, `~/.koad-os`) if the executing agent is ranked as an `Officer` or below.

## 3. The "Ollama Intake" Bottleneck
**The Plan:** Use a local 7B model (Ollama) to parse the "Think Tank" input and create structured GitHub issues.
**The Vulnerability:** If the local machine is under heavy load (e.g., rust-analyzer compiling the workspace), a 7B model inference can take 15-30 seconds. The Admiral drops an idea and waits... and waits. If they close the web deck, does the idea vanish?
**The Fix:** The Idea Pipeline must be **Fully Asynchronous and Durable**. 
1. Idea is written to SQLite `intake_queue` instantly (Latency: <5ms).
2. The UI reports "Idea Queued."
3. A background worker pulls from the queue, waits for CPU availability, runs the Ollama inference, creates the issue, and then marks the row as `Processed`. Never block the Admiral's UI on a local LLM inference.