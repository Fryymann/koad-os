# Dark Mode Local Persistence Format (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #152

## 1. Requirement
When an agent loses Citadel connectivity, they must write to a standardized `.md` structure that CASS can reliably parse.

## 2. Path Convention
- **Format:** `~/.<agent>/memory/session_<session_id>.md`
- **Example:** `~/.helm/memory/session_20260312_1900.md`

## 3. Schema (Frontmatter)
All files MUST lead with a TOML block.

```toml
version = "5.0"
agent = "helm"
session_id = "20260312_1900"
timestamp = 1773369516  # Unix
trace_id = "TRC-HELM-a1b2"
status = "OFFLINE"       # OFFLINE | RECOVERED | SYNCED
```

## 4. Hierarchy Reconciliation
1. **Dood Overrides:** Manual edits by Ian always win.
2. **Citadel State:** If connectivity is restored, the Citadel's last known state is the primary source.
3. **Local Save:** Used to hydrate the agent's context during the "Dark Mode" session.
4. **Conflict:** If a local save conflicts with Citadel history, quarantine the save to `.koad-conflict/` for Dood review.

## 5. Constraints
- **Human-Readable:** The body of the markdown MUST be clear prose/logs.
- **Portability:** Use relative paths or standardized agent-bay aliases.