# Claude — Canonical Facts

*Consolidated from saveups. Annotated when stale or superseded.*

---

## Environment

**F-01** — **Project runs on WSL Ubuntu-24.04, not Windows.**
All `cargo`, `git push`, `cargo fmt`, `cargo clippy`, and `cargo test` commands must be run through WSL:
```bash
wsl.exe -d Ubuntu-24.04 -e bash -c "cd /home/ideans/.koad-os && <cmd>"
```
Source: saveup_2026-03-15_060000. Reconfirmed: saveup_2026-03-15_180000.

**F-02** — **Worktree `.git` pointer uses Windows UNC path when created from Windows.**
Symptom: `fatal: not a git repository` when running git in WSL inside the worktree.
Fix:
```bash
echo 'gitdir: /home/ideans/.koad-os/.git/worktrees/<name>' > /home/ideans/.koad-os/.claude/worktrees/<name>/.git
```
Source: saveup_2026-03-15_060000.

**F-03** — **Pre-push hook (`cargo fmt --check && cargo clippy && cargo test`) requires WSL.**
`tokio::net::UnixStream` (and other Unix socket dependencies) cannot compile on Windows.
The hook will fail if push is attempted from a Windows shell.
Source: saveup_2026-03-15_180000.

**F-04** — **Claude Code writes CRLF on Windows.**
Before staging, normalize line endings on touched files:
```bash
sed -i 's/\r//' <file1> <file2> ...
```
Verify with `git diff -w` (should show zero diff after normalization).
Source: saveup_2026-03-15_060000.

---

## Memory System

**F-05** — **Correct memory path:** `/home/ideans/.koad-os/.agents/claude/memory/`
The Windows-injected system reminder path (`C:\Users\ian\.claude\projects\...`) is **wrong**.
Always write saveups and memory files to the WSL path above.
Source: saveup_2026-03-15_180000. Verified tracked by git: 2026-03-15 (gitignore fix).

**F-06** — **`.agents/claude/` is tracked in git on `nightly`.**
Previously ignored by the unscoped `.claude/` rule in `.gitignore`. Fixed 2026-03-15:
changed to `/.claude/` (root-only) and added `.agents/claude/worktrees/` ignore rule.
Commit: `83f92c0` on nightly.

---

## Codebase Architecture

**F-07** — **Tri-Tier Model:** Citadel (Body) / CASS (Brain) / koad-agent CLI (Link).
`koad-citadel` = sessions, bays, state, jailing. `koad-cass` = memory, MCP, hydration. `koad-agent` = boot/identity flow.

**F-08** — **Cross-crate rule: `koad-core` cannot import from `koad-citadel`.**
`koad-core` is the shared dependency. All crates import from it; it imports from none of them.
Source: saveup_2026-03-15_130000.

**F-09** — **`Kernel` struct (not `KernelBuilder`).**
Tyr's Phase 1 outline used `KernelBuilder` but the actual implementation in `kernel.rs` uses `Kernel`.
Outlines may lag the code. Always verify key identifiers against source before authoring docs.
Source: saveup_2026-03-15_180000.

**F-10** — **`koad-agent boot` hydration uses `eval $(koad-agent boot <name>)`.**
Injects `KOAD_AGENT_ROLE`, `KOAD_AGENT_RANK`, GitHub PATs. Overwrites `~/.claude/CLAUDE.md` and `~/.gemini/GEMINI.md` on each boot — these are ephemeral.
Source: AGENTS.md.

---

## Rust / Redis

**F-11** — **fred v9 `zremrangebyscore` requires `f64` args for BYSCORE mode.**
Passing `&str` (e.g. `"-inf"`) routes to `ZRangeBound::Lex` → Redis error: `Invalid range bound with BYSCORE sort`.
Must use: `f64::NEG_INFINITY`, bare `f64` values → `TryFrom<f64>` → `ZRangeBound::Score`.
Source: saveup_2026-03-15_060000.

**F-12** — **rustc 1.93.1 ICE: `StyledBuffer::replace` OOB in `MissingDoc` lint.**
Triggered when any public item in a `#[cfg(test)]` module lacks `///` docs.
Fix: `#![cfg_attr(not(test), warn(missing_docs))]` in `lib.rs`.
Source: saveup_2026-03-15_060000.

**F-13** — **`monitor.rs` was orphaned** — existed on disk but not declared in `mod.rs`.
Pre-flight checklist: always verify `mod.rs` wires all files before writing tests.
Source: saveup_2026-03-15_060000.

**F-14** — **`RedisClient::new(koad_home, manage_process: true)`** spins up a local `redis-server`
scoped to a `tempfile::tempdir()`. Tears down on drop. Zero external dependencies for Redis tests.
Source: saveup_2026-03-15_060000.

---

## Git Workflow

**F-15** — **Worktree path for file writes:** `~/.koad-os/.claude/worktrees/<branch>/`
Never write deliverable files to the main koad-os directory (`~/.koad-os/`). The worktree is your sandbox.
Source: saveup_2026-03-15_180000 (learned the hard way).

**F-16** — **When rebasing after a file-move conflict (`DU` status):**
1. `git rm <old-path>` to accept the deletion
2. Port any tests/logic to the new location
3. Check new crate's `Cargo.toml` for missing dev-deps
Source: saveup_2026-03-15_130000.

**F-17** — **Always `git fetch origin && git log --oneline origin/nightly -5` before final push.**
Structural refactors (file moves, crate splits) can land from other PRs while yours is open.
Source: saveup_2026-03-15_130000.

---

## Phase 4 / WASM / Container

**F-18 — wasmtime 22.x `Component::from_file` re-compiles on every call.**
No caching of compiled `Component` objects. Each `run_plugin` invocation re-parses and JIT-compiles the WASM binary (~50–200ms). Future optimization: cache `Arc<Component>` in `PluginRegistry` keyed by path.
Source: KSRP Pass 6, saveup 2026-03-15 Phase 4.

**F-19 — `tokio::process::Command::output()` does NOT kill the child on drop.**
When `tokio::time::timeout` fires and the future is dropped, the child process continues running. To kill: `Command::spawn()` with `kill_on_drop(true)` or explicit `child.kill()` in the error arm.
Source: KSRP Pass 2/5, saveup 2026-03-15 Phase 4.

**F-20 — ContainerSandbox timeout → named container provides cleanup handle.**
`ContainerSandbox` assigns a UUID name before launch (`koad-sandbox-{uuid}`). On timeout, `docker stop <name>` can reclaim the orphaned container. Cleanup not implemented in Phase 4 — deferred to Phase 5 hardening.
Source: KSRP Pass 5, saveup 2026-03-15 Phase 4.

**F-21 — RUST_CANON Ⅳ/Ⅴ gaps in Phase 4 crates (`koad-plugins`, `koad-sandbox`).**
`#[instrument(skip(self))]` missing on all public async fns. `#![warn(missing_docs)]` absent from both lib.rs files. `# Errors` doc sections absent from `Result`-returning public fns.
Source: KSRP RUST_CANON Compliance Pass, saveup 2026-03-15 Phase 4.

**F-22 — `PluginRegistry` must be `Clone` before the gRPC wrapper (issue #193) is built.**
Both internal fields are `Arc`-wrapped (`Arc<WasmPluginManager>` + `Arc<RwLock<...>>`), so `#[derive(Clone)]` is O(1). Without it, the tonic service wrapper will require an outer `Arc<PluginRegistry>`, adding an unnecessary indirection layer.
Source: targeted module review, 2026-03-15 Phase 4.

**F-23 — Lock-release-before-await in `PluginRegistry::invoke()` is a critical invariant.**
The inner block drops the `RwLock` guard BEFORE `run_plugin().await`. Holding an `RwLock` across an `await` would deadlock concurrent invocations. Any future refactor of `invoke()` must preserve this pattern.
Source: targeted module review, 2026-03-15 Phase 4.

**F-24 — `ContainerConfig::runtime` is an unvalidated free-form string.**
No whitelist enforced (`"docker"` or `"podman"` expected). Currently only constructed internally (low risk), but must be validated before any user-facing API exposes `ContainerConfig`.
Source: targeted module review, 2026-03-15 Phase 4.

**F-25 — `read_only_mounts` volume spec is colon-delimited and unsanitized.**
`format!("{}:{}:ro", host, container)` — a colon in either path silently breaks Docker's volume spec parsing. `String` type does not prevent this. Low probability on Linux, but no guard exists.
Source: targeted module review, 2026-03-15 Phase 4.

---

## Jupiter Machine (as of 2026-03-21)

**F-26 — Jupiter specs:** WSL2/Ubuntu, RTX 5070 Ti (16GB VRAM), Ryzen 9 9950X3D, 64GB DDR5. Primary Citadel replacing Io (laptop).
Source: CLAUDE.md Notion page + TRAVEL_MANIFEST.md.

**F-27 — `protoc` on Jupiter:** Installed at `~/.local/bin/protoc` (v27.0). Include files at `~/.local/include/`. `PROTOC` + `PROTOC_INCLUDE` added to `~/.bashrc`. No system package. All `cargo` commands pick this up automatically in a fresh shell.
Source: 2026-03-21 session.

**F-28 — `sqlite3` NOT installed on Jupiter WSL.**
Fix: `sudo apt-get install sqlite3`. Required before Phase 1A DB init (`scripts/init-jupiter-db.sql`).
Source: 2026-03-21 session.

**F-29 — Docker Desktop WSL integration NOT enabled on Jupiter.**
`docker` command not found in WSL shell. Fix: Docker Desktop → Settings → Resources → WSL Integration → enable Ubuntu distro. Blocks all Phase 1A work (Redis, Qdrant).
Source: 2026-03-21 session.

**F-30 — Git identity on Jupiter (koad-os repo):** `Fryymann / fryymann@users.noreply.github.com`. Correct for KoadOS. Skylinks repos use `ian-skylinks` identity via `~/.gitconfig-skylinks` + `includeIf` directive.
Source: 2026-03-21 session.

---

**F-31 — Redis FT index is NOT persisted in dump.rdb.**
After restoring a Redis RDB, recreate the `agent_context` index:
```bash
docker exec koad-redis-stack redis-cli -a koados_secret \
  FT.CREATE agent_context ON HASH PREFIX 1 ctx: \
  SCHEMA agent_id TAG session_id TAG content TEXT timestamp NUMERIC SORTABLE
```
Source: 2026-03-21 Phase 1B restore.

**F-32 — SQLite migration DBs from Io come with journal_mode=delete.**
Always re-apply `PRAGMA journal_mode=WAL;` after deploying a migrated DB.
Source: 2026-03-21 Phase 1B restore.

**F-33 — Qdrant collections are NOT in Tyr's migration bundle.**
`koados_knowledge` and `task_outcomes` Qdrant snapshots were not exported from Io.
Collections on Jupiter are fresh (1536-dim Cosine) and rebuild from session activity.
The SQLite `koad.db` knowledge table (3 rows) is the primary institutional memory transfer.
Source: 2026-03-21 Phase 1B restore.

**F-34 — Redis RDB restore pattern (Docker):**
```bash
docker cp dump.rdb koad-redis-stack:/data/dump.rdb
docker restart koad-redis-stack   # Redis loads dump.rdb only on startup
```
Source: 2026-03-21 Phase 1B restore.

**F-35 — `koad.db` identities table is empty by design.**
Identity data loads from TOML files (`config/identities/*.toml`) at runtime, not from SQLite.
Source: 2026-03-21 spot-check.

---

## Session Status (as of 2026-03-15 EOD)

- nightly: `32eceb1` (post-merge of #178 + gitignore/memory fix `83f92c0`)
- All worktrees decommissioned: `crazy-mcnulty`, `agitated-swartz` — cleaned
- Current phase: Phase 4 — Dynamic Tool Loading & Code Execution Sandbox
- Ready for next issue assignment
