---
name: koad-intel
description: Use when storing a fact, learning or reflection in durable CASS memory from the command line, or querying prior knowledge from the CLI when the citadel-memory MCP tools are not available.
license: MIT
compatibility: Requires a KoadOS Citadel install (koad CLI, $KOAD_HOME, running Citadel and CASS services).
metadata:
  author: koados
  version: "2.0.0"
---

# koad intel

CLI access to CASS memory. With the `citadel-memory` MCP tools available, prefer them for reads (see the `cass-recall` and `cass-search` skills); use the CLI for writes.

## Query

```bash
koad intel query "<term>"                # CASS semantic matches, then the local archive
koad intel query "<term>" --limit 20
koad intel query "<term>" --agent hermes
```

## Store

```bash
koad intel remember fact "<statement>"          # durable truths: ports, conventions, decisions
koad intel remember learning "<insight>"        # discoveries, patterns, bug causes
koad intel remember fact "<statement>" -t tag1,tag2
koad intel ponder "<reflection>" -t design      # persona reflections, tradeoffs, post-mortems
```

Writes need a live session: source the session env from the `agent-boot` skill first. If a write fails with `Session not found or expired`, re-mint and retry once.

Write statements that stand alone: include the component, the date when it matters, and why.

## Other

```bash
koad intel guide [quick|canon|workflow|ais|saveup|worktree]   # KoadOS field guide
```

## Verify writes

After storing something important, confirm it can be recalled (`memory.search_semantic` via MCP, or `koad intel query`). A write that cannot be found is not memory.
