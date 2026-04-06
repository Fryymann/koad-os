# Clyde — Working Memory

*This file is the active session context. Updated during sessions, distilled at close.*

## Current Status

- **Condition:** GREEN
- **Phase:** 4+5 / 7 — Citadel Pulse, Phase 4 Cleanup, koad-agent MVP delivered
- **Last Session:** 2026-04-05 (Session 14 — Three Operations COMPLETE)
- **XP:** 712 (Initiate, Level 1 → Level 2 threshold approaching)

## Session 14 — 2026-04-05

### Operation Citadel Pulse (Phase 7.5) — COMPLETE
- **CP-01** — `PulseService` gRPC (proto + CASS impl). `PulseTier` trait on `RedisTier` with TTL keys `koad:pulse:{id}`, sets `koad:pulse:role:{role}`. `CassPulseService` registered in `main.rs`. Qdrant degraded-mode fix (bonus): `QdrantTier::new_offline()`, `Option<Qdrant>`, 3s `tokio::time::timeout` on boot — CASS no longer hangs when Qdrant is offline.
- **CP-02** — `koad pulse` CLI (`--list`, message positional). `updates post` now fires best-effort `add_pulse` after posting. `Sandbox` command added to CLI.
- **CP-03** — `sync-status.sh` docs sync script. Smoke test + full `koad-cass` test coverage added (MockPulseStore, tiered fallback, all passing).

### Operation Phase 4 Cleanup — COMPLETE
- **CP-07** — `SandboxRunner` trait in `koad-sandbox`. Container execution: `docker`/`podman` subprocess with `--rm --read-only --tmpfs /tmp --network none --security-opt no-new-privileges`. `register-tool.rs` updated with `container_image` field.
- **CP-08** — `PluginRegistry` finalized: `register_with_permissions()`, `get_permissions()`, `get_container_image()`, `register_with_opts()` with container routing in `invoke()`. `start_hot_reload()` background task (5s mtime poll). `NativePluginManager` for `.so`/`.dll` via `libloading`. All tests pass including `test_register_with_permissions_preserves_container_image`.

### Operation koad-agent MVP (Phase 5) — COMPLETE
- **CP-04** — `koad-agent context` command: reads crate doc comments, indexes via `koad-codegraph`, pulls git log, outputs `.context.md` packet for TCH injection.
- **CP-05** — `KOAD_RUNTIME` export added to `koad-agent boot` handler. Reads `identity_config.runtime`, exports alongside `KOAD_RANK` etc.
- **CP-06** — `koad-agent task` command: loads/creates `~/.koad-os/run/tasks.json`, validates manifest existence, fuzzy role matching, worktree collision detection, registers READY state. `trim_matches` fix for paths with trailing `:`, `,`, `.`.

### Bugs Fixed
- Pre-existing `test_tiered_write_and_read` failure (required live Qdrant) — fixed by `new_offline()` fallback in tiered test.
- Orchestration handoff miss on CP-01 (no active monitoring at agent boundary) — corrected protocol: poll output files at dependency completion before dispatching next wave. Saved as feedback memory.

## Session 13 — 2026-04-03

- **Phase 7 — CASS Tiered Memory Stack (L1-L4)** — COMPLETE.
- **Implementation** — Refactored `CassStorage` into `SqliteTier`, implemented `RedisTier` (hot cache) and `QdrantTier` (semantic index).
- **Orchestration** — `TieredStorage` now manages the multi-tier sync (L1/L2 sync, L3 fire-and-forget).
- **Verification** — All tests pass, live Qdrant write confirmed, ACR clean.

## Session 12 — 2026-04-03


- **Phase 7 Tiered Memory Stack** — `tasks/phase_7_memory_stack/mission_brief.md` COMPLETE. Implemented `MemoryTier` trait, `SqliteTier` (L2), `RedisTier` (L1, fred), `QdrantTier` (L3, qdrant-client 1.17, 32-dim cosine), `TieredStorage` orchestrator. Updated `main.rs` to boot tiered stack. 3/3 tests pass. L3 Qdrant write confirmed live. ACR clean.
- **Qdrant gRPC port** — `:6334` (gRPC), `:6333` (REST). qdrant-client uses gRPC.
- **`CassStorage` type alias** — `pub type CassStorage = SqliteTier` retained in `storage/mod.rs` for backward compat.

## Session 12 — 2026-04-03

- **Phase 4 Skill Integration** — `tasks/phase_4_skill_integration/mission_brief.md` COMPLETE (all 3 phases). `koad bridge skill register/list/run/deregister` wired to CASS `ToolRegistryService`. E2E verified live. ACR clean. Binary installed.
- **hello-plugin WASM** — Built and validated: `crates/koad-plugins/examples/hello-plugin/target/wasm32-unknown-unknown/release/hello_plugin.component.wasm` (23K).

## Session 11 — 2026-04-03

- **Handoff from Tyr** — New SDD plan prepped for Phase 4 Tool Registry & CLI Integration.
- **Team Assignment** — Assigned `clyde-dev` and `clyde-qa` to connect the `koad bridge skill` subcommands to CASS.

## Session 10 — 2026-04-03

- **Handoff from Tyr** — Transitioned to Spec Driven Development (SDD). AIS Phase C SDD plan approved.
- **Phase 1 Complete** — Tyr implemented initial gRPC degradation logic in `boot.rs`.
- **Team Assignment** — Assigned `clyde-dev` and `clyde-qa` to stabilize the boot sequence and infrastructure dependencies.
- **SUCCESS** — AIS Phase C complete.

## Session 7 — 2026-03-24

- **Citadel + CASS brought online** — both running under systemd, enabled for WSL auto-start.
- **`koad system start/restart`** — new commands added to koad CLI. systemctl-first, direct-spawn fallback.
- **`koad system stop`** — fixed to also kill koad-cass (was missing).
- **`koad system refresh --restart`** — fixed stale binary refs (kcitadel → koad-citadel, koad-watchdog removed, koad-cass added).
- **`config/systemd/koad-citadel.service`** — added `Wants=koad-cass.service` for cascade start.
- **`.env` fix** — `KOADOS_HOME=~/.koad-os` → absolute path (systemd doesn't expand `~`).
- **`scripts/install-services.sh`** — fixed sudo `$HOME` vs `$SUDO_USER` resolution.
- **`agent-boot` timing** — ~1s (was 3+ min; hang was gRPC calls to dark services with no timeout).

## What Is Stable

- **Vault rename** — all 5 agent vaults at `agents/<name>/` (no dot prefix). All TOMLs, vault docs, Rust source updated.
- **`koad agent`** — `new/list/info/verify` fully implemented and validated. PATH A: TOML pre-exists → reads from `config.identities`, scaffolds vault only. PATH B: no TOML → requires `--role`/`--bio`.
- **`koad updates`** — `post/list/show/digest` implemented, CASS-contract design, board has 7 entries. KoadStream Notion DB confirmed (data source: `310fe8ec-ae8f-8046-9172-000bfe5966cd`).
- **All 8 agents GREEN** — tyr, sky, scribe, cid, clyde, claude, helm, + Dood. Vigil deprecated. `koad agent verify <name>` passes for all active agents.
- **Helm KAI** — `config/identities/helm.toml` + `agents/KAPVs/helm/` vault. Officer rank, Gemini runtime, Citadel Build Engineer.
- **Identity TOMLs** — all 7 active (tyr/sky/scribe/cid/clyde/claude/helm) have `runtime` field. Vigil TOML archived to `config/identities/deprecated/`.
- **agent-boot fix** — `koad-functions.sh` runtime detection moved into `agent-boot` function body. Fires at call time in non-interactive shells. `KOAD_RUNTIME=claude` auto-set via `CLAUDE_CODE_ENTRYPOINT`.
- **Minion Architecture** — `~/.claude/agents/KAPVs/clyde-minion.md` live. `MINION_ARCHITECTURE.md` approved (partial). Pool ceiling 4, no nested minions, promotion → Phase 5. Items 3–5 deferred to Noti.
- **Clyde = sole sovereign Claude KAI** — Vigil deprecated. Formally established this session.
- **`AgentIdentityConfig.tier`** — added to koad-core with `#[serde(default = "default_agent_tier")]` (default 3). All existing TOMLs backward compatible.
- **GEMINI.md** — `agents/.gemini/GEMINI.md` rewritten to Citadel v3 era (koad-citadel, koad-cass, koad-plugins, dark mode, RUST_CANON rules).
- **CREW.md** — Citadel v3 status, Vigil added, Noti corrected (Notion MCP remote), Helm corrected (Officer, container scope), boot command updated.
- **SYSTEM_MAP.md** — crates tree updated (11 current crates, koad-watchdog removed), Helm bay entry added.
- **BOOT_SEQUENCE.md** — boot command corrected to `eval $(KOAD_RUNTIME=<runtime> koad-agent boot <name>)`.
- **GitHub project #6** — labels created (phase/4-6, ops, jupiter), all open Phase 4 issues labeled, 11 new issues created (#197–207), metadata on #182/#193.

## Open Items

- **AGENTS.md degraded-mode boot step** — add `koad updates digest` as a fallback step when CASS is offline, so contractor/cold agents can reach the board. Blocker for contractor Claude agent awareness.
- **KoadStream Author: "Clyde"** — ✅ RESOLVED. Tyr added "Clyde" to the Author select field in the Notion KoadStream schema (2026-04-03). Posts now authored as Clyde directly.
- **Contractor Claude agent briefing** — needs `koad updates digest` output + current crew state before starting Notion sync work.
- **Minion Architecture open items (deferred to Noti)** — Desktop env detection, counter race condition, report retention policy.
- **GitHub project board sync** — needs `gh auth refresh -s read:project` (Dood), then: `for issue in 189 190 191 192 193 194 195 196 197 198 199 200 201 202 203 204 205 206 207; do gh project item-add 6 --owner Fryymann --url "https://github.com/Fryymann/koad-os/issues/$issue"; done`
- **Issues #194, #195, #196** — incomplete template bodies, labeled `🟡 needs-refinement`. Need proper scoping before work begins.
- **AIS Phase B** — Rewrite `CITADEL.md`, `BOOT_SEQUENCE.md` (done partially), fix Scribe `GEMINI.md`, add `KOAD_CONTEXT_FILE` export, create `bodies/gemini/BOOT.md` and `bodies/codex/BOOT.md`.
- **AIS Phase C** (requires Dood + live services) — systemd units (#206), `handlers/boot.rs` graceful gRPC degradation (#204), Qdrant restore (#205), Docker Desktop WSL integration.
- **Phase 4 active work** — #181 (MCP Tool Registry), #182 (Sandbox, Helm), #183 (Hot-Plugins, research-needed), #192 (WasmPluginManager tests), #193 (PluginRegistry gRPC), #191 (RUST_CANON sweep).

## Services (Jupiter State)
- Redis: PONG (unix socket: ~/.koad-os/koad.sock)
- Citadel gRPC (:50051): ACTIVE (systemd managed, enabled for WSL boot)
- CASS gRPC (:50052): ACTIVE (systemd managed, enabled for WSL boot)
- Qdrant: OFFLINE (Docker WSL integration needed — #205)
- Docker: OFFLINE
