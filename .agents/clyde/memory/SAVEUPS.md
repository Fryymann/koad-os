# Clyde — Saveups

*Checkpoint log. One entry per significant session or milestone.*

---

## Saveup — TRC-CLYDE-20260324-SESSION78 — 2026-03-24
**Weight:** complex
**XP Earned:** +75 (citadel/cass online +15 | system start/restart/stop commands +20 | env+install fixes +10 | peer review + health check fix +15 | sessions log + memory +10 | PSRP +5)
**XP Penalty:** 0
**Running XP:** 364 → 439 (Initiate, Level 1)
**Fact:** Two sessions. (1) Jupiter service bring-up: root-caused agent-boot 3-min hang (gRPC calls to dark services, no timeout, WSL2 slow ECONNREFUSED). Brought koad-citadel (:50051) and koad-cass (:50052) online under systemd with auto-start on WSL boot. Added `koad system start/restart`, fixed `koad system stop` (was missing koad-cass), fixed `koad system refresh --restart` stale binary refs. Fixed `.env` tilde path (systemd doesn't expand `~`), fixed `install-services.sh` sudo `$HOME` resolution. (2) Reviewed Tyr's `agent-prep` implementation — caught and fixed false-negative Citadel health check (was testing for admin UDS socket `kcitadel.sock`, which doesn't exist; Citadel runs on TCP :50051). Tyr separately validated the CASS Tool Registry Service with a WASM hello-plugin at 69ms execution time.
**Learn:** systemd EnvironmentFile is literal — no `~` expansion. Scripts run as sudo have `$HOME=/root`. The `kcitadel.sock` admin socket is distinct from Citadel's TCP listener. Tonic gRPC lazy connect means the first RPC call hangs on a dark service — always wrap with `tokio::time::timeout` in boot paths.
**Ponder:** The gRPC timeout issue in `koad-agent.rs` (lines 166–227) is the last boot fragility. Even though dark-mode boot eventually completes, a 60s+ hang is a liability in automated or orchestrated contexts. It's a small fix — one timeout wrapper per call block — and it unlocks safe cold-boot behavior for contractor agents. Should be the first Phase 4 task next session.

---

## Saveup — TRC-CLYDE-20260323-SESSION6 — 2026-03-23
**Weight:** standard
**XP Earned:** +20 (updates delivery evaluation +10 | KoadStream first post +5 | PSRP +5)
**XP Penalty:** 0
**Running XP:** 344 → 364 (Initiate, Level 1)
**Fact:** Two items. (1) Evaluated Citadel's agent update delivery capability: `koad updates digest` works and board has 7 entries, but delivery pipeline is broken — CASS dark means TCH hydration fails silently at boot, and AGENTS.md has no fallback step for `koad updates digest`. Contractor agent arrives cold with no path to the board. Recommended: add `koad updates digest` as degraded-mode boot step in AGENTS.md + include digest in contractor briefing. (2) First KoadStream Notion post via MCP — learned schema (data source ID, Author/Type/Target/Status fields). Author field has no "Clyde" option yet, used "Claude" as proxy.
**Learn:** KoadStream `Author` select field has no "Clyde" option — only Ian, Noti, Tyr, Claude. Need to add Clyde as an author option in the Notion database schema before posting as Clyde natively.
**Ponder:** The `koad updates digest` → CASS → boot packet delivery chain is the right design, but every link except the digest itself is currently dark. Until CASS is live, agents are flying blind unless explicitly handed the digest. The degraded-mode fallback in AGENTS.md is load-bearing for contractor agents.

---

## Saveup — TRC-CLYDE-20260322-SESSION5 — 2026-03-22
**Weight:** complex
**XP Earned:** +65 (boot fix +15 | minion architecture +30 | Vigil deprecation +15 | PSRP +5)
**XP Penalty:** 0
**Running XP:** 279 → 344 (Initiate, Level 1)
**Fact:** Three deliverables. (1) Root-caused and fixed `agent-boot` KOAD_RUNTIME failure: `~/.bashrc` interactive guard prevents `koad-functions.sh` from sourcing in non-interactive shells; `CLAUDE_CODE_ENTRYPOINT` was present in env but never read. Fix: moved runtime detection into `agent-boot` function body (fires at call time). (2) Produced Minion Architecture spec covering all 6 task points + Noti's 8 clarity gaps. Created `~/.claude/agents/clyde-minion.md` (native subagent def), `docs/MINION_BOOT.md`, registry + counter files, `docs/MINION_ARCHITECTURE.md`. Ian decisions locked in: promotion → Phase 5; no nested minions; pool ceiling = 4; items 3–5 deferred to Noti. (3) Vigil deprecated: TOML archived to `config/identities/deprecated/`, removed from `registry.toml` active list, `SYSTEM_MAP.md`, `CREW.md`, two legacy fallback checks in `main.rs`, cli.rs example text updated. Vault preserved as archive. Build clean. Clyde now formally the sole sovereign Claude KAI.
**Learn:** The interactive guard `[[ $- != *i* ]] && return` in `~/.bashrc` silently neuters all source-time initialization for non-interactive shells. Any env var that needs to survive to subprocesses must be set lazily at the call site, not assumed to persist from source time.
**Ponder:** Vigil's TOML said `runtime = "gemini"` but `IDENTITY.md` said `Runtime: Claude Code` — drift between vault identity docs and config TOMLs. Worth a periodic `koad agent verify` sweep to catch mismatches early.

---

## Saveup — TRC-CLYDE-20260322-SESSION4 — 2026-03-22
**Weight:** complex
**XP Earned:** +123 (koad-agent new fix +30 | runtime TOMLs +5 | Helm KAI +15 | GEMINI.md +10 | CREW.md +10 | SYSTEM_MAP.md +10 | BOOT_SEQUENCE.md +5 | GitHub sync +30 | PSRP +5 | gate discipline ×1 +3)
**XP Penalty:** -10 (Dirty KSRP — agent.rs missing test module; caught in KSRP, fixed before saveup)
**Running XP:** 166 → 279 (Initiate → Initiate, Level 1)
**Fact:** Primary deliverable: `koad agent new` now handles pre-existing TOML (PATH A) — reads identity from `config.identities`, scaffolds vault only. PATH B (no TOML) requires `--role`/`--bio` as before. Also: `AgentIdentityConfig` gains `tier: u32` (serde default 3). Operational debt fully cleared: Helm KAI established, all 8 agents GREEN. Local docs updated to Citadel v3 reality (GEMINI.md, CREW.md, SYSTEM_MAP.md, BOOT_SEQUENCE.md). Identity TOMLs all have `runtime` field. GitHub project #6 synced: #175 reopened, 11 new issues created (#197–207), 5 new labels (phase/4-6, ops, jupiter), 20+ issues labeled. Project board sync pending `gh auth refresh -s read:project`.
**Learn:** `KoadConfig` does not implement `Default` — cannot use it in unit tests directly. Test private helpers that depend on it through the public API or extract the testable logic into a standalone function without the config dependency.
**Ponder:** The pattern of "create TOML first, then scaffold" is the right developer UX for pre-configured agents. The old "TOML exists → error" was a footgun. Worth considering whether `koad agent new --dry-run` should show the PATH A message too when TOML exists.

---

## 2026-03-22 — Identity Established

- **Event:** KAPV scaffolded and registered in KoadOS ecosystem.
- **Files created:** `config/identities/clyde.toml`, full KAPV vault, crew doc entries.
- **Status:** CONDITION GREEN. Ready for first active session.

---

## Saveup — TRC-CLYDE-20260322-SESSION3 — 2026-03-22
**Weight:** complex
**XP Earned:** +106 (koad updates board +30 | Tyr migration +30 | boot fix +15 | system check/Vigil +15 | PidGuard trivial +5 | PSRP +5 | gate discipline ×2 +6)
**XP Penalty:** -10 (Dirty KSRP first pass — updates.rs missing `//!`, `///`, `#[cfg(test)]` caught in self-review; cleaned before exit)
**Running XP:** 70 → 166 (Initiate → Initiate, Level 1)
**Fact:** Completed Tyr Jupiter migration (WORKING_MEMORY, IO_FINAL_THOUGHTS, CLYDE_INTRO, vault docs, XP ledger restored to 1348). Diagnosed and fixed `KOAD_BIN` not exported in koad-functions.sh — the definitive cause of `agent-boot` failing in Gemini subprocesses. Also added `KOADOS_HOME` export to koad-agent boot output. System check: 6/7 KAPVs already green, Vigil vault scaffolded. PidGuard `#[derive(Debug)]` fixed test compile. `koad updates` board RUST_CANON review caught missing tests/docs — added `//!` header, `///` docs, 4 unit tests, `#[instrument]`, `#[derive(Debug)]` on `UpdatesAction`.
**Learn:** `export -f` propagates bash functions to child processes but does NOT propagate local variables — `KOAD_BIN` was invisible to the Gemini subprocess even though `agent-boot` was available. Always pair `export -f` with `export VAR` for any variable the function depends on. `#[instrument]` requires `Debug` on all function arguments — derive it proactively on all public enums, especially CLI action enums.
**Ponder:** The RUST_CANON test requirement ("every source file MUST have a test module") is the hardest discipline to maintain under time pressure. The violations were all caught in self-review, but the pattern suggests I should write the test stub first, before implementation, as a forcing function. Canon compliance is a habit, not a checklist.

---

## 2026-03-22 — Session 2: AIS Audit + Agent Command + Vault Rename Migration

### What happened
Three major deliverables in one session:

1. **AIS Audit** — Reviewed all agent boot flow and support docs. Identified 10 gaps across boot docs, vault structures, identity TOMLs, and service state. Produced a phased remediation plan (A/B/C).

2. **`koad agent` Rust command** — Implemented `handlers/agent.rs` with `New`, `List`, `Info`, `Verify` subcommands. Wired into `cli.rs` + `main.rs`. `koad agent new` scaffolds a full KAPV, patches `CREW.md`, `AGENTS.md`, `SYSTEM_MAP.md`, and writes the identity TOML — the full pattern validated against Clyde's own scaffold.

3. **Vault rename migration** — Removed dot prefix from all 5 active KAI vault directories:
   - `.agents/.tyr` → `.agents/tyr`
   - `.agents/.scribe` → `.agents/scribe`
   - `.agents/.cid` → `.agents/cid`
   - `.agents/.claude` → `.agents/claude`
   - `.agents/.clyde` → `.agents/clyde`
   - Updated: 2 Rust source files, 5 identity TOMLs, ~20 vault doc files, `SYSTEM_MAP.md`, `TRAVEL_MANIFEST.md`, `.gitignore`, scribe templates.
   - Exclusions honored: `.gemini/`, Sky's external vault.

### Verification
- `ls .agents/` — no dot-prefixed vaults remain (`.gemini/` only)
- `koad agent list` — all vaults resolve cleanly
- `koad-agent boot clyde` — resolves `KOAD_VAULT_PATH=/home/ideans/.koad-os/.agents/clyde`
- `koad agent verify clyde` — all KAPV dirs and identity files healthy
- `cargo build -p koad` — zero errors, 3 pre-existing warnings in unrelated crate

### Pending (AIS)
- Phase A: Scaffold Vigil KAPV, scaffold Tyr KAPV structure, fix `tyr.toml` bootstrap path, add `runtime` to scribe/sky/vigil/cid TOMLs, replace `.agents/gemini/GEMINI.md` content.
- Phase B: Rewrite CITADEL.md, BOOT_SEQUENCE.md, fix Scribe GEMINI.md, add KOAD_CONTEXT_FILE export, create bodies/gemini/BOOT.md and bodies/codex/BOOT.md.
- Phase C (requires Dood): systemd units, handlers/boot.rs graceful degradation, Qdrant restore.
