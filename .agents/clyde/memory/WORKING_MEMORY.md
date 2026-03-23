# Clyde — Working Memory

*This file is the active session context. Updated during sessions, distilled at close.*

## Current Status

- **Condition:** GREEN
- **Phase:** 4 — Dynamic Tools & Containerized Sandboxes
- **Last Session:** 2026-03-22 (Session 4 — operational debt cleanup, Helm KAI, GitHub sync, koad-agent new fix)
- **XP:** 279 (Initiate, Level 1)

## What Is Stable

- **Vault rename** — all 5 agent vaults at `.agents/<name>/` (no dot prefix). All TOMLs, vault docs, Rust source updated.
- **`koad agent`** — `new/list/info/verify` fully implemented and validated. PATH A: TOML pre-exists → reads from `config.identities`, scaffolds vault only. PATH B: no TOML → requires `--role`/`--bio`.
- **`koad updates`** — `post/list/show/digest` implemented, CASS-contract design, board seeded with 6 entries.
- **All 9 agents GREEN** — tyr, sky, scribe, cid, vigil, clyde, claude, helm (new), + Dood. `koad agent verify <name>` passes for all.
- **Helm KAI** — `config/identities/helm.toml` + `.agents/helm/` vault. Officer rank, Gemini runtime, Citadel Build Engineer.
- **Identity TOMLs** — all 8 (tyr/sky/scribe/cid/vigil/clyde/claude/helm) have `runtime` field.
- **`AgentIdentityConfig.tier`** — added to koad-core with `#[serde(default = "default_agent_tier")]` (default 3). All existing TOMLs backward compatible.
- **GEMINI.md** — `.agents/.gemini/GEMINI.md` rewritten to Citadel v3 era (koad-citadel, koad-cass, koad-plugins, dark mode, RUST_CANON rules).
- **CREW.md** — Citadel v3 status, Vigil added, Noti corrected (Notion MCP remote), Helm corrected (Officer, container scope), boot command updated.
- **SYSTEM_MAP.md** — crates tree updated (11 current crates, koad-watchdog removed), Helm bay entry added.
- **BOOT_SEQUENCE.md** — boot command corrected to `eval $(KOAD_RUNTIME=<runtime> koad-agent boot <name>)`.
- **GitHub project #6** — labels created (phase/4-6, ops, jupiter), all open Phase 4 issues labeled, 11 new issues created (#197–207), metadata on #182/#193.

## Open Items

- **GitHub project board sync** — needs `gh auth refresh -s read:project` (Dood), then: `for issue in 189 190 191 192 193 194 195 196 197 198 199 200 201 202 203 204 205 206 207; do gh project item-add 6 --owner Fryymann --url "https://github.com/Fryymann/koad-os/issues/$issue"; done`
- **Issues #194, #195, #196** — incomplete template bodies, labeled `🟡 needs-refinement`. Need proper scoping before work begins.
- **AIS Phase B** — Rewrite `CITADEL.md`, `BOOT_SEQUENCE.md` (done partially), fix Scribe `GEMINI.md`, add `KOAD_CONTEXT_FILE` export, create `bodies/gemini/BOOT.md` and `bodies/codex/BOOT.md`.
- **AIS Phase C** (requires Dood + live services) — systemd units (#206), `handlers/boot.rs` graceful gRPC degradation (#204), Qdrant restore (#205), Docker Desktop WSL integration.
- **Phase 4 active work** — #181 (MCP Tool Registry), #182 (Sandbox, Helm), #183 (Hot-Plugins, research-needed), #192 (WasmPluginManager tests), #193 (PluginRegistry gRPC), #191 (RUST_CANON sweep).

## Services (Jupiter State)
- Redis: PONG
- Citadel gRPC (:50051): DARK
- CASS gRPC (:50052): DARK
- Qdrant: OFFLINE (Docker WSL integration needed — #205)
- Docker: OFFLINE
