## [2026-03-21] — Jupiter Migration Session 2: Config Verification & Env Namespace Alignment

- **Fact:** KoadOS Rust codebase builds clean on Jupiter. `koad-core`, `koad`, `koad-agent` pass `cargo check`. 3 signal tests fail as expected (Redis not running — Docker blocked). Requires `PROTOC=/home/ideans/.local/bin/protoc PROTOC_INCLUDE=/home/ideans/.local/include` (now in `~/.bashrc`).
- **Fact:** Fixed 5 env var namespace mismatches for KOADOS_ migration. `config.rs:247` now reads `KOADOS_HOME` with `KOAD_HOME` legacy fallback. `bridge.rs` tries `KOADOS_PAT_NOTION_MAIN` first for Notion token. `koad-agent.rs` exports `GITHUB_OWNER` from `KOADOS_MAIN_GITHUB_USER` (was hardcoded `"Fryymann"`). `utils.rs` PAT lookups use `KOADOS_PAT_GITHUB_ADMIN` / `KOADOS_MAIN_GITHUB_PAT`. Committed `7e067c6` on nightly.
- **Fact:** `protoc` v27.0 installed to `~/.local/bin/protoc`. Include files at `~/.local/include/`. Installed via Python zipfile extraction from GitHub release (no apt/sudo needed). `PROTOC` + `PROTOC_INCLUDE` added to `~/.bashrc`.
- **Fact:** `sqlite3` NOT installed in WSL Ubuntu on Jupiter. `sqlite3 --version` → not found. Fix: `sudo apt-get install sqlite3`.
- **Fact:** Docker Desktop WSL integration NOT enabled on Jupiter. `docker` command not found in WSL shell. Fix: Docker Desktop → Settings → Resources → WSL Integration → enable Ubuntu distro.
- **Fact:** Git identity on Jupiter in koad-os repo is `Fryymann / fryymann@users.noreply.github.com`. Correct for KoadOS work.
- **Learn:** Background `sudo apt-get install` commands can hang waiting for a password prompt — avoid running sudo commands in background. Use `python3 -c "import zipfile..."` as an alternative for extracting archives without `unzip`.
- **Ponder:** Phase 1A (entire Docker stack) is blocked by a single toggle in Docker Desktop. Once Docker WSL integration is on, Phase 1A can proceed rapidly. The sqlite3 install is a one-liner. Neither blocker requires deep work.

---

## [2026-03-15 EOD] — Knowledge Consolidation Pass

**Consolidation health:** GOOD
**Entries reviewed:** 3 saveups (060000, 130000, 180000), context.md, log.md
**Changes made:**
- Created `FACTS.md` — 17 canonical facts extracted from all saveups, deduplicated
- Created `LEARNINGS.md` — 13 operational lessons, categorized by domain
- Created `PONDERS.md` — first-person reflection on session deviations and open tensions
- Created `WORKING_MEMORY.md` — current state snapshot
- **Staleness:** No entries marked stale. All saveup facts remain accurate as of EOD.
- **Structural gap identified:** No boot ritual was documented anywhere before this session. Now captured in LEARNINGS.md (L-01 to L-03) and WORKING_MEMORY.md.
- **Worktree cleanup:** `agitated-swartz` pruned. `claude/agitated-swartz` branch deleted. Main worktree clean on nightly `32eceb1`.

---

## [2026-03-15] — Issue #173: Phase 4 — WASM Plugin Host, PluginRegistry, ContainerSandbox — PR #185

- **Fact:** Fixed 3 wasmtime 22.x `bindgen!` bugs in `koad-plugins` (wrong trait name `Host` → `CitadelHooksImports`, wrong linker fn, wrong async impl pattern). Added `hello-plugin` WASM guest built for `wasm32-unknown-unknown` (no WASI adapter). Added `PluginRegistry` (thread-safe in-memory, `tokio::sync::RwLock`). Added `ContainerSandbox` via `docker run`/`podman run` subprocess (avoids bollard rustc 1.93.1 ICE). Extended `TurnMetrics` proto with `execution_duration_ms` (field 5) and `execution_memory_bytes` (field 6). Fixed pre-existing sandbox test regression (stale mock KoadConfig JSON missing `system`/`network`/`storage` fields). All CI green: fmt ✓ clippy ✓ test 14/14 ✓. PR #185 open.
- **Learn:** Tokio `Command::output()` does NOT kill the child on drop — `tokio::time::timeout` firing causes a container orphan. Fix: `kill_on_drop(true)` or explicit cleanup. `Component::from_file` in wasmtime re-JIT-compiles on every call — cache `Arc<Component>` for hot paths. `wasm32-unknown-unknown` + `wasm-tools component new` is the correct guest target when the guest has no WASI deps. `cargo expand` is the definitive oracle for wasmtime `bindgen!` generated API names.
- **Ponder:** The gRPC wrapper for `PluginRegistry` is deferred to the next phase. The in-memory registry is ready, but the Phase 4 "via gRPC" criterion is technically half-satisfied. RUST_CANON Ⅳ (`#[instrument]`) and Ⅴ (`#![warn(missing_docs)]`, `# Errors`) gaps remain in the new Phase 4 crates — a canon compliance sweep issue should be filed before Phase 5 hardens the API surface.

---

## [2026-03-15] — Issue #163: Diagnostic Harness (Signal Corps/Streams Testing) — MERGED PR #170
- **Fact:** Completed 8-test harness for Signal Corps async messaging layer. Fixed silent production bug in `quota.rs` (fred v9 ZRange API). Wired orphaned `monitor.rs`. Resolved merge conflict after Phase 2 moved `SignalCorps` to `koad-core` — ported 3 stream tests to `koad-core/src/signal.rs` and used `xlen` directly instead of `StreamMonitor`. PR #170 merged into nightly.
- **Learn:** When rebasing after a file-move conflict (`DU` status): `git rm <old>`, port tests to new crate, verify dev-deps in new `Cargo.toml`. Cross-crate rule: `koad-core` cannot import from `koad-citadel`. Fetch origin before final push to anticipate structural refactors landing from other PRs.
- **Ponder:** The pre-existing clippy failures in `kernel.rs`, `sanctuary.rs`, `hierarchy.rs`, `bay.rs`, `session.rs` represent undocumented public API surface. A bulk docs pass on `koad-citadel` would clean up these warnings and improve onboarding for future contractors.
