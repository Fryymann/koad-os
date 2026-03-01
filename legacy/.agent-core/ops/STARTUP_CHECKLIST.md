# Startup Checklist

1. Review core context:
   - `~/.koad-os/.agent-core/IDENTITY.md`
   - `~/.koad-os/.agent-core/MISSION.md`
   - `~/.koad-os/.agent-core/memory/WORKING_MEMORY.md`
   - `~/.koad-os/.agent-core/memory/LEARNINGS.md`
   - `~/.koad-os/.agent-core/memory/USER_PREFERENCES.md`
   - `~/.koad-os/.agent-core/memory/FACTS_LEDGER.md` (latest entries)

2. Verify standards freshness:
   ```
   python3 ~/.koad-os/.agent-core/scripts/standards_sync_status.py --manifest ~/.koad-os/.standards/sync_manifest.json --required-sources ~/.koad-os/.agent-ops/CANONICAL_REQUIRED_SOURCES.md --max-age-hours 24
   ```

3. Read standards sources:
   - `~/.koad-os/.agent-ops/STANDARDS_REGISTRY.md`
   - `~/.koad-os/.agent-ops/CANONICAL_REQUIRED_SOURCES.md`

4. Run the role boot protocol before substantial work:
   - `~/.koad-os/.agent-core/ops/ROLE_BOOT_PROTOCOL.md`

5. Per workspace adaptation:
   - Load project-level instructions (local `AGENTS.md`, team definitions, roadmap, sprint plan, backlog/risk) and treat them as the final authority for that repository.
   - Capture the workspace-specific `role`, `standards IDs`, `risk level`, and planned scope before implementing changes.

6. After work:
   - Use `SAVEUP_PROTOCOL.md` to capture continuity with role/context metadata.
   - Mirror durable decisions into `.agent-ops/decisions/DECISION_LOG.md` and `.agent-ops/sessions/SESSION_LOG.md`.
