# Housekeeping, Release Promotion & Episode Partition Design — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Clear the five priority actions from the 2026-07-05 status report: push pending commits, remove the dead `intelligence` field, refresh SITREP.md, promote nightly → main with a v3.2.0 tag, prune stale branches, and produce an approved design for episode partition keying.

**Architecture:** Six sequential tasks on branch `nightly`. Tasks 1–3 land cleanup commits on nightly; Task 4 promotes the cleaned state to `main` and tags the release; Task 5 prunes branches; Task 6 produces a design document (no implementation — Dood Gate applies). Orchestrator (Clyde) executes git/release operations directly and delegates bounded work per the matrix below.

**Tech Stack:** Rust workspace (cargo), git/GitHub (SSH remote `origin` = github.com/Fryymann/koad-os), Ollama local models, cavecrew subagents.

---

## Delegation Matrix

| Task | Executor | Rationale |
| :--- | :--- | :--- |
| 1. Push nightly | **Orchestrator** | Trivial git op; outward-facing |
| 2. Remove dead `intelligence` field | **cavecrew-builder** (edits) + orchestrator (verify/commit) | Surgical 2-file mechanical edit — exactly builder's scope |
| 3. SITREP.md refresh | **phi4-mini:3.8b** via ollama-delegate (Branch A: doc update) | Content fully drafted below; model does format/merge only. Escalation: granite3.3:2b → qwen3.5:9b → orchestrator |
| 4. Promote nightly → main + tag | **Orchestrator** | Release op, irreversible-ish, needs human gate |
| 5. Branch pruning | **cavecrew-investigator** (report) + orchestrator (deletes after approval) | Report = read-only fan-out; deletes = destructive, gated |
| 6. Episode partition design | **cavecrew-investigator** (code survey) + **Orchestrator** (design doc) | Architectural reasoning is NOT delegable to lesser models. Dood Gate before any code |

**Ollama concurrency:** before any local-model dispatch, run `ollama ps` and apply the ollama-delegate skill's concurrency rules. If `ollama ps` fails, stop delegation and execute that step in the orchestrator instead.

**Hard gates (require Ian / Dood):**
- Task 4 Step 4: push of `main` + tag (release publication)
- Task 5 Step 4: remote branch deletion list approval
- Task 6 Step 4: Condition Green on the design doc

---

### Task 1: Push pending commits to origin/nightly

**Files:** none (git only)

- [ ] **Step 1: Confirm what is unpushed**

Run: `git log origin/nightly..nightly --oneline`
Expected (before Tasks 2–3 land):
```
5228d58 fix(cass): make L3 threshold verdict final in semantic search
17b4712 feat(cass): wire SearchSemantic min_score threshold
```

- [ ] **Step 2: Push**

Run: `git push origin nightly`
Expected: `To github.com:Fryymann/koad-os.git` … `nightly -> nightly`, exit 0.
(SSH auth restored 2026-07-02. If SSH fails, HTTPS fallback is documented in agent memory `github-push-workaround`.)

- [ ] **Step 3: Verify clean**

Run: `git log origin/nightly..nightly --oneline | wc -l`
Expected: `0`

---

### Task 2: Remove dead `intelligence` field from CassHydrationService

**Files:**
- Modify: `crates/koad-cass/src/services/hydration.rs` (struct :24-30, constructor :32-47, doc comment :61, 4 test call sites :447, :482, :496, :559, :561, :633, :635, imports :11/:14)
- Modify: `crates/koad-cass/src/main.rs:95-100`

**Context for the builder (zero-context engineer):** `CassHydrationService.intelligence` is injected at construction but never read — `git log -S 'self.intelligence'` over the file's full history returns nothing. Cargo emits `warning: field 'intelligence' is never read`. The fix is pure removal. Note: `main.rs` also constructs `CassMemoryService::new(storage.clone(), intelligence.clone())` on line 94 — that usage is LIVE, do not touch it; only remove the argument from the `CassHydrationService::new` call.

- [ ] **Step 1: Confirm the warning exists (failing state)**

Run: `~/.cargo/bin/cargo check -p koad-cass 2>&1 | grep 'never read'`
Expected: `warning: field 'intelligence' is never read`

- [ ] **Step 2: Delegate edits to cavecrew-builder**

Dispatch `caveman:cavecrew-builder` with this exact spec:

In `crates/koad-cass/src/services/hydration.rs`:

Struct (lines 24–30) — remove the `intelligence` field:
```rust
pub struct CassHydrationService {
    storage: Arc<dyn MemoryTier>,
    hierarchy: Arc<HierarchyManager>,
    codegraph: Arc<CodeGraph>,
    pulse_store: Option<Arc<dyn PulseTier>>,
}
```

Constructor (lines 32–47) — remove the parameter and field init:
```rust
impl CassHydrationService {
    /// Creates a new `CassHydrationService`.
    pub fn new(
        storage: Arc<dyn MemoryTier>,
        hierarchy: Arc<HierarchyManager>,
        codegraph: Arc<CodeGraph>,
    ) -> Self {
        Self {
            storage,
            hierarchy,
            codegraph,
            pulse_store: None,
        }
    }
```

Doc comment (line 61) — change:
```rust
    /// Returns a `tonic::Status` if storage queries or intelligence distillation fail.
```
to:
```rust
    /// Returns a `tonic::Status` if storage queries fail.
```

Test call sites (lines 449, 496, 561, 635) — change every
```rust
CassHydrationService::new(storage, hierarchy, codegraph, intelligence)
```
to
```rust
CassHydrationService::new(storage, hierarchy, codegraph)
```
and delete each preceding now-unused binding of the form (lines 447, 482, 559, 633):
```rust
let intelligence = Arc::new(InferenceRouter::new_default()?);
```
(Exact text at each site may wrap differently after rustfmt — match on the `CassHydrationService::new` call and the `let intelligence =` binding, not on line numbers.)

Imports (lines ~11 and ~14) — after the edits above, remove `IntelligenceRouter` and `InferenceRouter` imports **only if no other reference remains in this file** (grep the file for each symbol first; `cargo check` in Step 3 is the backstop).

In `crates/koad-cass/src/main.rs` (lines 95–100) — remove the fourth argument:
```rust
    let hydration_svc = CassHydrationService::new(
        storage.clone(),
        hierarchy.clone(),
        codegraph.clone(),
    )
    .with_pulse_store(Arc::clone(&redis_tier) as Arc<dyn koad_cass::storage::PulseTier>);
```
Leave line 94 (`CassMemoryService::new(storage.clone(), intelligence.clone())`) untouched.

- [ ] **Step 3: Verify warning gone, no new errors**

Run: `~/.cargo/bin/cargo check -p koad-cass 2>&1 | tail -5`
Expected: `Finished` line, zero errors, no `never read` warning, no `unused import` warning.

- [ ] **Step 4: Run crate tests**

Run: `~/.cargo/bin/cargo test -p koad-cass 2>&1 | grep 'test result'`
Expected: all lines read `... 0 failed ...` (44 test fns across targets; 41-passed suite is the big one).

- [ ] **Step 5: Commit**

```bash
git add crates/koad-cass/src/services/hydration.rs crates/koad-cass/src/main.rs
git commit -m "refactor(cass): drop never-read intelligence field from hydration service

Field was injected at construction but never read in the file's history
(git log -S confirms). Distillation work landed in the enrichment worker
instead.

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 3: Refresh SITREP.md

**Files:**
- Modify: `SITREP.md` (repo root — full content replacement)

- [ ] **Step 1: Check Ollama state**

Run: `ollama ps`
Apply ollama-delegate concurrency rules for a Small model (phi4-mini:3.8b, ≤5GB: OK if ≤1 model loaded). If `ollama ps` errors: skip delegation, orchestrator writes the file directly with the content below and jumps to Step 4.

- [ ] **Step 2: Emit delegation packet and dispatch phi4-mini:3.8b**

```json
{
  "model": "phi4-mini:3.8b",
  "task_id": "clyde-2026-07-05-001",
  "objective": "Rewrite SITREP.md with the exact content provided, preserving the existing section structure and emoji headers; done = file contains all provided bullets under correct sections.",
  "context_budget_tokens": 4096,
  "max_output_tokens": 1024,
  "concurrency_class": "small",
  "co_runnable_with": ["qwen2.5-coder:7b", "qwen3:14b", "granite3.3:2b", "nemotron-mini:4b"],
  "timeout_s": 120,
  "acceptance": [
    "All bullets from the supplied content appear under their designated sections",
    "Section headers match the template exactly (🎯 Active Missions, 🗂 AIS Backlog, 🛠️ Recent Accomplishments, 🏗️ Architectural Decisions, 🔜 Immediate Next Actions)",
    "No invented facts beyond the supplied content"
  ]
}
```

Supply this exact target content in the prompt:

```markdown
# Citadel SITREP (Situation Report)
**Date:** 2026-07-05
**Current Objective:** v3.2.0 release promotion (nightly → main) and episode partition keying design for CASS semantic recall.

## 🎯 Active Missions
- [ ] **Episode Partition Keying Design:** Close the episode recall gap — `search_semantic` matches episodes by `session_id.contains(partition)`, which is empty under production ids. Design doc in progress; Dood Gate pending.
- [ ] **v3.2.0 Release Promotion:** Merge nightly → main (86+ commits) and cut the v3.2.0 tag.
- [ ] **Fleet Distribution Prep:** Verify fresh installation on external systems and prepare release tag.
- [ ] **AIS Documentation Sync:** Complete operating documents for the unified installer and `--update` flags.
- [ ] **Docker Integration Guide:** Document WSL Resource configuration constraints for new developers.

## 🗂 AIS Backlog
- [ ] Add machine-readable nMap JSON export workflow.
- [ ] Add link-check script for AIS docs and protocol references.
- [ ] Add owner metadata and last-reviewed dates to legacy docs.

## 🛠️ Recent Accomplishments
- **CASS Semantic Enrichment Pipeline (P1, live):** Real embeddings via dedicated embedding client (`InferenceTask::Embedding`), Qdrant L3 with fingerprint fallback removed, enrichment worker consuming the `cass:enrichment` Redis stream with `is_meaningful` guard, `backfill_embeddings` migration binary. Verified 6/6 paraphrase recall. See `docs/cass-semantic-recall-verification.md`.
- **SearchSemantic Threshold Wiring (P2):** `min_score` threshold wired end-to-end; L3 threshold verdict made final in semantic search.
- **CASS Token-Aware Memory Metadata (P2, deployed):** Optional `MemoryMetadata` layer (SQLite L2 + Redis L1), budget-aware hydration, offline backfill binary. See `docs/cass-memory-metadata.md`.
- **Rook Decommission:** Hardcoded agent identity removed from the codebase.
- **Unified Installer & Updater (P1):** Single root entrypoint `install.sh` supporting `--install` and `--update`.

## 🏗️ Architectural Decisions
- **Episode partition keying requires its own design pass** — session ids carry no partition string; fix is a schema/keying decision, not a filter tweak.
- **Private Identity Separation:** User agent settings remain strictly decoupled from core codebase.
- **Rust Toolchain Modernization:** Container builders track modern compiler versions (>=1.90).

## 🔜 Immediate Next Actions
1. Land dead-code cleanup and SITREP refresh on nightly; push.
2. Merge nightly → main, tag v3.2.0, push (Dood approval required).
3. Prune merged local branches; review remote-only branches for deletion (Dood approval required).
4. Complete episode partition keying design doc; obtain Condition Green.
```

- [ ] **Step 3: Verify acceptance**

Diff model output against the supplied content. All three acceptance criteria must hold. On failure: one retry with the diff as feedback, then escalate per doc path (granite3.3:2b → qwen3.5:9b → orchestrator writes it directly).

- [ ] **Step 4: Write file and commit**

Write accepted content to `SITREP.md`, then:

```bash
git add SITREP.md
git commit -m "docs: refresh SITREP — semantic pipeline shipped, v3.2.0 promotion queued

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
```

---

### Task 4: Promote nightly → main and tag v3.2.0

**Files:** none (git only)

**Precondition:** Tasks 1–3 complete; `~/.cargo/bin/cargo check --workspace` and `cargo test --workspace` green on nightly (re-verified in Step 2).

- [ ] **Step 1: Push accumulated nightly commits**

Run: `git push origin nightly`
Expected: exit 0, `git log origin/nightly..nightly --oneline | wc -l` → `0`

- [ ] **Step 2: Re-verify green on exact release commit**

Run: `~/.cargo/bin/cargo check --workspace 2>&1 | tail -3 && ~/.cargo/bin/cargo test --workspace 2>&1 | grep -c 'test result: ok'`
Expected: `Finished` with no errors; test-result count ≥ 34 with no failures (spot-check: `grep 'test result' | grep -v ' 0 failed'` → empty).

- [ ] **Step 3: Merge and tag locally**

```bash
git checkout main
git merge --no-ff nightly -m "release: v3.2.0 — CASS semantic enrichment pipeline, token-aware metadata, unified installer

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
git tag -a v3.2.0 -m "KoadOS v3.2.0 — semantic memory release"
git log --oneline -3 && git describe
```
Expected: merge commit on main, `git describe` → `v3.2.0`.

- [ ] **Step 4: 🚦 HARD GATE — Dood approval**

Present to Ian: merge commit hash, tag, test evidence from Step 2. **Do not push until explicit approval.** If rejected: `git checkout nightly` and leave main/tag local for revision (`git tag -d v3.2.0`, `git branch -f main origin/main` to fully unwind).

- [ ] **Step 5: Push main + tag**

```bash
git push origin main
git push origin v3.2.0
git checkout nightly
```
Expected: both pushes exit 0; session ends back on `nightly`.

---

### Task 5: Branch pruning

**Files:** none (git only)

- [ ] **Step 1: Delete merged local branches (safe — verified merged into nightly)**

```bash
git branch -d feature/cass-qdrant-f1 feature/cass-semantic-enrichment feature/stable-release-v3.2
```
Expected: three `Deleted branch ...` lines. `-d` (not `-D`) so git itself refuses if somehow unmerged.
**Exclusion:** `feature/agent-boot-skill` is checked out in a worktree (`+` marker) — do not touch.

- [ ] **Step 2: Delegate remote-branch report to cavecrew-investigator**

Dispatch `caveman:cavecrew-investigator` with:

> For each remote branch on `origin` with no local counterpart (18 expected, e.g. `agent/claude/issue-73-wasm-plugin-fix`, `claude/agitated-swartz`, `feat/rebuild-phase-*`), produce a table: branch name | last commit date | author | ahead/behind nightly (`git rev-list --left-right --count origin/<b>...nightly`) | merged into nightly? (`git branch -r --merged nightly`). Use `git for-each-ref --sort=-committerdate 'refs/remotes/origin/' --format='%(refname:short) %(committerdate:short) %(authorname)'`. Read-only; no deletions.

- [ ] **Step 3: Orchestrator classifies**

From the report, bucket each branch: **DELETE** (merged into nightly, or stale >30 days with 0 unique commits), **KEEP** (unmerged unique work), **ASK** (unclear). Produce the recommendation list.

- [ ] **Step 4: 🚦 HARD GATE — Dood approves deletion list**

Present buckets to Ian. Delete only approved names.

- [ ] **Step 5: Execute approved deletions**

```bash
git push origin --delete <approved-branch-1> <approved-branch-2> ...
git fetch --prune
git branch -r | wc -l
```
Expected: `- [deleted]` line per branch; final count reflects removals.

---

### Task 6: Episode partition keying — design pass (NO implementation)

**Files:**
- Create: `docs/plans/2026-07-05-cass-episode-partition-keying-design.md`

**Problem statement (from `docs/cass-semantic-recall-verification.md:72`):** `search_semantic` matches episodes by `ep.session_id.contains(agent_name)` (`crates/koad-cass/src/storage/qdrant_tier.rs:527`); production session ids look like `20260606_140451_01834a` — no partition string — so episode recall under production partitions is effectively empty. Fact-card partitioning was fixed (partition = domain prefix); episodes need their own keying decision.

- [ ] **Step 1: Delegate code survey to cavecrew-investigator**

Dispatch `caveman:cavecrew-investigator` with these exact questions:

> 1. Where are episodes CREATED — which code generates `session_id` (grep `session_id` in crates/koad-citadel and koad-cass; find the timestamp-hex format like `20260606_140451_01834a`)? Does the creator know the agent name/partition at creation time?
> 2. Full schema of `episodic_memories` (crates/koad-cass/src/storage/sqlite_tier.rs:35 region) — list every column. Is there any agent/partition column today?
> 3. How do episodes reach Qdrant — which code upserts episode points, what payload fields are attached (qdrant_tier.rs)?
> 4. Every read path that filters episodes by agent/partition — all `session_id.contains` sites and any others.
> 5. How many episode rows exist in production SQLite (`~/.citadel-jupiter` data dir, `episodic_memories` count) — sizing for backfill.

- [ ] **Step 2: Orchestrator writes design doc** (architectural — not delegable)

Document must contain, with no placeholders:
- **Current state:** findings from Step 1 verbatim (schema, write path, read paths, row count).
- **Option A — dedicated partition column:** add `source_agent`/`partition` column to `episodic_memories` + Qdrant payload field; filter on payload at query time. Cover: migration DDL, Qdrant re-upsert/backfill of existing points, write-path change.
- **Option B — encode partition into session_id at creation:** e.g. `clyde_Jupiter_ideans:20260706_...`. Cover: no schema change, but rewrites id semantics; existing rows unfixable without id rewrite; downstream consumers of session_id format.
- **Option C — join through fact cards:** derive episode ownership via fact cards sharing the session_id. Cover: query cost, correctness when episodes have no facts.
- **Backfill strategy** for ~N existing production rows (N from Step 1.5), following the `backfill_embeddings` / `backfill_metadata` binary pattern.
- **Recommendation with rationale** (expected: Option A — explicit column matches the fact-card fix's "partition = payload field" canon, backfill is mechanical; confirm against survey findings).

- [ ] **Step 3: Commit design doc**

```bash
git add docs/plans/2026-07-05-cass-episode-partition-keying-design.md
git commit -m "docs(cass): episode partition keying design — options + recommendation

Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>"
git push origin nightly
```

- [ ] **Step 4: 🚦 HARD GATE — Dood Gate (Condition Green)**

Present design doc to Ian. **No implementation until Condition Green.** On approval, write a separate implementation plan (new doc, TDD, per writing-plans skill) — implementation is explicitly out of scope here.

---

## Execution Order & Dependencies

```
Task 1 (push) ──→ Task 2 (dead code) ──→ Task 3 (SITREP) ──→ Task 4 (release) ──→ Task 5 (prune)
                                                                                      
Task 6 (design) — independent; can run parallel to Tasks 2–5 (investigator survey is read-only)
```

Tasks 2–3 must precede Task 4 so `main` receives the cleaned state. Task 5 after Task 4 so merge status is judged against a promoted main. Task 6 has no code dependency on anything — dispatch its Step 1 survey early if parallelizing.

## Rollback Notes

- Task 2: single revert commit (`git revert <hash>`).
- Task 4 before push: `git checkout nightly; git tag -d v3.2.0; git branch -f main origin/main`. After push: do not force-push main; fix-forward.
- Task 5: local deletes recoverable via `git reflog`; remote deletes recoverable from any clone that still has the ref — hence the hard gate.
