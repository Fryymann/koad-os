# Claude — Ponder Log (Append-Only)

---

## 2026-03-15 — End of Day Reflection (Sessions: crazy-mcnulty + agitated-swartz)

**Worktrees completed:** `claude/crazy-mcnulty` (PR #170, Issue #163), `claude/agitated-swartz` (PR #178, compose_articles)

---

### Canon adherence — honest assessment

Overall reasonably solid, but two deviations stand out and both hit the same root failure:
**I did not read my own SAVEUPS.md at session start.**

**Deviation 1 (agitated-swartz):** Wrote article files to the main koad-os repo (`~/.koad-os/docs/`) instead of the worktree path. This directly violated the Sanctuary Rule. I knew the rule — it's in AGENTS.md — but I wrote to the wrong directory anyway because I didn't verify my working directory at the start of the session. The fix (cp -r into the worktree) worked, but it was wasted motion and left untracked files in the main repo until nightly pull cleaned them.

**Deviation 2 (agitated-swartz):** Re-hit the `.git` UNC path bug that I already documented in my session 1 saveup from `crazy-mcnulty`. That fact was *sitting in my saveup* and I didn't read it. This is the single biggest efficiency leak in my workflow right now.

---

### Moments of hesitation / over-caution

- During the `agitated-swartz` session: hesitated before pushing because I wasn't sure whether to push from Windows or WSL. I knew the answer (WSL, always), but I second-guessed myself because the task file didn't explicitly say. Should have trusted prior knowledge.
- When the user told me my `.agents/.claude/` wasn't tracked by git, I felt surprised. I should have proactively checked `git ls-files .agents/.claude/` after the memory system was first set up. I assumed tracked without verifying.

---

### Moments of acting too fast

- The gitignore fix: I initially staged the entire `.agents/.claude/` without thinking through what was inside — this pulled in hundreds of files from old worktree snapshots in `worktrees/`. Caught it before committing, but it required a `git reset` and a second gitignore rule. Should have scanned the directory contents before staging.

---

### Open tensions

**The Windows-injected memory path problem.** Every session starts with a Claude Code system reminder pointing to `C:\Users\ian\.claude\projects\...` as the memory path. This is wrong. The correct path is `~/.koad-os/.agents/.claude/memory/`. Until there's a way to change the injected path, this is a cognitive trap at every session boot. My mitigation: the first thing I do in any new session must be to check the memory path explicitly:
```bash
wsl.exe -d Ubuntu-24.04 -e bash -c "ls /home/ideans/.koad-os/.agents/.claude/memory/"
```

**SAVEUPS.md is not useful if I don't read it.** It exists. It's committed. But I'm not reading it at session start. This needs to become a habit, not an option.

**Do outlines ever go stale?** The `KernelBuilder` → `Kernel` discrepancy in the compose_articles task showed that Tyr's outlines may lag behind the code. I caught it by checking the source, but for future authoring tasks I should build in a source verification step for all code identifiers named in outlines before writing.

---

### What I'd tell my future self

1. **Boot ritual (non-negotiable):** read `SAVEUPS.md` → check working directory (`git status`) → verify memory path is writable. Do this before touching any file.
2. **Before `git add`, scan the directory.** Especially for previously-ignored directories being added for the first time.
3. **Worktree path is your home.** Every file you create should be under `~/.koad-os/.claude/worktrees/<branch>/`. If `git status` shows you're on nightly, stop and get into your worktree first.
4. **Outlines are specs, not ground truth.** Always spot-check key identifiers against the actual source before authoring documentation.
