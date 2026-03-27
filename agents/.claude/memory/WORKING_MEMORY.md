# Claude — Working Memory

*Current state snapshot. Overwritten each session.*

---

## Session State (2026-03-21 — Jupiter Migration, Session 3)

**Status:** Phase 1A + 1B complete. Paused for saveup. Ready for Phase 1C.
**Machine:** Jupiter (WSL2/Ubuntu — RTX 5070 Ti, Ryzen 9 9950X3D, 64GB DDR5)
**nightly HEAD:** `f37058b` (Phase 1B restore commit)

---

## Completed This Session

| Task | Commit | Status |
|---|---|---|
| KOADOS_ env namespace fixes | `7e067c6` | ✅ |
| protoc installed to `~/.local/bin/` | — | ✅ |
| Redis Stack deployed + validated | `60930e3` | ✅ |
| Qdrant deployed + 5 collections created | `60930e3` | ✅ |
| SQLite initialized (WAL, 8 tables) | `60930e3` | ✅ |
| `init-jupiter-db.sql` bug fixed | `60930e3` | ✅ |
| Phase 1B: `koados_memory_transfer.tar.gz` unpacked | `f37058b` | ✅ |
| Tyr vault → `~/.tyr/` | — | ✅ |
| Sky vault → `~/data/skylinks/agents/sky/` | — | ✅ |
| Agent bays (.tyr, .cid, .scribe) restored | `f37058b` | ✅ |
| 4 SQLite DBs restored + WAL re-applied | — | ✅ |
| Redis dump.rdb loaded, FT index recreated | — | ✅ |
| Notion updated throughout | — | ✅ |

---

## Infrastructure State (Jupiter)

| Service | Status | Details |
|---|---|---|
| Redis Stack | ✅ Running | port 6379, `agent_context` FT index live |
| Qdrant | ✅ Running | port 6333/6334, 5 collections |
| `koad.db` | ✅ Ready | WAL, knowledge + procedural tables |
| `cass.db` | ✅ Ready | WAL, episodic_memories + fact_cards |
| `codegraph.db` | ✅ Ready | symbols table |
| `notion-sync.db` | ✅ Ready | WAL, pages + sync tables |
| koad-citadel | ❓ Not started | Required for Phase 1C boot handshake |

---

## Next Steps

1. **Phase 1C** — `agent-boot tyr`
   - May need `koad system start` first (launches koad-citadel binary)
   - Watch for gRPC handshake — Citadel must be running for session lease
   - If Citadel not running, boot falls back gracefully (env exports still work)
2. Validate Tyr can read/write to Redis + SQLite on Jupiter
3. Run smoke-test task through Tyr to validate full memory pipeline
4. Update Notion Phase 1C checklist

---

## Key Facts for Next Session

- Redis FT index schema: `FT.CREATE agent_context ON HASH PREFIX 1 ctx: SCHEMA agent_id TAG session_id TAG content TEXT timestamp NUMERIC SORTABLE`
- Redis default password: `koados_secret` (docker-compose default; `KOADOS_AUTH_REDIS` blank in .env)
- Qdrant: all 5 collections exist, 1536-dim Cosine, empty (rebuild naturally)
- `koad.db` identities table is empty — identities load from TOML (`config/identities/`)
- koad-agent boot: `eval $(koad-agent boot tyr)` — exports KOAD_AGENT_NAME, KOAD_AGENT_ROLE, etc.

---

## Boot Ritual

```
1. Read SAVEUPS.md — always
2. git status in ~/.koad-os — confirm on nightly, HEAD should be f37058b
3. docker ps — confirm Redis Stack + Qdrant running
4. PROTOC=~/.local/bin/protoc for any cargo builds (in .bashrc)
```
