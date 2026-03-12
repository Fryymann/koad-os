# koadOS Global Agent Bootstrap

These documents capture the reusable memory framework, learning cadence, knowledge base, and protocols that Codex agents should rely on across workspaces.

## Boot Hook (Required)
1. Read this file plus `~/.koad-os/.agent-core/IDENTITY.md` and `~/.koad-os/.agent-core/MISSION.md` to understand the global posture.
2. Review the memory ledgers under `~/.koad-os/.agent-core/memory/` (Working Memory, Learnings, Facts, Patterns, User Preferences) so you know what is already known.
3. Run the global standards gate:
   ```
   python3 ~/.koad-os/.agent-core/scripts/standards_sync_status.py --manifest ~/.koad-os/.standards/sync_manifest.json --required-sources ~/.koad-os/.agent-ops/CANONICAL_REQUIRED_SOURCES.md --max-age-hours 24
   ```
4. Read `~/.koad-os/.agent-ops/STANDARDS_REGISTRY.md` and `~/.koad-os/.agent-ops/CANONICAL_REQUIRED_SOURCES.md` to understand what this kit enforces.
5. Run the role boot protocol defined at `~/.koad-os/.agent-core/ops/ROLE_BOOT_PROTOCOL.md` before substantial work.
6. After the global kit steps, load any project-specific bootstrap (e.g., the local `AGENTS.md`, team definitions, roadmap files, and environment constraints) and treat those instructions as the final authority for that workspace.

## Memory & Knowledge Procedures
- Keep `~/.koad-os/.agent-core/memory/LEARNINGS.md` and `FACTS_LEDGER.md` as append-only sources; add only new observations with explicit reasoning, value, and behavior updates.
- Track recurring workflows or playbooks in `PATTERNS.md` so similar future situations reuse proven responses.
- Record confirmed user style preferences inside `USER_PREFERENCES.md` when the user explicitly accepts a way of working.
- Working memory summarizes the current project context, open unknowns, and risks in `WORKING_MEMORY.md`; update it whenever priorities or scope change.

## Operational Rules
- Each saveup call must follow `~/.koad-os/.agent-core/ops/SAVEUP_PROTOCOL.md` and include a role plus `context_ref` metadata.
- Use lane-isolated saveups for feature branches and avoid writing shared ledgers (`SAVEUP_CALLS.md`, `LOG.md`, `PROJECT_PROGRESS.md`) outside the designated support branch.
- Keep the global kit synchronized with any project-specific updates; this file should be reread whenever the kit refreshes or when you bootstrap a new workspace.

## Scope
- This kit provides the generic portion of Koad OS. Projects still provide their own concerns (team structures, roadmaps, sprint plans, `CODEX_ROLE_PROMPTS.md`, or branch policies).
- When local instructions conflict with these global protocols, treat the stricter instructions as controlling but keep this kit as the reference for shared memory, learning, and saveup behavior.
