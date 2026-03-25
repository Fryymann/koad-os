# Clyde — Working Memory

*This file is the active session context. Updated during sessions, distilled at close.*

## Current Status

- **Condition:** GREEN
- **Phase:** 4 — Dynamic Tools & Containerized Sandboxes
- **Last Session:** 2026-03-24 (Sessions 7 & 8 — Jupiter service bring-up, peer review)
- **XP:** 439 (Initiate, Level 1)

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

- **Vault rename** — all 5 agent vaults at `.agents/<name>/` (no dot prefix). All TOMLs, vault docs, Rust source updated.
- **`koad agent`** — `new/list/info/verify` fully implemented and validated. PATH A: TOML pre-exists → reads from `config.identities`, scaffolds vault only. PATH B: no TOML → requires `--role`/`--bio`.
- **`koad updates`** — `post/list/show/digest` implemented, CASS-contract design, board has 7 entries. KoadStream Notion DB confirmed (data source: `310fe8ec-ae8f-8046-9172-000bfe5966cd`).
- **All 8 agents GREEN** — tyr, sky, scribe, cid, clyde, claude, helm, + Dood. Vigil deprecated. `koad agent verify <name>` passes for all active agents.
- **Helm KAI** — `config/identities/helm.toml` + `.agents/helm/` vault. Officer rank, Gemini runtime, Citadel Build Engineer.
- **Identity TOMLs** — all 7 active (tyr/sky/scribe/cid/clyde/claude/helm) have `runtime` field. Vigil TOML archived to `config/identities/deprecated/`.
- **agent-boot fix** — `koad-functions.sh` runtime detection moved into `agent-boot` function body. Fires at call time in non-interactive shells. `KOAD_RUNTIME=claude` auto-set via `CLAUDE_CODE_ENTRYPOINT`.
- **Minion Architecture** — `~/.claude/agents/clyde-minion.md` live. `MINION_ARCHITECTURE.md` approved (partial). Pool ceiling 4, no nested minions, promotion → Phase 5. Items 3–5 deferred to Noti.
- **Clyde = sole sovereign Claude KAI** — Vigil deprecated. Formally established this session.
- **`AgentIdentityConfig.tier`** — added to koad-core with `#[serde(default = "default_agent_tier")]` (default 3). All existing TOMLs backward compatible.
- **GEMINI.md** — `.agents/.gemini/GEMINI.md` rewritten to Citadel v3 era (koad-citadel, koad-cass, koad-plugins, dark mode, RUST_CANON rules).
- **CREW.md** — Citadel v3 status, Vigil added, Noti corrected (Notion MCP remote), Helm corrected (Officer, container scope), boot command updated.
- **SYSTEM_MAP.md** — crates tree updated (11 current crates, koad-watchdog removed), Helm bay entry added.
- **BOOT_SEQUENCE.md** — boot command corrected to `eval $(KOAD_RUNTIME=<runtime> koad-agent boot <name>)`.
- **GitHub project #6** — labels created (phase/4-6, ops, jupiter), all open Phase 4 issues labeled, 11 new issues created (#197–207), metadata on #182/#193.

## Open Items

- **AGENTS.md degraded-mode boot step** — add `koad updates digest` as a fallback step when CASS is offline, so contractor/cold agents can reach the board. Blocker for contractor Claude agent awareness.
- **KoadStream Author: "Clyde"** — Notion KoadStream schema needs "Clyde" added to the Author select field. Currently posting as "Claude" as proxy.
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
