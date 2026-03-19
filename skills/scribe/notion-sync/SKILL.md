# Skill: Notion-Sync (Scribe)

## Overview
Synchronizes designated Notion databases to the local KoadOS SQLite datastore and exports them as Markdown documentation. This skill maintains the boundary between Citadel-level (KoadOS) and Station-level (SLE) content.

## Protocol
When asked to "sync Notion" or "update requests", Scribe MUST:
1. Identify the target database scope (KoadOS vs. Skylinks).
2. Run the `koad bridge notion sync` command for the corresponding ID.
3. Export the content to the correct station level:
    - **KoadOS** (`324fe8ecae8f80f6babdff16436d5307`) -> `~/.koad-os/docs/requests/`
    - **Skylinks** (`326fe8ecae8f803bb152c882beea6318`) -> `/mnt/c/data/skylinks/docs/requests/`
4. Verify that no duplicate titles exist across the stations (KoadOS version takes precedence).

## Maintenance
- **Deduplication:** Periodically run title-based audits on `notion-sync.db` to identify and resolve draft or duplicate entries.
- **Verification:** Ensure files exported to SLE are accessible to Sky and her crew.

---
*Status: OPERATIONAL | Agent: Scribe | Updated: 2026-03-19*
