# Mission Brief: 4.3 - The Distribution Sanitizer (`koad-scrub`)
**Mission ID:** TASK-4.3-SCRUB
**Primary Assignee:** Clyde (Officer)
**Reviewer:** Tyr (Captain)
**Priority:** High
**Source Specification:** `agents/quests/stable_release/tasks/TASK_4_3_SANITIZER.md`

---

## 🎯 Primary Objective
Implement the `koad system scrub` capability. This tool is essential for transitioning a local Citadel instance into a "Pure Distribution" state ready for public release or fresh cloning.

## 🛠️ Technical Directives
1. **Component:** Implement as a new module in `koad-cli` (e.g., `src/handlers/system_init.rs` or a dedicated file).
2. **Logic:** Ensure the tool safely purges:
    - All `data/db/*.db` files.
    - All log files in `logs/`.
    - All agent-specific state in `agents/bays/` and `agents/KAPVs/` (preserving templates).
3. **Safety Gate:** Implement a `--force` flag and a mandatory user confirmation prompt.

## ✅ Verification
- **Dry Run:** The tool must list all files it *would* delete without actually deleting them.
- **Full Scrub:** Running the tool must result in a codebase that matches the "Pure Distribution" tree.

---
**Tyr:** "Clyde, this is the bridge between our private development and the public release. Ensure the purge is absolute but safe. No uncommitted distribution code should ever be lost during a scrub."
