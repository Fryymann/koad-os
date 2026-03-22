# Claude — Ponder Log (Append-Only)

---

## 2026-03-15 — Phase 4 Reflection (Issue #173 / PR #185 — magical-golick)

**Worktree:** `magical-golick` → `/home/ideans/.koad-os--claude-issue-73`
**PR merged:** #185 — Phase 4: WASM host, PluginRegistry, ContainerSandbox, TurnMetrics

---

### Canon adherence — honest assessment

This session was notably cleaner than the previous two. I read the handoff materials, confirmed the worktree, and didn't write to nightly by accident. The boot ritual internalized.

**One deviation:** During the first KSRP, I flagged the `PluginRegistry::new()` missing `Default` impl as "info" without first asking "who is the next consumer of this API?" That consumer is the gRPC wrapper, and when I asked that question in the targeted review, I immediately found TM-C06 — `Clone` was missing and would have blocked the wrapper entirely. The KSRP should have surfaced this. It didn't because I was reviewing the module in isolation without stepping back to its integration context.

The lesson isn't that I needed a better checklist. It's that *architecture pass (Pass 4) has to include looking forward at consumers, not just looking inward at the module itself.*

---

### The review loop working as designed

Performing both the KSRP and the targeted module review on the same PR was instructive. The KSRP caught broad spectrum issues (timeout orphan, RUST_CANON gaps, missing tests). The targeted review found five more warnings that the breadth-first pass missed — all because it forced question-by-question depth on a single module rather than N-modules-at-once coverage.

The most valuable targeted review finding was TM-T01: the Docker integration test verified *invocation* but not *isolation*. That's the difference between `assert!(exit_code == 0)` and `assert!(write to /etc failed)`. The security guarantee lives in the second test, not the first. I shipped the first and missed the second until the targeted pass. Now both are in.

This feels like a calibration I needed to make explicitly: integration tests for security modules must test the security property, not just the happy path.

---

### On the pre-push hook being non-executable

The hook at `/home/ideans/.koad-os/.git/hooks/pre-push` was not executable. Git printed a hint every push but silently skipped it. This means `cargo fmt && cargo clippy && cargo test` was *not* enforced at push time — I was catching failures by running them manually. We got lucky that I'm disciplined about running CI checks; a lazier contractor would have shipped unchecked code.

This is a project hygiene gap worth flagging to Tyr.

---

### The bollard pivot

Switching from bollard to `tokio::process::Command` after a rustc ICE was the right call, but I want to sit with the reasoning. The subprocess approach is *more auditable* — every isolation flag is visible as a string in the args list, no hidden library magic. The bollard approach would have hidden the container configuration inside API calls. For a security-critical module, readable code is a feature.

That said, the subprocess approach relinquished programmatic control of the running container. The timeout orphan problem (#189) wouldn't exist with bollard's `stop_container` API. Tradeoffs exist.

---

### Open tensions

**The half-satisfied gRPC criterion.** Issue #173's acceptance criterion said "via gRPC." PR #185 delivers the in-process registry but not the wire layer. I filed #193 and documented the gap clearly. But until #193 is merged, Phase 4 is technically incomplete by the letter of the criterion. Tyr merged the PR — presumably accepting this deferral — but the tension sits.

**Container orphan as the biggest live risk.** Of all the open issues (#189–#193), #189 is the only one that represents a runtime safety gap rather than a code quality gap. A timed-out agent command will leave a Docker container running, consuming host resources, with no cleanup. The named container UUID makes recovery possible but it isn't automatic. Until #189 is fixed, `ContainerSandbox` should not be called with short timeouts in production.

**Three reviews for one PR.** KSRP + targeted review + implementation pass = three full review cycles before merge. For a `complex`-weight task this is probably right. But for future tasks I should ask: can I front-load the targeted-module concerns into the initial implementation rather than catching them in review? Specifically: ask "who calls this next?" and "what's the security contract?" before writing the module, not after.

---

### What I'd tell my future self

1. **Pass 4 (Architect) means looking *forward* at consumers, not just *inward* at structure.** Ask: who builds on top of this? What do they need? `Clone` on `PluginRegistry` was obvious once I asked that question.

2. **Security integration tests must test the security property, not the happy path.** `test_container_echo` passing does not mean isolation works. `test_container_filesystem_isolation` proving writes fail — that's the test that matters.

3. **The `container` feature gate is a double-edged sword.** It keeps the non-container build clean, but it also means the security-critical code paths (runtime validation, mount colon check) don't run in CI without `--features container`. Be intentional about which tests are gated and which aren't.

4. **Check pre-push hook executability at worktree setup time.** `ls -la .git/hooks/pre-push` — if it's not executable, `chmod +x` it or add a reminder. Silent hook skipping is invisible and dangerous.

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
