## Purpose

Reusable instruction prompt for any KoadOS agent (Tyr, Vigil, etc.) to perform a deep sweep of the `koad-os` repo, classify every file, and produce a structured intel report.

---

## Prompt — Copy & Hand to Agent

<aside>
📋

**Copy everything below this callout and paste it as the agent's instruction input.**

</aside>

---

### MISSION: KoadOS Repo Cleanup Sweep

**Authority:** Dood Authority — Admiral Override

**Output:** `.koad-os/docs/cleanup/cleanup_sweep_3-11-2026.md`

**Mode:** Read-only analysis. Do NOT delete, move, or modify any files. Report only.

---

### 1 — Objective

The `koad-os` repo has accumulated AI-generated bloat — redundant drafts, orphaned configs, stale docs, and duplicated logic. Your job is to perform a **full, file-by-file deep sweep** of the repo and produce a structured intel report classifying every file into one of three tiers:

- **🟢 KEEP** — Essential to the system. Actively imported, referenced, or required by runtime, build, or CI.
- **🟡 QUESTIONABLE** — Purpose is unclear, potentially redundant, or only partially used. Needs human review before removal.
- **🔴 SAFE TO REMOVE** — Confirmed dead weight. Not imported, not referenced, not required by any active path. Removal would have zero impact on functionality.

---

### 2 — Sweep Protocol

Execute the following passes **in order**:

**Pass 1 — Structural Map**

- Generate a full directory tree of the repo.
- Note total file count, directory depth, and any abnormally large directories.

**Pass 2 — Import & Reference Graph**

- For every `.ts`, `.js`, `.mjs`, `.cjs` file: trace all `import`/`require` statements.
- Build a dependency graph. Flag any file that is **never imported by any other file** and is **not an entrypoint** (e.g., `index.ts`, `main.ts`, CLI entry, or explicitly configured in `package.json` / `tsconfig`).

**Pass 3 — Config & Infra Audit**

- Check all config files (`.env*`, `tsconfig*`, `package.json`, `Dockerfile`, `.github/`, `.koad-os/`, etc.).
- Flag duplicates, conflicting configs, or configs that reference paths/files that no longer exist.

**Pass 4 — Documentation & Markdown Audit**

- Inventory all `.md` files.
- Classify each as: **active doc** (referenced or linked), **orphaned doc** (not linked from anywhere), or **AI-generated draft** (signs of boilerplate, generic phrasing, duplicated content across multiple docs).
- Pay special attention to docs that appear to be multiple versions of the same content.

**Pass 5 — Test & Script Audit**

- Identify all test files and utility scripts.
- Flag tests that reference removed or renamed modules.
- Flag scripts that duplicate functionality already handled by other tooling.

**Pass 6 — AI Bloat Detection**

- Look for telltale signs of AI-generated clutter:
    - Multiple files with near-identical content or structure.
    - Files with generic names like `utils2.ts`, `helper_v2.ts`, `draft-*.md`, `temp-*`.
    - Commented-out blocks longer than 20 lines.
    - Files that appear to be "exploration" or "brainstorming" artifacts with no integration into the codebase.
    - README or doc files that repeat information already present elsewhere.

---

### 3 — Report Format

Write the report to `.koad-os/docs/cleanup/cleanup_sweep_3-11-2026.md` using the following structure:

```markdown
# KoadOS Repo Cleanup Sweep — 2026-03-11

## Executive Summary
- Total files scanned: [N]
- 🟢 KEEP: [N]
- 🟡 QUESTIONABLE: [N]
- 🔴 SAFE TO REMOVE: [N]
- Estimated bloat percentage: [N]%

## High-Priority Removals
<!-- Top 10-15 files/dirs that are the most obviously dead and would clean up the most clutter -->
| File / Directory | Tier | Reason |
|---|---|---|
| ... | 🔴 | ... |

## Full File Classification

### 🟢 KEEP
| File | Role / Justification |
|---|---|
| ... | ... |

### 🟡 QUESTIONABLE
| File | Concern | Suggested Action |
|---|---|---|
| ... | ... | Review / merge / clarify |

### 🔴 SAFE TO REMOVE
| File | Reason |
|---|---|
| ... | Not imported, orphaned draft, duplicate of X |

## Dependency Graph Notes
<!-- Any interesting findings: circular deps, deeply nested unused chains, etc. -->

## Config Conflicts
<!-- Any conflicting or stale config entries -->

## AI Bloat Hotspots
<!-- Directories or patterns where AI-generated clutter is concentrated -->

## Recommendations
<!-- Prioritized next steps for Ian/Dood to approve -->
```

---

### 4 — Rules of Engagement

1. **Read-only.** Do not delete, rename, or modify any file. This is an intel mission.
2. **Be specific.** Every classification must include the file path and a concrete reason. No vague "might not be needed."
3. **When in doubt, classify as 🟡 QUESTIONABLE** — never 🔴. Ian will make the final call.
4. **Trace before you judge.** Before marking anything 🔴, confirm it is not imported, not referenced in configs, and not an entrypoint.
5. **Group related removals.** If 5 files are all part of the same abandoned feature, group them and explain the feature.
6. **No hallucination.** If you cannot determine a file's purpose, say so explicitly. Do not fabricate justifications.
7. **Complete the KSRP 7-pass self-review** on the final report before delivering.

---

### 5 — Delivery

- Write the completed report to: `.koad-os/docs/cleanup/cleanup_sweep_3-11-2026.md`
- Summarize your top findings in chat after delivery.
- Await Dood approval before any follow-up action.