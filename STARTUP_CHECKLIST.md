# Jupiter Migration Checklist — KoadOS Citadel Transfer

## Ⅰ. Infrastructure (Phase 1A)
- [ ] **Docker Up:** `docker compose up -d` (Verify Redis Stack is running on port 6379).
- [ ] **Redis Search Check:** Run `docker exec koad-redis-stack redis-cli FT._LIST`.
- [ ] **SQLite Provision:** Run `sqlite3 ~/.koad-os/koad.db < scripts/init-jupiter-db.sql`.
- [ ] **WAL Verification:** `sqlite3 ~/.koad-os/koad.db "PRAGMA journal_mode;"` (Must return `wal`).

## Ⅱ. Environment & Secrets
- [ ] **Env Sync:** `cp .env.template .env` and populate secrets.
- [ ] **Hardware Abstraction:** Verify `$SKYLINKS_HOME` is set in `.env` for Jupiter paths.
- [ ] **PimpedBash Install:** Run `.pimpedbash/install.sh` on Jupiter.

## Ⅲ. Crew & Identity
- [ ] **Identity Check:** `koad identity list` (Verify Sky, Vigil, Tyr).
- [ ] **Handshake:** Run `agent-boot tyr`.
- [ ] **L4 Promotion:** Confirm Tyr has promoted Institutional Knowledge to `procedural_memory`.

## Ⅳ. Validation
- [ ] **Sanctuary Test:** Verify MCP-only filesystem writes are functional.
- [ ] **Uplink:** Verify Notion sync works with the new Jupiter `.env`.
