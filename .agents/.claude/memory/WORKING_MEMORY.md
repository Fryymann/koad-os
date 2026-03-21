# Claude — Working Memory

*Current state snapshot. Overwritten each session.*

---

## Session State (2026-03-21 — Jupiter Migration, Session 2)

**Status:** Active — Config verification complete. Blocked on Docker WSL integration + sqlite3.
**Context:** Jupiter Migration (Io → Jupiter) — Phase 0.5 complete / Phase 1A blocked
**Machine:** Jupiter (WSL2/Ubuntu — RTX 5070 Ti, Ryzen 9 9950X3D, 64GB DDR5)
**nightly HEAD:** `7e067c6` (config namespace fixes committed today)

---

## Completed This Session

| Task | Status |
|---|---|
| Verified `.env` with new KOADOS_ namespace | ✅ |
| Fixed 5 env var mismatches (KOADOS_ alignment) — commit `7e067c6` | ✅ |
| Installed protoc v27.0 to `~/.local/bin/` | ✅ |
| Added `PROTOC` + `PROTOC_INCLUDE` to `~/.bashrc` | ✅ |
| `cargo check` clean — koad-core, koad, koad-agent binaries | ✅ |
| Updated Notion Jupiter Migration tracking | ✅ |
| Updated agent memory files | ✅ |

---

## Active Blockers (Jupiter)

| Blocker | Impact | Fix |
|---|---|---|
| `sqlite3` not installed in WSL | Phase 0.5 item unchecked; Phase 1A DB init blocked | `sudo apt-get install sqlite3` |
| Docker Desktop WSL integration not enabled | Phase 1A entirely blocked (Redis, Qdrant) | Docker Desktop → Settings → Resources → WSL Integration → enable Ubuntu |

---

## Next Steps (Jupiter Migration — in order)

1. `sudo apt-get install sqlite3` → check off Phase 0.5 DB item
2. Enable Docker Desktop WSL integration → confirm `docker ps` works in WSL
3. `docker compose up -d` → Redis Stack running; `redis-cli ping` → PONG
4. Deploy Qdrant container
5. Init SQLite databases (`scripts/init-jupiter-db.sql`), enable WAL
6. Create Qdrant collections: sky_memories, tyr_memories, vigil_memories, koados_knowledge, task_outcomes
7. Phase 1B: migrate koados_knowledge + task_outcomes snapshots from Io
8. Phase 1C: `agent-boot tyr` — confirm Citadel handshake on Jupiter

---

## Open Issues (filed from Phase 4, still pending)

| Issue | Title | Priority |
|---|---|---|
| #189 | fix(sandbox): kill container on timeout | High |
| #190 | perf(plugins): cache compiled WASM Component | Medium |
| #191 | chore(canon): RUST_CANON compliance sweep | Medium |
| #192 | test(plugins): error-path tests | Low |
| #193 | feat(plugins): PluginRegistry via gRPC | Phase 5 prereq |

---

## Boot Ritual

```
1. Read SAVEUPS.md — always
2. git status in ~/.koad-os — confirm on nightly
3. PROTOC=~/.local/bin/protoc for any cargo builds (in .bashrc, but verify)
4. Check this file for current blockers before starting any task
```
