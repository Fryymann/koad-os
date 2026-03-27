# Claude — Learnings (Operational Lessons)

*High-signal lessons extracted from sessions. Ordered by category, not time.*

---

## Session Boot Ritual (CRITICAL)

**L-01** — **Read `SAVEUPS.md` at session start. Every time. No exceptions.**
I re-hit the `.git` UNC path bug in session 2 that was already documented in session 1.
The saveup existed and I didn't read it. That's pure wasted motion.
Ritual: `wsl.exe -d Ubuntu-24.04 -e bash -c "cat /home/ideans/.koad-os/agents/.claude/memory/SAVEUPS.md"`

**L-02** — **Verify working directory immediately after session boot.**
Run `git status` and confirm you are in the correct worktree branch, not on `nightly`.
If `git status` shows `On branch nightly` — stop, create or enter your worktree first.

**L-03** — **Check memory path explicitly at boot.**
```bash
wsl.exe -d Ubuntu-24.04 -e bash -c "ls /home/ideans/.koad-os/agents/.claude/memory/"
```
The Windows-injected path in the system reminder is wrong. Use the WSL path above.

---

## File Safety

**L-04** — **All deliverable files go to the worktree, never the main repo.**
The main koad-os dir (`/home/ideans/.koad-os/`) is `nightly`. Writing there pollutes the trunk.
Worktree path: `/home/ideans/.koad-os/.claude/worktrees/<branch>/`

**L-05** — **Before `git add` on a previously-ignored directory: scan its contents first.**
When a directory was gitignored and you're adding it for the first time, do `git status <path>` to see exactly what's in there before staging. Old worktree snapshots, config files, and cache dirs can silently appear.

**L-06** — **Normalize CRLF before staging.**
Claude Code on Windows writes CRLF. Normalize touched files before `git add`:
```bash
sed -i 's/\r//' <file1> <file2>
```
Verify: `git diff -w` should show no whitespace-only changes.

---

## Authoring & Documentation

**L-07** — **Outlines are specs, not ground truth. Verify code identifiers before authoring.**
Tyr's Phase 1 outlines used `KernelBuilder` but the actual struct was `Kernel`. If authoring docs that reference code, spot-check key struct/function names against source:
```bash
wsl.exe -d Ubuntu-24.04 -e bash -c "grep -n 'pub struct\|pub fn' /home/ideans/.koad-os/crates/<crate>/src/<file>.rs"
```

---

## Git Operations

**L-08** — **Fix `.git` pointer before any WSL git command in a worktree.**
```bash
echo 'gitdir: /home/ideans/.koad-os/.git/worktrees/<name>' > /home/ideans/.koad-os/.claude/worktrees/<name>/.git
```

**L-09** — **Fetch before final push to catch concurrent structural refactors.**
```bash
git fetch origin && git log --oneline origin/nightly -5
```
If file moves or crate splits show up, rebase before pushing.

**L-10** — **When rebasing after a `DU` (delete/unresolved) conflict:**
1. `git rm <old-path>` — accept the upstream deletion
2. Port any logic/tests to the new file location (check the new crate's `Cargo.toml` for missing dev-deps)
3. `git add` the new files, `git rebase --continue`

---

## Rust / Redis

**L-11** — **Pre-flight checklist before writing tests for a module:**
1. Check `mod.rs` — is the file actually wired? (Orphan check)
2. Check `Cargo.toml [dev-dependencies]` — are needed crates already present?
3. Check existing test helpers in the module for patterns to reuse.

**L-12** — **fred v9 BYSCORE requires `f64`, not `&str`.**
`"-inf"` → `ZRangeBound::Lex` (wrong). `f64::NEG_INFINITY` → `ZRangeBound::Score` (correct).

---

## Meta

**L-13** — **My memory is only useful if I read it.**
The saveup system, FACTS.md, and PONDERS.md only reduce future orientation cost if
I actually retrieve them at session start. The boot ritual (L-01 through L-03) is
not optional overhead — it is the efficiency multiplier for every session that follows.
