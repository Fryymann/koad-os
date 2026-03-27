## 1. Codex CLI — Architecture Overview

Codex CLI is OpenAI's **open-source, Rust-built** local coding agent that runs in your terminal. It reads, edits, and executes code in a sandboxed environment with human-in-the-loop approval controls. It ships across three surfaces — **CLI (TUI)**, **IDE extension (VS Code)**, and **Codex App (cloud)** — all sharing the same config layer.

Key architectural traits:

- **Agent loop**: Codex receives a prompt → reads files → proposes edits (via `apply_patch`) → runs shell commands → iterates based on results
- **Sandboxed execution**: Commands run inside constrained environments (file write boundaries + network controls)
- **Approval policies**: `suggest` (ask before everything), `on-request` (ask only for risky ops), `never` (full auto)
- **Models**: Defaults to `gpt-5-codex` family; configurable per-profile or per-agent role
- **Session state**: Threads are resumable; supports `codex resume` to continue prior work

---

## 2. Configuration System — The Five Layers

Codex has a rich, layered configuration system.

### Layer 1: `config.toml` (User + Project)

- **User-level**: `~/.codex/config.toml`
- **Project-level**: `.codex/config.toml` (only loaded for trusted projects)
- Project configs walk from repo root to CWD; closest file wins on conflicts

Key settings:

- `model` — default model (e.g., `"gpt-5-codex"`)
- `model_reasoning_effort` — `"low"` / `"medium"` / `"high"`
- `approval_policy` — `"suggest"` / `"on-request"` / `"never"`
- `sandbox_mode` — `"read-only"` / `"workspace-write"`
- `project_root_markers` — defaults to `[".git"]`, customizable
- `project_doc_fallback_filenames` — add alternate instruction file names
- `project_doc_max_bytes` — defaults to 32KiB

### Layer 2: `AGENTS.md` (Hierarchical Instructions)

This is **the Codex equivalent of `GEMINI.md`** — the primary instruction injection point.

**Discovery order:**

1. **Global**: `~/.codex/AGENTS.md` (or `AGENTS.override.md` if it exists)
2. **Project**: Walks from project root → CWD, loading one file per directory
3. **Merge**: Concatenated root-down; later files override earlier ones

**Override mechanism**: `AGENTS.override.md` in any directory replaces the normal `AGENTS.md` for that level. Fallback filenames are configurable via `project_doc_fallback_filenames` in config.toml.

### Layer 3: Skills (`SKILL.md`)

Skills are **modular, on-demand instruction bundles** — Codex's equivalent of reusable workflow packages.

- A skill = a directory with a `SKILL.md` (must have `name` + `description` in YAML frontmatter)
- Optional subdirs: `scripts/`, `references/`, `assets/`, `agents/openai.yaml`
- **Progressive disclosure**: Only metadata loaded at startup; full instructions loaded when Codex decides to use it
- Discovered from: `$CWD/agents/skills`, repo root `agents/skills`, `~/agents/skills`, `/etc/codex/skills`, and system-bundled

Enable in config:

```toml
[features]
skills = true
```

### Layer 4: MCP Servers

Codex connects to **STDIO MCP servers** (no remote SSE/HTTP yet) via config.toml:

```toml
[mcp.servers."my-server"]
command = "/path/to/server"
args = ["--flag"]
env.MY_VAR = "value"
```

Scoped per-user or per-project. The CLI also has `codex mcp` commands for management.

### Layer 5: Multi-Agents (Experimental)

Codex can spawn **specialized sub-agents in parallel** with distinct models, instructions, and sandbox policies.

Enable:

```toml
[features]
multi_agent = true
```

Define roles:

```toml
[agents]
max_threads = 6
max_depth = 1

[agents.explorer]
description = "Read-only codebase explorer"
config_file = "agents/explorer.toml"

[agents.worker]
description = "Implementation-focused agent"
config_file = "agents/worker.toml"
```

Each role TOML can override `model`, `model_reasoning_effort`, `sandbox_mode`, `developer_instructions`, and even `[mcp_servers.*]`.

### Bonus: Profiles

Named config sets for switching contexts:

```toml
[profiles.deep-review]
model = "gpt-5-pro"
model_reasoning_effort = "high"
approval_policy = "never"
```

Launch with: `codex --profile deep-review`

---

## 3. What KoadOS Already Does (Current State)

From the `agents-os` repo, we already have:

- **Root `AGENTS.md`** with KOAD startup order for Codex sessions
- **Per-app `AGENTS.md`** (e.g., `apps/skylinks-wordpress-site/AGENTS.md`) with:
    - Boot hook: `koad boot --project --task <task_id>`
    - Identity injection from CLI output
    - Auth via `koad auth`
    - Sanctuary Rule enforcement
    - Saveup Protocol at session end
- **Session logging** in `agents/sessions/SESSION_LOG.md`
- **Memory/learnings** in `.koad/memory/LEARNINGS.md`

For **Gemini**, the equivalent uses `GEMINI.md` files for context and `~/.gemini/settings.json` / `.gemini/settings.json` for config + MCP.

---

## 4. Parity Map: Gemini ↔ Codex for KoadOS

| **KoadOS Capability** | **Gemini Implementation** | **Codex Implementation** |
| --- | --- | --- |
| Global agent instructions | `~/.gemini/GEMINI.md` | `~/.codex/AGENTS.md` |
| Project instructions | `.gemini/GEMINI.md` | `AGENTS.md` (repo root + nested) |
| User config | `~/.gemini/settings.json` | `~/.codex/config.toml` |
| Project config | `.gemini/settings.json` | `.codex/config.toml` |
| MCP servers | `settings.json` → `mcpServers` | `config.toml` → `[mcp.servers.*]` |
| Boot hook | In [GEMINI.md](http://GEMINI.md) | In [AGENTS.md](http://AGENTS.md) ✅ (already done) |
| Sanctuary Rule | In [GEMINI.md](http://GEMINI.md) | In [AGENTS.md](http://AGENTS.md) ✅ (already done) |
| Saveup Protocol | In [GEMINI.md](http://GEMINI.md) | In [AGENTS.md](http://AGENTS.md) ✅ (already done) |
| Reusable workflows | Extensions | Skills (`SKILL.md` in `agents/skills/`) |
| Multi-agent delegation | Isolated agents (manual) | Native `[agents.*]` roles |
| Named profiles | N/A | `[profiles.*]` in config.toml |
| Override mechanism | N/A | `AGENTS.override.md` per-directory |

---

## 5. Action Plan — What to Build for Codex Parity

### 5A. Global Codex Config (`~/.codex/config.toml`)

Create a KoadOS-aware global config:

```toml
model = "gpt-5-codex"
model_reasoning_effort = "medium"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
project_root_markers = [".git", ".koad"]
project_doc_max_bytes = 65536

[features]
skills = true
multi_agent = true
```

Adding `.koad` to `project_root_markers` means Codex will recognize your KoadOS workspace root automatically.

### 5B. Global [AGENTS.md](http://AGENTS.md) (`~/.codex/AGENTS.md`)

This is the **Codex equivalent of the global [GEMINI.md](http://GEMINI.md)** — shared identity + protocols across all repos:

```markdown
# KoadOS Global Agent Contract

## Identity
You are operating under the KoadOS framework. The human principal is the Admiral (Ian Deans).

## Prime Directives
- Simplicity over complexity
- Plan before build
- Programmatic-first communication
- Sanctuary Rule: Developer agents only touch project files & docs
- Never edit ~/.koad-os/ or config/

## Boot Protocol
Before substantial work, run: koad boot --project --task <task_id>
Use the task ID from the prompt or infer from context.

## Authentication
Use koad auth for directory-aware credential selection.

## Saveup Protocol
When session is complete, run:
koad skill run global/saveup.py -- --summary "<desc>" --scope "<scope>" --fact "<fact>" --learning "<lesson>"

## Communication
All async delegation occurs via the Koad Stream (Notion).
```

### 5C. KoadOS Skills (`agents/skills/`)

Convert your existing `koad skill` commands into native Codex skills for auto-discovery:

**`~/agents/skills/saveup/SKILL.md`:**

```markdown
---
name: koad-saveup
description: Run the KoadOS saveup protocol to persist session learnings, facts, and summaries to memory.
---

When the session is complete or the user requests a saveup:

1. Summarize the work done in this session
2. Identify key facts and learnings
3. Run: koad skill run global/saveup.py -- --summary "<summary>" --scope "<scope>" --fact "<fact>" --learning "<learning>"
4. Confirm the saveup completed successfully
```

**`~/agents/skills/boot/SKILL.md`:**

```markdown
---
name: koad-boot
description: Initialize a KoadOS project context for the current workspace. Run at session start.
---

1. Run koad boot --project --task <task_id> immediately
2. Parse the CLI output for identity/persona configuration
3. Apply the recommended model and scope from boot output
4. Confirm initialization before proceeding with work
```

### 5D. Multi-Agent Roles for KoadOS

Define KoadOS-native agent roles in your project `.codex/config.toml`:

```toml
[agents]
max_threads = 4
max_depth = 1

[agents.explorer]
description = "Read-only codebase exploration for KoadOS projects. Maps code paths before changes."
config_file = ".codex/agents/explorer.toml"

[agents.worker]
description = "Implementation agent for KoadOS project tasks. Respects Sanctuary Rule."
config_file = ".codex/agents/worker.toml"
```

**`.codex/agents/explorer.toml`:**

```toml
model = "gpt-5-codex"
model_reasoning_effort = "medium"
sandbox_mode = "read-only"
developer_instructions = """
You are a KoadOS explorer agent. Read-only. Map code paths, trace execution, cite files.
Never propose edits. Report findings to the parent agent.
"""
```

**`.codex/agents/worker.toml`:**

```toml
model = "gpt-5-codex"
model_reasoning_effort = "medium"
developer_instructions = """
You are a KoadOS worker agent. Own the fix once the issue is understood.
Make the smallest defensible change. Respect the Sanctuary Rule.
Never edit ~/.koad-os/ or config/.
"""
```

### 5E. KoadOS Profile

Add a koad profile for switching into full KoadOS mode:

```toml
[profiles.koad]
model = "gpt-5-codex"
model_reasoning_effort = "high"
approval_policy = "on-request"
```

Launch with: `codex --profile koad`

### 5F. Agent-Agnostic Abstraction

The key insight for true agent-agnostic KoadOS: **your `koad boot` / `koad auth` / `koad skill` CLI commands are already the abstraction layer**. Both [GEMINI.md](http://GEMINI.md) and [AGENTS.md](http://AGENTS.md) simply invoke the same `koad` CLI commands. The only delta is:

- **File naming**: `GEMINI.md` → `AGENTS.md`
- **Config format**: JSON (`settings.json`) → TOML (`config.toml`)
- **MCP syntax**: Slightly different key structure
- **New capabilities**: Codex gives you native Skills and Multi-agents that Gemini doesn't have natively

You could have `koad` generate both files from a single canonical source (e.g., `koad init --agent codex` vs `koad init --agent gemini`) to keep them in sync.

---

## 6. Summary

Codex CLI is architecturally more configurable than Gemini CLI for this use case — it has native multi-agent roles, skills with progressive disclosure, profiles, and per-directory override files. The KoadOS boot hook pattern already built for Codex's `AGENTS.md` is solid. The main gaps to close are:

1. **Global `~/.codex/AGENTS.md`** with KoadOS identity + protocols (matching what global [GEMINI.md](http://GEMINI.md) does)
2. **Global `~/.codex/config.toml`** with KoadOS-optimized defaults + `.koad` as a project root marker
3. **Native Codex Skills** wrapping `koad skill` commands for auto-discovery
4. **Multi-agent role definitions** for explorer/worker patterns
5. **A `koad init --agent codex` command** that scaffolds all of the above from canonical KoadOS config

The existing `koad` CLI is already the agent-agnostic bridge — it just needs the Codex-specific wiring to match Gemini's.

---

## 7. Codex Strengths & Weaknesses Profile

This section profiles where Codex shines and where it falls short, based on community usage reports, benchmark data, and real-world comparisons with Claude Code and Gemini CLI. Use this to inform KoadOS task routing.

### Where Codex Excels

- **Speed & throughput** — Codex is consistently reported as the fastest CLI agent. It generates code quickly, iterates rapidly, and wastes fewer tokens on preamble or status updates. In speed-focused workflows, it outpaces both Claude Code and Gemini CLI.
- **Backend debugging & bug fixes** — Multiple developers report Codex solving Redis/Celery bugs, tracing backend failures, and patching issues that other tools struggled with for days. GPT-5 in Codex is particularly strong at diagnosing root causes in server-side code.
- **Parallel task execution** — Codex's native multi-agent and cloud task systems allow 5+ agents working simultaneously on different parts of a codebase. No other CLI tool has this built-in at the same level.
- **Refactoring & migrations** — OpenAI's own internal teams use Codex daily for large-scale refactors, framework migrations, and cross-file renames. The sandbox + approval flow makes these safer.
- **Instruction adherence** — GPT-5-Codex was specifically trained via reinforcement learning on real coding tasks to follow instructions precisely and match human PR style/preferences. It stops less often to ask unnecessary questions.
- **Codebase exploration** — The `explorer` agent role in read-only mode excels at mapping unfamiliar codebases, tracing execution paths, and answering architectural questions.
- **Simple-to-medium feature implementation** — For features under ~500 lines, Codex reliably produces working code with fewer iterations than competitors.
- **CI/CD and automation** — `codex exec` (non-interactive mode), GitHub Actions integration, and the Codex SDK make it the strongest option for scripted/automated coding workflows.
- **Cost efficiency** — Bundled with ChatGPT Plus ($20/mo), and with GPT-5-Codex-Mini offering 4x more usage, Codex delivers strong value per productive hour.

### Where Codex Struggles

- **Complex greenfield projects** — When building complex systems from scratch, Codex tends to go "bull in a china shop." Claude Code is widely preferred for from-scratch architecture where deep planning matters.
- **Understanding existing large codebases** — Despite strong exploration features, Codex's context handling in very large codebases (350k+ LOC) lags behind Claude Code, which handles context more carefully and maintains coherence longer.
- **Context pollution in long sessions** — Extended multi-turn sessions can lead to context rot where Codex starts making inconsistent or contradictory changes. Claude Code handles long sessions more gracefully.
- **Frontend/UI precision** — Community consensus: "Claude outdid GPT-5 in frontend implementation." For pixel-perfect UI work, design-to-code, and frontend polish, Claude Code produces higher quality output.
- **Production code quality** — Code produced by Codex sometimes needs more manual cleanup. Claude Code is rated higher for clean abstractions, elegant solutions, and production-grade code quality (Claude 9.1/10 vs Codex ~7.8/10 in quality benchmarks).
- **No network access in sandbox** — Codex cannot reach the internet from its execution sandbox (by design for security). This blocks dependency installation, API testing, and package updates during automated runs.
- **Rate limit sensitivity** — Heavy users report hitting 5-hour usage windows. The GPT-5-Codex-Mini auto-switch at 90% helps, but sustained heavy usage can still bottleneck.
- **Occasional regressions** — Some users report model quality fluctuations after updates. Codex's rapid release cycle means occasional regressions in specific task categories.

### Head-to-Head: Codex vs Claude Code vs Gemini CLI

| **Dimension** | **Codex CLI** | **Claude Code** | **Gemini CLI** |
| --- | --- | --- | --- |
| Speed | **Fastest** | Moderate | Fast |
| Code quality | Good (7.8/10) | **Best** (9.1/10) | Good (7.8/10) |
| Error handling | Good (7.5/10) | **Best** (8.8/10) | Good (7.5/10) |
| Context window | 400K tokens | 200K tokens | **1M tokens** |
| Backend/debugging | **Strong** | Strong | Moderate |
| Frontend/UI | Moderate | **Strong** | Moderate |
| Greenfield architecture | Moderate | **Strong** | Moderate |
| Parallel agents | **Native** | Subagents | Manual |
| Refactoring at scale | **Strong** | Strong | Moderate |
| Automation/CI | **Best** | Good | Good |
| Long-context reasoning | Good | Good | **Best** |
| Free tier | Limited trial | None | **60 req/min free** |
| Cost (top tier) | $200/mo (Pro) | $200/mo (Max) | Pay-per-use / free |

---

## 8. Available Models & When to Use Each

Codex supports multiple models, each optimized for different workloads. Here's the current lineup and guidance:

### GPT-5-Codex (Default)

- **What it is**: The flagship Codex model. GPT-5 variant specifically trained via RL on real-world coding tasks.
- **Context window**: 400K tokens
- **Reasoning effort**: Configurable (low / medium / high)
- **API pricing**: $1.25 / $10.00 per 1M tokens (input/output)
- **Best for**: General coding tasks, feature implementation, debugging, refactoring, code review
- **SWE-Bench**: ~55-57% (competitive with Claude Sonnet)
- **When to use**: This should be your default for most KoadOS tasks. Strong all-rounder.

### GPT-5.3-Codex

- **What it is**: Updated Codex model (Feb 2026) with improved performance.
- **Best for**: Complex multi-step tasks, PR reviews, multi-agent orchestration
- **SWE-Bench Pro**: 56.8%
- **When to use**: When you need stronger reasoning for complex tasks. Good for the `reviewer` and `explorer` agent roles.

### GPT-5.4

- **What it is**: The latest frontier model (not Codex-specific but usable in Codex CLI).
- **SWE-Bench Pro**: 57.7%
- **Best for**: Hardest engineering problems, deep architectural decisions, complex debugging
- **When to use**: Reserve for high-stakes tasks where maximum reasoning quality matters. Set via `codex -m gpt-5.4` or in profiles.

### GPT-5-Codex-Mini

- **What it is**: Smaller, more cost-effective variant. Up to 4x more usage within subscription limits.
- **Best for**: Simple bug fixes, straightforward feature additions, code formatting, test generation, routine tasks
- **When to use**: For high-volume, lower-complexity work. Codex auto-offers this when you hit 90% of your 5-hour usage limit. Great for the `worker` agent role on simple tasks.

### GPT-5-Codex-Spark

- **What it is**: Lightweight, fast model designed for quick responses.
- **Best for**: Read-only exploration, fast Q&A about code, metadata gathering, quick checks
- **When to use**: Ideal for the `explorer` agent role where speed matters more than deep reasoning.

### GPT-4.1

- **What it is**: Previous-gen model, still available. Larger context window (1M tokens).
- **API pricing**: $2.00 / $8.00 per 1M tokens (input/output)
- **Best for**: Tasks requiring massive context (very large codebases), cost-sensitive batch work
- **When to use**: When you need the 1M token context window, or for cost optimization on simpler tasks. Not as strong on coding benchmarks as GPT-5-Codex.

### Model Selection Strategy for KoadOS

| **KoadOS Task Type** | **Recommended Model** | **Reasoning Effort** |
| --- | --- | --- |
| Quick exploration / read-only | GPT-5-Codex-Spark | medium |
| Simple bug fixes / formatting | GPT-5-Codex-Mini | low |
| Feature implementation | GPT-5-Codex | medium |
| Backend debugging | GPT-5-Codex | high |
| Refactoring / migration | GPT-5.3-Codex | high |
| PR review / security audit | GPT-5.3-Codex or GPT-5.4 | high |
| Complex architecture decisions | GPT-5.4 | high |
| Large codebase navigation (1M ctx) | GPT-4.1 | medium |
| Batch/CI automation | GPT-5-Codex-Mini | low |

This maps directly into `koad boot --task <id>` — the boot command can recommend the optimal model based on task type, which Codex then picks up from the profile or `config.toml`.

---

## 9. KoadOS Agent Routing Recommendation

Based on the strengths/weaknesses profile above, here's how KoadOS should route tasks across agents:

<aside>
🎯

**Route to Codex when**: Speed matters, backend/debugging work, refactoring, parallel task fan-out, CI/CD automation, codebase exploration, simple-to-medium features, batch operations

</aside>

<aside>
🎯

**Route to Claude Code when**: Complex greenfield architecture, frontend/UI precision, production-grade code quality, long multi-turn planning sessions, tasks requiring deep contextual coherence across large existing codebases

</aside>

<aside>
🎯

**Route to Gemini CLI when**: Very large context tasks (1M tokens), multimodal reasoning (images/diagrams → code), Google Cloud integration, cost-sensitive or free-tier exploration, tasks requiring long-context reasoning

</aside>

This three-agent routing strategy lets `koad boot` select the right tool for each task, maximizing the strengths of each while avoiding their respective weak spots.