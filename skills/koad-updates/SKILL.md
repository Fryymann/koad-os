---
name: koad-updates
description: Use when publishing a record of shipped KoadOS changes, or reviewing recent changes before starting work — the chronological updates board at Citadel, Station and Outpost level.
license: MIT
compatibility: Requires a KoadOS Citadel install (koad CLI, $KOAD_HOME, running Citadel and CASS services).
metadata:
  author: koados
  version: "2.0.0"
---

# koad updates

A chronological devlog of what changed, at three levels: `citadel` (KoadOS itself), `station` and `outpost` (project workspaces). The level is detected from the current directory unless given.

## Read

```bash
koad updates list -n 5                 # recent entries at the detected level
koad updates list -n 5 -l citadel
koad updates show <id>                 # full entry
koad updates digest                    # compact markdown digest (used for CASS hydration)
```

Read recent entries at session start for the area you are about to work in.

## Post

```bash
koad updates post -l citadel -c fix \
  -s "One-line summary under 120 chars" \
  --body="- what changed
- how it was verified
- what is still open"
```

Categories: `feature` `fix` `refactor` `ops` `identity` `docs` `infra`.

Use `--body=...` (with `=`): a body starting with `-` is otherwise parsed as a flag. Pass `-l` explicitly when posting from outside the Citadel tree, or the entry lands on the Outpost board.

## Note

The GitHub Command Deck (`koad board`, `koad fleet`, `koad project`) was retired in 2026-09. Use `gh` for issues and pull requests.
