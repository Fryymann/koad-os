<aside>
📋

**Status:** CANONICAL DRAFT — Pending implementation

**Author:** Ian Deans (Principal Systems & Operations Engineer)

**Target:** Contractor Agent (Claude Code) for repo-level enforcement

</aside>

---

## 1. Design Goals

- **Speed over ceremony.** Issues resolve fast in KoadOS. The git flow must not bottleneck agentic velocity.
- **Tracking without gatekeeping.** Every change is traceable (who, what, why) without mandatory human review on every commit.
- **Parallel agent safety.** Multiple agents (Sky, Tyr, contractors) may work simultaneously without stepping on each other.
- **Guardrails at CI, not at process.** The pipeline enforces quality — not branch policies or PR rituals.
- **Solo-dev + agents reality.** This is not a 20-person team. The flow must stay lean.

---

## 2. Architecture: Protected Trunk + Agent Auto-Merge Lanes

The flow is a hybrid of **trunk-based development** and **tiered branch lanes** with CI-gated auto-merge for agent work.

### 2.1 Branch Topology

```
main (protected)
├── agent/<agent-name>/<task-slug>   ← Agent work branches (auto-merge on CI pass)
├── ian/<task-slug>                  ← Admin work branches (PR optional, self-review)
└── arch/<feature-slug>              ← Architectural / breaking changes (PR required)
```

### 2.2 Branch Naming Convention

| **Pattern** | **Used by** | **Merge policy** | **Example** |
| --- | --- | --- | --- |
| `agent/<name>/<slug>` | Any KoadOS agent | Auto-merge if CI passes | `agent/sky/fix-stream-timeout` |
| `ian/<slug>` | Admin (Ian) | Direct push or lightweight PR | `ian/refactor-cass-memory` |
| `arch/<slug>` | Anyone (architectural) | PR with self-review required | `arch/citadel-grpc-v2` |
| `hotfix/<slug>` | Anyone (emergency) | Direct push to `main` | `hotfix/redis-panic` |

### 2.3 Tiered Merge Policy

| **Change type** | **Flow** | **Review** |
| --- | --- | --- |
| Hotfix / typo / config tweak | Direct push to `main` | None (CI still runs) |
| Scoped agent task | `agent/*` → auto-merge on CI pass | Automated only |
| Admin feature work | `ian/*` → merge or PR at discretion | Optional self-review |
| Architectural / breaking | `arch/*` → PR with review checkpoint | Required self-review |
| Release snapshot | Tag on `main` (e.g. `v0.1.0`) | Manual tag |

---

## 3. Git Worktrees for Parallel Agent Isolation

<aside>
⚠️

**Critical constraint:** All KoadOS agents (Sky, Tyr, contractors) run on the **same physical machine**. Standard `git checkout` only allows one branch per working directory — parallel agents would clobber each other's files. Git Worktrees solve this by giving each agent its own isolated filesystem while sharing a single git history.

</aside>

### 3.0 Why Worktrees (Not Clones, Not Branches)

**The problem:** A single git checkout = one working directory = one branch at a time. If Sky is editing `cass/src/memory.rs` on branch `agent/sky/fix-memory` and Tyr tries to checkout `agent/tyr/refactor-cass` in the same directory, they destroy each other's uncommitted work. `git stash` is not an option for autonomous agents.

**Why not multiple clones?** Cloning the repo N times duplicates the entire git object database. A 500MB repo × 5 agents = 2.5GB+ of redundant history. Each clone needs its own `git fetch`. Refs diverge. It's wasteful and fragile.

**Why worktrees work:** A `git worktree` is a linked working directory that shares the same `.git` data (object database, refs, config) as the main checkout. Each worktree gets:

- Its own **HEAD** (checked out branch)
- Its own **index/staging area**
- Its own **working files** on disk
- Its own **build artifacts** (`target/` in Rust)

But all worktrees share:

- The **git object database** (commits, blobs, trees)
- **Refs** (branches, tags) — a commit in one worktree is instantly visible to all others
- **Config** (`.gitconfig`, hooks)

One `git fetch` updates all worktrees. Disk cost is only the working files + build artifacts per worktree, not the full history.

**Industry validation:** This is exactly how Cursor's Parallel Agents, Claude Code multi-session workflows, and production teams like [incident.io](http://incident.io) operate. Anthropic officially recommends worktrees for running multiple Claude Code sessions simultaneously. It's the emerging standard for agentic development.

### 3.1 How It Works Under the Hood

When you run `git worktree add ../koad-os--sky -b agent/sky/fix-stream`, Git:

1. Creates a new directory `../koad-os--sky` with a full copy of the working files
2. Places a `.git` **file** (not folder) in that directory pointing back to the main repo: `gitdir: /path/to/koad-os/.git/worktrees/koad-os--sky`
3. Stores worktree-specific data (HEAD, index) under `.git/worktrees/koad-os--sky/` in the main repo
4. **Enforces one-branch-per-worktree** — Git will refuse to check out a branch that's already active in another worktree (this is a feature, not a bug — it prevents agent collisions)

```
~/.git/                          ← Shared git database
├── objects/                     ← Shared commits, blobs, trees
├── refs/                        ← Shared branches, tags
├── worktrees/
│   ├── koad-os--sky/            ← Sky's HEAD + index
│   └── koad-os--tyr/            ← Tyr's HEAD + index
│
koad-os/                         ← Main worktree (main branch)
koad-os--agent-sky-fix-stream/   ← Sky's isolated workspace
koad-os--agent-tyr-cass-refac/   ← Tyr's isolated workspace
```

### 3.2 KoadOS-Specific Advantages

**Rust/Cargo is worktree-friendly.** Unlike Node.js projects (where each worktree needs its own `node_modules` install), Cargo resolves dependencies from `Cargo.toml`/`Cargo.lock` which are version-controlled. Each worktree gets its own `target/` directory for build artifacts automatically — no extra setup. `cargo build` in one worktree does not interfere with another.

**No port conflict problem.** KoadOS is a CLI/daemon system, not a web app with dev servers fighting over port 3000. The main runtime concern is shared Redis, which is addressed below.

**Lightweight.** Creating a worktree takes seconds (just copies working files). Removing one is instant. Perfect for ephemeral agent tasks.

### 3.3 Known Caveats & Mitigations

| **Caveat** | **Risk for KoadOS** | **Mitigation** |
| --- | --- | --- |
| **Merge conflicts with yourself** — two agents editing the same file in parallel will create conflicts at merge time | Medium — likely if agents work on overlapping crates | Scope agent tasks to distinct crates/modules. The Citadel/CASS/CLI/Stream boundaries already provide natural isolation. CI catches conflicts at merge time. |
| **Disk space from `target/` dirs** — each worktree builds its own `target/`, Rust release builds can be 1-2GB each | Medium — 5 active worktrees could use 5-10GB | Worktrees are ephemeral — remove after merge. Add a cleanup script or `git worktree prune` to the post-merge flow. Use `cargo clean` in worktrees before removal if needed. Consider `sccache` for shared compilation cache. |
| **Shared Redis state** — agents testing against the same local Redis instance could collide | Low-Medium — only matters during integration tests that write to Redis | Use per-agent Redis key prefixes (e.g., `koad:test:sky:*`) in test harnesses. Or use separate Redis databases (`SELECT 1`, `SELECT 2`, etc.) per worktree. The Citadel's identity lease system already namespaces sessions. |
| **Stale worktrees accumulate** — forgotten worktrees waste disk and create confusion | Low — manageable with discipline | Run `git worktree list` before starting new work. Add `git worktree prune` to agent startup scripts. Enforce cleanup in the Agent Contract (Section 8). |
| **No cross-worktree conflict detection** — Git won't warn you that two worktrees are editing the same file until merge | Low — task scoping minimizes overlap | Future: Add a pre-merge check script that diffs the agent branch against `main` and flags overlapping file paths with other active agent branches. |

### 3.4 Directory Layout

```
~/projects/
├── koad-os/                        ← Main worktree (main branch, never used for feature work)
├── koad-os--agent-sky-fix-stream/  ← Sky's worktree
├── koad-os--agent-tyr-cass-refac/  ← Tyr's worktree
└── koad-os--ian-cli-overhaul/      ← Admin worktree
```

Naming convention: `koad-os--<branch-slug>` (double-dash separator for readability).

### 3.5 Worktree Lifecycle Commands

```bash
# Create a new agent worktree (branching from main)
git worktree add ../koad-os--agent-sky-fix-stream -b agent/sky/fix-stream-timeout

# List active worktrees
git worktree list

# Remove after merge (clean working tree required, or use --force)
git worktree remove ../koad-os--agent-sky-fix-stream

# Prune stale worktree references (e.g., after manual directory deletion)
git worktree prune
```

### 3.6 Rules

- **One worktree per active task.** No two agents share a worktree. Git enforces one-branch-per-worktree.
- **Worktrees are ephemeral** — create on task start, remove after merge. They are not persistent workspaces.
- **The main worktree** (`koad-os/`) stays on `main` and is never used for feature work. It's the merge target and CI reference.
- **Agents must not `cd` into another agent's worktree.** Each agent operates exclusively in its own directory.
- **Post-merge cleanup is mandatory.** Delete remote branch → `git worktree remove` → verify with `git worktree list`.
- Optional: Use `@johnlindquist/worktree` CLI (`wt` command) or `agentree` for streamlined management.

### 3.7 Verdict

<aside>
✅

**Git Worktrees are the correct and canonical solution for KoadOS parallel agent isolation.** They solve the single-machine constraint cleanly, are lightweight, align with the existing Contributor Manifesto, and are the industry-standard pattern for agentic development. The caveats are manageable with the mitigations above. No alternative (multiple clones, branch switching, container-per-agent) offers a better tradeoff for KoadOS's solo-dev + agents architecture.

</aside>

---

## 4. Conventional Commits (Enforced)

All commits **must** follow the Conventional Commits specification. This is the primary mechanism for self-documenting history and automated changelog generation.

### 4.1 Format

```
<type>(<scope>): <description>

[optional body]

[optional footer(s)]
```

### 4.2 Allowed Types

| **Type** | **Use** |
| --- | --- |
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `refactor` | Code restructure (no behavior change) |
| `docs` | Documentation only |
| `test` | Adding or fixing tests |
| `chore` | Tooling, config, dependencies |
| `ci` | CI/CD pipeline changes |
| `perf` | Performance improvement |
| `style` | Formatting (no logic change) |

### 4.3 Scopes (KoadOS-specific)

Use the crate or module name as scope: `citadel`, `cass`, `cli`, `stream`, `config`, `sentinel`, `tui`, `agent`, `memory`, `grpc`.

### 4.4 Examples

```
feat(cass): add TCH hydration for dark-mode recovery
fix(stream): resolve timeout panic on empty Redis queue
refactor(citadel): extract identity lease into standalone module
chore(ci): add commitlint to pre-push hook
docs(cli): update koad intel subcommand help text
```

### 4.5 Footer: Issue Linking

Every commit **should** reference its issue number in the footer (per the Contributor Manifesto's Ticket-First rule):

```
fix(stream): resolve timeout panic on empty Redis queue

Closes #42
```

---

## 5. CI Enforcement (GitHub Actions)

The CI pipeline is the **sole quality gate** for `main`. No branch policy or human review replaces it.

### 5.1 Required Checks on `main`

- `cargo fmt --check` — formatting
- `cargo clippy -- -D warnings` — lint (zero warnings)
- `cargo test` — full test suite
- `cargo build --release` — build verification
- `commitlint` — commit message format validation

### 5.2 Agent Auto-Merge Flow

```
1. Agent creates branch: agent/sky/fix-stream-timeout
2. Agent commits work (conventional commits)
3. Agent pushes branch → CI triggers
4. CI passes all checks → auto-merge to main (squash or rebase)
5. Branch deleted, worktree removed
```

If CI fails, the agent must fix and re-push. No manual intervention unless the agent is stuck.

### 5.3 Suggested GitHub Actions Workflow

```yaml
# .github/workflows/ci.yml
name: KoadOS CI

on:
  push:
    branches: [main, "agent/**", "ian/**", "arch/**", "hotfix/**"]
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - name: Format check
        run: cargo fmt --check
      - name: Clippy
        run: cargo clippy -- -D warnings
      - name: Test
        run: cargo test
      - name: Build
        run: cargo build --release

  commitlint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: wagoid/commitlint-github-action@v6
```

---

## 6. Branch Protection Rules (GitHub)

Apply to `main`:

- ✅ Require status checks to pass before merging (`check`, `commitlint`)
- ✅ Require branches to be up to date before merging
- ❌ Do NOT require pull request reviews (agents auto-merge)
- ❌ Do NOT require signed commits (optional, add later)
- ✅ Allow force pushes: **No**
- ✅ Allow deletions: **No**

---

## 7. Release & Tagging

- Releases are manual tags on `main` using semver: `v0.1.0`, `v0.2.0`, etc.
- Use annotated tags: `git tag -a v0.1.0 -m "Release v0.1.0: Citadel bootstrap"`
- No release branches. `main` is always the release candidate.
- Changelog is auto-generated from conventional commit history.

---

## 8. Agent Contract (Instructions for Claude Code / Contractor Agents)

<aside>
🤖

**When working in the koad-os repo, all agents MUST follow these rules:**

</aside>

1. **Branch from `main`** using the pattern `agent/<your-name>/<task-slug>`.
2. **Use a Git worktree** — never work directly in the main checkout.
3. **Conventional commits only.** Every commit must match `<type>(<scope>): <description>`. Reference the issue number in the footer.
4. **Small, surgical commits.** One logical change per commit. Do not bundle unrelated changes.
5. **Run checks before pushing:** `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
6. **Push and let CI decide.** If CI passes, the branch auto-merges. If CI fails, fix and re-push.
7. **Clean up after yourself.** After merge, delete the remote branch and remove the local worktree.
8. **Never force-push to `main`.** Never rewrite `main` history.
9. **If your change is architectural or breaking**, escalate: create a PR on an `arch/*` branch and flag Ian for review.
10. **Condition Green applies.** Zero warnings, full build, verified tests — or it doesn't merge.

---

## 9. Local Setup Checklist

- [ ]  Install `commitlint` and `@commitlint/config-conventional`
- [ ]  Add `.commitlintrc.yml` to repo root:

```yaml
extends:
  - '@commitlint/config-conventional'
rules:
  scope-enum:
    - 2
    - always
    - [citadel, cass, cli, stream, config, sentinel, tui, agent, memory, grpc]
```

- [ ]  Add pre-push git hook (via `cargo-husky` or manual `.git/hooks/pre-push`):

```bash
#!/bin/sh
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

- [ ]  Configure GitHub branch protection on `main` per Section 6
- [ ]  Add `.github/workflows/ci.yml` per Section 5.3
- [ ]  Install `@johnlindquist/worktree` globally (optional but recommended):

```bash
npm install -g @johnlindquist/worktree@latest
```

---

## 10. Quick Reference Card

<aside>
⚡

**Daily flow for agents:**

`git worktree add` → branch `agent/<name>/<task>` → work → conventional commits → push → CI auto-merges → `git worktree remove`

**Daily flow for Admin:**

Direct push to `main` for small fixes, `ian/*` branch for features, `arch/*` + PR for big changes.

**Release:**

`git tag -a v0.x.0 -m "..."` on `main` when ready.

</aside>