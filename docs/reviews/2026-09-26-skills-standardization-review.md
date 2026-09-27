# Skills Standardization Review — 2026-09-26

Author: Clyde (Claude Code, Opus 5.5), at Ian's request.
Status: **proposal for discussion.** No skills have been moved or edited.

## The standard today

- **Agent Skills** ([agentskills.io/specification](https://agentskills.io/specification)) is the
  cross-vendor format. Anthropic published it as an open standard on 2025-12-18; it is stewarded
  through the Agentic AI Foundation. OpenAI (Codex, ChatGPT), Google (Gemini CLI), Microsoft (VS Code,
  Copilot), Cursor, JetBrains, Goose and 25+ others ship compatible loaders.
- **Format:** a directory named after the skill, containing `SKILL.md` (YAML frontmatter + Markdown),
  plus optional `scripts/`, `references/` and `assets/`.
  - Required frontmatter: `name` (1–64 chars, `a-z0-9` and single hyphens, **must equal the directory
    name**) and `description` (≤1024 chars, what it does and when to use it).
  - Optional: `license`, `compatibility` (≤500 chars), `metadata` (string→string map, e.g. `author`,
    `version`), `allowed-tools` (experimental).
  - Progressive disclosure: name + description are always loaded (~100 tokens); the body loads on
    activation (keep it under 500 lines / ~5000 tokens); `references/` and `scripts/` load on demand,
    one level deep.
  - Validate with `skills-ref validate <dir>`.
- **Shared location:** `~/.agents/skills` (user) and `.agents/skills` (project) are read by Codex,
  Gemini CLI, Cursor, Copilot and others. Claude Code reads only `~/.claude/skills` and
  `.claude/skills`
  ([anthropics/claude-code#56193](https://github.com/anthropics/claude-code/issues/56193)), so it
  needs symlinks.
- **Distribution:** Vercel's `skills` CLI (`npx skills`, now 1.7.0; Jupiter's lock was written by an
  older version in June) installs from GitHub repos or local paths into `~/.agents/skills`, symlinks
  into each agent's directory (`-a '*'` for all), records source and folder hash in
  `~/.agents/.skill-lock.json`, and refreshes with `npx skills update`.

## What Jupiter has

### Where skills live

| Location | Read by | What's there |
|---|---|---|
| `~/.agents/skills` | Codex, Gemini CLI, Continue (via symlinks), Claude Code (via symlinks) | 27 skills: 14 third-party (lock-tracked), 11 from the KoadOS repo, 2 Stripe skills tracked by nothing. **The live copies.** |
| `~/.claude/skills` | Claude Code | Symlinks into `~/.agents/skills`, plus `ollama-delegate` (real dir, Claude-only) |
| `~/.continue/skills` | Continue | 15 symlinks into `~/.agents/skills` |
| `~/.codex/skills` | — | Empty. Per current docs Codex reads `~/.agents/skills` directly (not tested on Jupiter). |
| `koados-citadel/skills/` | **`npx skills add`** (the CLI discovers this one) | 12 KoadOS skills, oldest text (2026-07-05) |
| `koados-citadel/plugin/skills/` | `install.sh` | 10 KoadOS skills, newer than `skills/` |
| `~/.citadel-jupiter/skills` | Nothing found | Copied from `plugin/skills/` by `install.sh` on every update |
| `KAPVs/hermes/skills` | Hermes | Tailored forks of `agent-boot`, `koad-intel`, `koad-map`, `rtk` |
| `KAPVs/{cid,scribe,tyr}/skills` | — | Non-conforming (see below) |

### Problems found

1. **The live KoadOS skills are unversioned and ahead of the repo.** `~/.agents/skills` holds fixes
   that exist in no repo: Hermes's August corrections to `agent-boot`, 14 lines in `koad-intel`,
   17 lines in `koad-signal` (the "signal bus is a stub" warning) and 9 in `koad-fleet`. None of the
   KoadOS skills are in the lock file; they were copied in by hand.
2. **Three sources in one repo, all stale.** `skills/` and `plugin/skills/` disagree with each other
   and with the live copies. `agent-boot` exists in **four** different versions across Jupiter,
   `koad-intel` and `koad-signal` in three.
3. **The repo would install the oldest text.** `npx skills add ~/koados-citadel --list` finds the 12
   skills under `skills/`, the July 5 copies. Using the standard tool today would roll every agent
   back.
4. **`install.sh` deploys skills nowhere useful.** It copies `plugin/skills/` into
   `~/.citadel-jupiter/skills`, which no harness reads.
5. **Skills teach broken tools** (see the tooling relevance review, #226):
   - `koad-signal` shows `koad signal send` as the way to message agents; the service drops messages.
   - `koad-fleet` documents `koad project register/info/sync`; they are placeholders.
   - `koad-system` lists `koad system save` and `backup` as routine. `save` takes the Citadel down,
     and `backup` reports fake success.
   - `koad-intel` teaches `koad intel snippet`, which harness Read now covers.
6. **Agent forks instead of profiles.** Hermes's `agent-boot` is a full fork of the universal skill.
   Every universal fix has to be merged by hand (Hermes did this on 2026-08-19).
7. **Non-conforming agent skills:**
   - `KAPVs/cid/skills/SKILL.md` sits at the skills root: it's a manual, not a skill.
   - `KAPVs/scribe/skills/github-project-auditor/SKILL.md` has no frontmatter, so no harness can
     discover it.
   - `KAPVs/tyr/skills/hello-world/` is empty.
8. **Minor spec deviations in third-party skills:** `firebase-ai-logic-basics` has a top-level
   `version` key (belongs under `metadata`); `upgrade-stripe` has `alwaysApply` (a Cursor rules
   field). These are upstream issues; local edits would be overwritten by `npx skills update`.
9. **`ollama-delegate` is Claude-only.** It lives as a real directory in `~/.claude/skills`, so the
   local-model harnesses it is meant to feed can't see it.

All KoadOS skills pass the format rules (name, name = directory, description length, allowed keys,
body length). The problems are structure and content, not syntax.

## Proposed standard for KoadOS skills

1. **One source:** `koados-citadel/skills/`, the path the `skills` CLI discovers. Retire
   `plugin/skills/`, or make the Claude plugin point at `skills/`.
2. **Reconcile first:** bring the newest text from `~/.agents/skills` into `skills/`, review the
   diffs, and commit. The live copies are the most correct version today.
3. **Install with the CLI, not `cp`:** `npx skills add <koad-os repo or local path> -g -a '*' -y`,
   so KoadOS skills are lock-tracked like third-party ones and `npx skills update` refreshes every
   harness. `install.sh` should call the CLI (or print the command) instead of copying into
   `~/.citadel-jupiter/skills`.
4. **Agent variants as references, not forks:** keep one `agent-boot` and move agent-specific
   behaviour into `references/<agent>.md` (e.g. `references/hermes.md`), loaded when
   `$KOAD_AGENT_NAME` matches. Agent-private skills that are truly unique stay in the KAPV, but in
   spec format.
5. **Spec hygiene for every KoadOS skill:**
   - `license: MIT` (matches the repo).
   - `compatibility: Requires a running KoadOS Citadel ($KOAD_HOME, koad CLI)`.
   - `metadata: { author: koados, version: "<semver>" }`.
   - Move long tables (for example the agent-boot traps table) into `references/`.
6. **Validate in CI** with `skills-ref validate` over `skills/*`.
7. **Content pass after the tooling fixes:** rewrite `koad-signal` (inbox files until the service is
   real), `koad-fleet`, `koad-system` and `koad-intel` to match what actually works, and shrink
   `agent-boot` once the session lease is fixed.
8. **Housekeeping:**
   - Move `ollama-delegate` into `skills/` so every harness sees it.
   - Convert or delete the Cid manual, the Scribe auditor and Tyr's empty `hello-world`.
   - Run `npx skills update` to refresh the June third-party installs, reviewing diffs first since
     skills are executable instructions.

## Suggested order

1. Reconcile the live copies into `skills/` and delete `plugin/skills/` (points 1–2).
2. Switch installation to the CLI and update `install.sh` (point 3).
3. Add spec metadata and a CI validation step (points 5–6).
4. Fold the Hermes fork into the universal skill with a reference file (point 4). Needs Hermes's
   review.
5. Content pass on skills that teach broken tools (point 7), alongside the fixes from #226.
6. Housekeeping (point 8).
