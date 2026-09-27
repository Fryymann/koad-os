---
name: koad-intel
description: Use when storing a fact or learning to durable memory, querying prior knowledge from the CLI, reading a precise file range, or recording an architectural reflection.
---

# koad intel

CLI memory operations against CASS. Use for direct writes and targeted reads outside of MCP context.

## Decision: CLI vs MCP

```
In a session with citadel-memory MCP? → use MCP tools (cass-recall, cass-search)
No MCP / need to write memory?        → use koad intel CLI, after verifying its active Citadel path
```

### Jupiter scheduled-shell caveat

In Hermes cron/scheduled shells, `koad intel` may resolve its database under the unrelated `~/.koad-os` tree even after a Jupiter boot and fail with `unable to open database file: /home/ideans/.koad-os/data/db/koad.db`. Do not repeat or flood logs with retries.

For Jupiter Hermes, use the live partition-bound MCP bridge at `http://127.0.0.1:9745/mcp` when the native `citadel-memory` tools are not exposed:

1. Check `GET /health`.
2. Call MCP `status.citadel` and verify partition `hermes_jupiter_ideans`.
3. Use `memory.list_topics`, `memory.recall`, or `memory.commit` through that bridge.
4. Verify every commit by recall/search or the returned card identifier.

Use direct CLI writes only when the CLI is demonstrably pointed at the intended Citadel and a live session identity exists.

## Commands

### Query memory
```bash
koad intel query "<term>"            # search by keyword/regex
koad intel query "<term>" --limit 20
koad intel query "<term>" --tags rust,grpc
koad intel query "<term>" --agent clyde
```

### Commit a fact
```bash
koad intel remember fact "<statement>"
koad intel remember learning "<technical insight>"
```

Facts: persistent system truths (ports, conventions, decisions).  
Learnings: technical discoveries, patterns, bug fixes.

### Record a reflection
```bash
koad intel ponder "<architectural thought>" --tags design,grpc
```

Reflections are persona-specific. Use for design decisions, tradeoffs, post-mortems.

### Read file snippet (No-Read rule)
```bash
koad intel snippet <path> --start <N> --end <N>
koad intel snippet crates/koad-cass/src/lib.rs --start 45 --end 80
```

Use instead of reading entire files. Satisfies the No-Read efficiency rule.

### Other
```bash
koad intel mind          # cognitive health + learning status
koad intel guide <topic> # KoadOS field guide lookup
koad intel scan          # deep workspace scan for project roots
```

## Common Mistakes

| Mistake | Fix |
|---|---|
| Reading entire files for one function | Use `snippet --start --end` |
| Storing facts during MCP sessions | Still do it — CLI writes are durable regardless of MCP state |
| Generic `remember fact` with no context | Include domain, crate, or component in the statement |
