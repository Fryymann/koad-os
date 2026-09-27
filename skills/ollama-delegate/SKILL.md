---
name: ollama-delegate
description: Route a task to the correct local Ollama model using the KoadOS delegation matrix. Checks live VRAM state via `ollama ps`, applies the decision tree, enforces concurrency limits, and emits a delegation packet. Use when Clyde or Hermes is about to hand off a bounded subtask to a local Ollama model.
license: MIT
compatibility: Requires Ollama (`ollama` CLI and daemon) with the routed models pulled; tuned for a 12 GB VRAM GPU.
metadata:
  author: koados
  version: "1.1.0"
---

# Ollama Delegation Skill

**Announce:** "Using ollama-delegate to select a model and emit a delegation packet."

## Step 1: Classify the task

Answer these questions in order — **first match wins:**

### Branch A — Doc update / spec-driven edit?
Is the deliverable a documentation file, README, docstring pass, or spec-following text edit?
- **Yes, and strict JSON / schema output required** → `nemotron-mini:4b` *(≤4K ctx only)*
- **Yes, and tight extraction or classification** → `granite3.3:2b`
- **Yes, otherwise** → `phi4-mini:3.8b`

### Branch B — Needs tool calls or structured JSON? (non-doc task)
- **No tool calls, and prose / summary / multilingual / vision** → `gemma4:e4b` *(no tool calls — Jinja bugs)*
- **No tool calls, general / mixed planning** → `qwen3.5:9b`
- **Yes, tool calls needed:**
  - Code-heavy?
    - **Bounded / single-file** → `qwen2.5-coder:7b`
    - **Multi-file refactor or architectural reasoning** → `qwen3:14b` *(solo, thinking mode ON)*
    - **Unclear scope** → `qwen2.5-coder:7b`, fallback `qwen3.5:9b`
  - **Not code-heavy** → `qwen3.5:9b`

### Confirm the model is installed

```bash
ollama list
```

If the selected model is not listed, do not pull it implicitly (downloads are several GB). Take the next model on the escalation path in Step 4 that *is* installed, and mention the substitution in the delegation packet's objective.

---

## Step 2: Check live VRAM state

Run:
```bash
ollama ps
```

If `ollama ps` fails (command not found, daemon not running, permission error): **stop and report the error to Clyde. Do not assume VRAM is clear. Do not proceed with delegation.**

Apply these concurrency rules before proceeding:

**If selected model is Large (≥9GB): `qwen3:14b` or `gemma4:e4b`**
- Any other model loaded → run `ollama stop <model>`, wait 5s, re-check — **except:** a sub-3GB Small model (`granite3.3:2b`, `phi4-mini:3.8b`) may co-run with `qwen3:14b` at ≤8K ctx each (see `co_runnable_with` table). `gemma4:e4b` is always solo.
- For `gemma4:e4b`: proceed only when `ollama ps` shows no other models.
- For `qwen3:14b`: proceed when only a sub-3GB Small is loaded, or no models are loaded.

**If selected model is Mid (~7GB): `qwen3.5:9b`**
- Another Mid or Large is loaded → stop it first.
- A Small model loaded → OK to proceed.

**If selected model is Small (≤5GB): `qwen2.5-coder:7b` (~4.7GB, quantized), `phi4-mini:3.8b`, `granite3.3:2b`, `nemotron-mini:4b`**
- Two models already loaded → stop one before proceeding.
- Zero or one model loaded → OK.

**Hard limit:** Max 2 concurrent models. Never exceed it.

---

## Step 3: Emit delegation packet

Output this JSON with actuals filled in:

```json
{
  "model": "<selected-model>",
  "task_id": "clyde-<YYYY-MM-DD>-<NNN>",
  "objective": "<one sentence: what to produce and what done looks like>",
  "context_budget_tokens": 4096,
  "max_output_tokens": 1024,
  "concurrency_class": "<small|mid|large>",
  "co_runnable_with": ["<models safe to pair — see reference below>"],
  "timeout_s": 120,
  "acceptance": [
    "<measurable criterion 1>",
    "<measurable criterion 2>"
  ]
}
```

**`task_id`:** Use date of current session for `<YYYY-MM-DD>`. Increment `<NNN>` from `001` each time you emit a packet in the same session (e.g., `clyde-2026-05-02-001`, `clyde-2026-05-02-002`).

**`context_budget_tokens`:** Default 4096. Use 8192 only when the task genuinely needs it. Never exceed 16384 unless the model is solo. `nemotron-mini:4b` caps at 4096 — never raise it.

**`co_runnable_with` reference:**
| Model | Safe pairs |
|-------|-----------|
| `qwen2.5-coder:7b` | `qwen3.5:9b`, `phi4-mini:3.8b`, `granite3.3:2b`, `nemotron-mini:4b` |
| `qwen3.5:9b` | `qwen2.5-coder:7b`, `granite3.3:2b`, `nemotron-mini:4b` |
| `qwen3:14b` | `granite3.3:2b` *(at ≤8K ctx each)* |
| `gemma4:e4b` | *(none — solo only)* |
| `phi4-mini:3.8b` | `qwen2.5-coder:7b`, `qwen3:14b`, `granite3.3:2b`, `nemotron-mini:4b` |
| `granite3.3:2b` | all models |
| `nemotron-mini:4b` | `qwen2.5-coder:7b`, `phi4-mini:3.8b`, `granite3.3:2b` |

---

## Step 4: Escalation on acceptance failure

If a model fails its acceptance criteria, escalate using the path below.

> ⚠️ **Never escalate in parallel. Always try one model, wait for result, then escalate if needed.**


**Code path:**
1. `qwen2.5-coder:7b` → 2. `qwen3.5:9b` → 3. `qwen3:14b` *(solo, thinking ON)* → 4. Clyde (hosted) with distilled failure summary

**Doc / spec-follow path:**
1. `granite3.3:2b` → 2. `phi4-mini:3.8b` → 3. `qwen3.5:9b` → 4. `qwen3:14b` *(solo)* → 5. Clyde (hosted) with distilled summary

**Strict-JSON path:**
1. `nemotron-mini:4b` → 2. `granite3.3:2b` → 3. `phi4-mini:3.8b` → 4. `qwen3.5:9b` → 5. Clyde (hosted)

**When escalating to hosted Clyde:** distill the failed output + gap into a summary. Do NOT resend the original full context.
