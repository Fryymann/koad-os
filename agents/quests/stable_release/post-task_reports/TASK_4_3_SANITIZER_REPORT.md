# Post-Task Report: 4.3 - The Distribution Sanitizer (`koad system scrub`)
**Status:** Complete
**Assignee:** Clyde (+ clyde-dev / clyde-qa)
**Date:** 2026-04-13

## Objective
Create a reliable "Citadel-to-Distribution" bridge tool that sanitizes the KoadOS repository for sharing, cloning, or public release — removing local agent data, private logs, and runtime state without touching tracked source.

## Completed Work

### Implementation: `crates/koad-cli/src/handlers/system.rs`

Added a `// ── Distribution Sanitizer ──` section implementing:

- **`ScrubAction` enum** — four variants: `DeleteFile`, `TruncateFile`, `ResetFile { path, content }`, `DeleteDir`. Each variant implements `describe()` for human-readable dry-run output.
- **`collect_scrub_targets(home)`** — walks the KoadOS home directory and builds an action list covering all eight target classes:
  1. `data/db/` — deletes all `.db`, `.db-shm`, `.db-wal` files.
  2. `logs/` — truncates all files (keeps the files, empties content).
  3. `run/` — deletes `.sock` and `.pid` files.
  4. `agents/bays/` — recursively deletes all subdirectories.
  5. `agents/KAPVs/` — recursively deletes all subdirs; deletes loose files that don't start with `TEMPLATE`.
  6. `SESSIONS_LOG.md` — truncates to zero bytes.
  7. `TEAM-LOG.md` — resets to a distribution release header stub.
  8. `cache/` — deletes all files and subdirectories.
- **`execute_scrub_actions(actions, dry_run)`** — executes or simulates the action list. Dry-run prints every action with a `[dry-run]` label; live run executes each action with contextual error messages.
- **`handle_scrub(home, dry_run, force)`** — top-level orchestrator:
  - Git check: warns (non-blocking) if the working tree has uncommitted changes.
  - Lists all identified targets before acting.
  - Dry-run short-circuits before confirmation.
  - Confirmation gate: prompts user to type `SCRUB` unless `--force` is passed.
  - Reports count of completed actions on success.

### CLI Wiring
- `koad system scrub` subcommand with `--dry-run` and `--force` flags wired in `cli.rs` and dispatched from the `System` handler.

## Verification Results
- `test_collect_scrub_targets_finds_expected_files`: pass — fake home with 8+ target artifacts; collector finds ≥ 8 actions.
- `test_collect_scrub_targets_preserves_template_files`: pass — `TEMPLATE.md` in `agents/KAPVs/` is NOT included in action list.
- `test_execute_scrub_actions_dry_run_leaves_files_intact`: pass — dry-run produces no filesystem changes.
- `test_execute_scrub_actions_truncate`: pass — truncate action zeroes file content without deleting the file.
- All 4 tests: `ok` (0 failed).

## Constraints Met
- Confirmation prompt required unless `--force` is provided.
- Git dirty-tree check runs before any destructive action.
- `TEMPLATE*` files in `agents/KAPVs/` are preserved.
- Post-scrub instructions direct user to `koad system init` for re-initialization.

## Risk Notes
- The `logs/` truncation strategy keeps log files present (avoids breaking systemd/service expectations) while clearing sensitive content.
- `--force` bypasses the interactive prompt; suitable for CI/release pipelines but should not be the default in human workflows.
