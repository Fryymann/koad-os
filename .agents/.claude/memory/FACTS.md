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

**F-05** — **Correct memory path:** `/home/ideans/.koad-os/.agents/.claude/memory/`
The Windows-injected system reminder path (`C:\Users\ian\.claude\projects\...`) is **wrong**.
Always write saveups and memory files to the WSL path above.
Source: saveup_2026-03-15_180000. Verified tracked by git: 2026-03-15 (gitignore fix).

**F-06** — **`.agents/.claude/` is tracked in git on `nightly`.**
Previously ignored by the unscoped `.claude/` rule in `.gitignore`. Fixed 2026-03-15:
changed to `/.claude/` (root-only) and added `.agents/.claude/worktrees/` ignore rule.
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

---

## Session Status (as of 2026-03-15 EOD)

- nightly: `32eceb1` (post-merge of #178 + gitignore/memory fix `83f92c0`)
- All worktrees decommissioned: `crazy-mcnulty`, `agitated-swartz` — cleaned
- Current phase: Phase 4 — Dynamic Tool Loading & Code Execution Sandbox
- Ready for next issue assignment
