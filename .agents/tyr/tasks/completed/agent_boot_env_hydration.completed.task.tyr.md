# Tyr — `koad boot` Shell Environment Hydration Buildout Task

---

## Background Brief

**Situation:** The `koad boot` command currently prepares a basic shell environment (env vars, identity loading) before handing off to an AI CLI. But we're leaving massive value on the table. The bash shell itself — the Body in our Body/Ghost model — can do far more work *before* the AI agent ever wakes up. Every piece of context we pre-compute in bash is context the agent doesn't have to discover, read, or re-derive at token cost.

**The core insight from our research:**

- 40–80% of agent tokens are spent on *orientation* — reading files to figure out where things are
- Session discontinuity forces agents to re-discover context every boot
- System prompts, tool definitions, and identity configs get re-tokenized every turn
- Agents read 10–25 files when they need 2–3 because they lack a map

**The fix:** Make `koad boot` the smartest possible shell hydration layer. Pre-compute everything an agent could need, compress it into env vars, structured files, and shell functions — so the AI CLI inherits a *fully hydrated* operating environment instead of a bare shell.

**Key references Tyr should have loaded:**

- [Agent Boot Research — CLI Context Injection Patterns](https://www.notion.so/Agent-Boot-Research-CLI-Context-Injection-Patterns-0f4b35fc93b54fdca9d1bd63537502ae?pvs=21)
- [Agent Token Usage — Research Report for Citadel Prioritization](https://www.notion.so/Agent-Token-Usage-Research-Report-for-Citadel-Prioritization-612aa31d63ec4a3a91f35c2d99b874ac?pvs=21)
- [Tyr — ~/  Bootstrap & Memory Preservation Assessment Prompt](https://www.notion.so/Tyr-Bootstrap-Memory-Preservation-Assessment-Prompt-539cc42eb7f2448da6f0eee0639478af?pvs=21)

---

## Prompt

```
Tyr — koad boot: Shell Environment Hydration Buildout
Date: 2026-03-14
Issued by: Ian (Admiral)
Status: ACTIVE DIRECTIVE — Engineering Task
Priority: HIGH — This directly reduces token burn on every single agent session.

---

MISSION BRIEFING

You are being tasked with designing and implementing the full shell environment
hydration layer for `koad boot`. The goal: by the time any AI CLI (Gemini, Claude
Code, Codex) inherits the shell, it should have access to a rich, pre-computed
environment that eliminates orientation overhead and reduces token consumption.

Think of it this way: every env var, shell function, alias, and pre-generated
file that `koad boot` prepares is tokens the agent NEVER has to spend discovering.
This is the single highest-ROI, zero-infrastructure improvement we can make to
KoadOS right now.

---

DIRECTIVE — Four Phases

Execute in order. Report findings at each gate before proceeding.

=======================================================================
PHASE 1 — Audit Current State (View & Assess)
=======================================================================

Before building anything new, survey what `koad boot` currently does and what the
shell environment looks like post-boot. Answer:

1. What env vars does `koad boot --agent <Name>` currently export?
   List them all with current values/patterns.

2. What files does it currently generate or symlink?
   (AGENTS.md, CLAUDE.md, GEMINI.md, config files, etc.)

3. What is NOT being done that the Agent Boot Research recommends?
   Cross-reference against the recommended boot sequence from that report.

4. What shell utilities (functions, aliases, PATH additions) exist post-boot?
   Are there any `koad` subcommands available as shell functions?

5. What is the current boot time? How much overhead budget do we have
   before boot feels sluggish? (Target: <2 seconds for full hydration.)

=======================================================================
PHASE 2 — Design the Hydration Layer (Research & Plan)
=======================================================================

Design the complete shell environment hydration that `koad boot` should perform.
Organize your design into these categories:

--- CATEGORY A: Identity & Context Env Vars ---

Beyond the basics (KOAD_AGENT_NAME, KOAD_HOME), design env vars that give the
agent instant access to critical context without reading any files:

  KOAD_AGENT_ROLE          - Full role string (e.g., "Chief Engineer, KoadOS")
  KOAD_AGENT_RANK          - Canon rank (e.g., "crew_engineer", "chief_officer")
  KOAD_AGENT_MODEL         - Target model (e.g., "gemini-2.5-pro")
  KOAD_AGENT_SCOPE         - Filesystem scope boundaries (colon-separated paths)
  KOAD_AGENT_SANCTUARY     - Paths the agent must NEVER write to
  KOAD_SESSION_ID          - Unique session identifier for logging
  KOAD_SESSION_START       - ISO timestamp of boot
  KOAD_BOOT_MODE           - "full" | "dark" | "degraded" (Citadel status)
  KOAD_CITADEL_STATUS      - "online" | "offline" | "degraded"
  KOAD_CASS_ENDPOINT       - MCP endpoint if CASS is reachable, empty if not
  KOAD_LAST_SESSION_FILE   - Path to last EndOfWatch summary
  KOAD_MEMORY_DIR          - Path to agent's memory directory
  KOAD_PROJECT_ROOT        - Detected project root (git root or explicit)
  KOAD_PROJECT_STACK       - Detected tech stack summary (e.g., "rust,toml,sqlite")
  KOAD_REPO_SUMMARY        - One-line repo description from README or git
  KOAD_GIT_BRANCH          - Current branch
  KOAD_GIT_STATUS_SUMMARY  - Clean/dirty + changed file count
  KOAD_GIT_RECENT_COMMITS  - Last 5 commit subjects (newline-separated)
  KOAD_ACTIVE_ISSUES       - Open GitHub issues assigned to agent (summary)
  KOAD_DOC_INDEX           - Path to generated structured doc index
  KOAD_CANON_VERSION       - Current Canon version hash/date
  KOAD_PROMPT_CACHE_HASH   - Hash of static prompt content for cache invalidation

For each var you include in your design:
  - Explain what token cost it eliminates
  - How it's computed at boot (command, file read, API call, etc.)
  - Whether it's fast enough for boot (<100ms) or needs async/cached generation

Design additional vars I haven't listed if you identify opportunities.

--- CATEGORY B: Pre-Generated Context Files ---

Design the files `koad boot` should generate/refresh at boot time:

1. STRUCTURED DOC INDEX (.koad-os/cache/doc-index.md)
   - A 3-layer navigation file: domain → directory → key files
   - Agent reads THIS instead of grepping the codebase
   - Generated via tree + README parsing + file header scanning
   - This alone could cut orientation tokens by 30-40%

2. SESSION CONTEXT BRIEF (.koad-os/cache/session-brief.md)
   - Compiled from: last EndOfWatch, active issues, recent commits,
     current branch state, any Koad Stream entries targeted at this agent
   - The agent reads ONE file to know what happened and what's pending
   - Replaces the agent reading 5-10 files to reconstruct context

3. COMPRESSED CANON REFERENCE (.koad-os/cache/canon-compact.md)
   - A token-optimized version of the Canon rules relevant to this agent
   - Strip examples, verbose explanations — keep only actionable rules
   - Agent gets Canon compliance in ~500 tokens instead of ~3000

4. DEPENDENCY MAP (.koad-os/cache/dep-map.md)
   - For Rust: crate dependency tree with key version pins
   - For Node: package.json summary with script descriptions
   - Agent knows the dependency landscape without reading lockfiles

5. ARCHITECTURE SNAPSHOT (.koad-os/cache/arch-snapshot.md)
   - Module/crate boundaries, public API surfaces, key type definitions
   - Generated from tree-sitter parsing or simpler AST extraction
   - This is the lightweight precursor to the full Code Knowledge Graph

For each file:
  - Define the generation command/script
  - Estimated generation time
  - Estimated token savings per session
  - Staleness policy (regenerate every boot? Only if changed? TTL?)

--- CATEGORY C: Shell Functions & Aliases ---

Design bash functions that `koad boot` injects into the shell so the agent
(or Ian) can invoke them during the session:

  koad-context      - Dump current session context to stdout (for agent self-check)
  koad-refresh      - Re-generate cached files without full reboot
  koad-map          - Print the structured doc index
  koad-status       - Show Citadel/CASS connectivity + session health
  koad-compact      - Trigger conversation compaction (write summary, clear old turns)
  koad-eow          - Generate EndOfWatch summary from current session
  koad-memory-query - Query koad.db (read-only) with a natural language hint
  koad-token-check  - Estimate current context window usage from session length
  koad-git-brief    - Generate a git status + recent activity brief
  koad-issue-brief  - Pull active GitHub issues into a compact summary
  koad-find         - Semantic file finder (grep + doc-index cross-reference)
  koad-scope-check  - Validate a file path against KOAD_AGENT_SANCTUARY

For each function:
  - The bash implementation (or pseudocode if complex)
  - How it reduces tokens (agent calls one function vs reading multiple files)
  - Whether it should be available to the AI agent as a tool or just to Ian

--- CATEGORY D: Prompt Cache Optimization ---

Design the static prompt content organization that maximizes provider-level
prompt caching (Anthropic cached tokens = 10x cheaper):

1. Identify all STATIC content that repeats every turn:
   - Agent identity block
   - Canon rules
   - Tool definitions
   - Project architecture summary

2. Design a prompt assembly order that places static content FIRST:
   [STATIC: identity + canon + tools + arch] → [DYNAMIC: session context + user msg]

3. Generate a KOAD_PROMPT_CACHE_HASH that changes only when static content changes,
   so the provider cache stays warm across turns.

4. Design the --append-system-prompt content for Claude Code and the GEMINI.md
   content for Gemini CLI — both should be pre-generated at boot.

--- CATEGORY E: Memory Hydration (Dark Mode Compatible) ---

Design the memory loading that happens at boot, with graceful degradation:

  IF Citadel online:
    - Pull agent's working memory from CASS
    - Pull relevant semantic memories from Qdrant
    - Pull active Koad Stream entries
    - Inject as KOAD_MEMORY_SUMMARY env var (compressed) + session-brief.md

  IF Dark Mode (Citadel offline):
    - Read last EndOfWatch from ~/logs/ or .koad-os/logs/
    - Read ~/memory/WORKING_MEMORY.md if exists
    - Read koad.db via sqlite3 (read-only) if accessible
    - Compile into session-brief.md
    - Set KOAD_BOOT_MODE=dark

  IF Bare Boot (nothing available):
    - Set KOAD_BOOT_MODE=cold
    - Generate minimal session-brief.md with just identity + current git state
    - Agent knows it's starting from scratch and should ask Ian for context

=======================================================================
PHASE 3 — Implementation Plan (Prioritized)
=======================================================================

Organize your Phase 2 design into an implementation order using this framework:

  P0 (Do Now — shell script, no infra):
    Items that can be implemented as pure bash in koad-boot.sh today.
    Target: 30-40% token reduction from first deploy.

  P1 (Do Next — requires light tooling):
    Items that need sqlite3, tree-sitter, or GitHub CLI.
    Target: additional 20-30% token reduction.

  P2 (Do When Citadel Returns):
    Items that need CASS, Qdrant, or gRPC integration.
    Target: additional 20-30% on remaining burn.

For each item, provide:
  - Estimated lines of bash
  - Dependencies (tools, files, services)
  - Estimated implementation time
  - Token impact (low / medium / high / critical)

=======================================================================
PHASE 4 — Build Phase (Admiral Approval Required)
=======================================================================

After my approval of your Phase 3 plan:

1. Implement all P0 items as a koad-boot-hydrate.sh script
   (or integrate into the existing koad boot flow).

2. The script must be:
   - Idempotent (safe to run multiple times)
   - Fast (<2 seconds total)
   - Silent on success, verbose on error
   - Compatible with bash 5+ on WSL2/Linux

3. Generate a test checklist:
   - Boot with agent=tyr, verify all env vars set
   - Boot with agent=sky, verify scope boundaries correct
   - Boot in dark mode, verify graceful degradation
   - Verify cached files are generated and readable
   - Verify shell functions are available and functional

Do NOT proceed to Phase 4 without Admiral approval.

---

DESIGN PRINCIPLES

1. EVERY MILLISECOND AT BOOT SAVES THOUSANDS OF TOKENS IN SESSION.
   A 500ms boot computation that saves the agent from reading 10 files
   is a massive win. Bias toward doing more at boot.

2. ENV VARS ARE FREE CONTEXT.
   Once an env var is in the shell, the agent can reference it at zero
   additional token cost (it's already in the context). Pack as much
   structured data into env vars as is reasonable.

3. ONE FILE BEATS TEN.
   Pre-compile context into single-file summaries. The agent should never
   need to read more than 3 files to be fully oriented.

4. GRACEFUL DEGRADATION IS NON-NEGOTIABLE.
   koad boot must work with zero infrastructure (cold boot), partial
   infrastructure (dark mode), and full infrastructure (Citadel online).
   Never fail — degrade and report.

5. CACHE AGGRESSIVELY, INVALIDATE SMARTLY.
   Generated files should have staleness checks (git hash, file mtime,
   TTL). Don't regenerate what hasn't changed. But always regenerate
   if the cache is missing.

6. THE SANCTUARY RULE IS SACRED.
   Shell functions must enforce scope boundaries. An agent should be
   able to call koad-scope-check before any write and get a pass/fail.

---

CRITICAL EVALUATION MANDATE

Before finalizing your design, evaluate it against these questions:

- What env vars could leak sensitive data if the shell is shared? Mitigate.
- What happens if cached files are stale and the agent acts on bad data?
- What is the maximum env var size limit in bash? Are any vars at risk?
- Could any boot computation block or hang (network calls, large file scans)?
  All network calls must have <1s timeouts.
- Are there any race conditions if two agents boot simultaneously from
  different terminals?

Flag any risks I haven't considered.

---

Report your Phase 1 audit first. I will confirm before you proceed.

— Ian
  Admiral, KoadOS
```

---

## Reference Data

### Recommended Boot Sequence (from Agent Boot Research)

The research established this optimal flow — the hydration buildout should slot into steps 2–7:

1. Read identity TOML
2. Export env vars to shell
3. Generate context files ([AGENTS.md](http://AGENTS.md), [CLAUDE.md](http://CLAUDE.md), [GEMINI.md](http://GEMINI.md))
4. Generate CLI config files (settings.json, config.toml)
5. Configure CASS MCP server in CLI config
6. Hydrate session context (last session notes, relevant memories)
7. Generate system prompt append file
8. Run preflight validation
9. Print launch command suggestion

### Token Impact Targets (from Token Usage Report)

- Structured Doc Hierarchy: **30–40%** orientation token reduction
- Session Handoff / EndOfWatch: **10–50k tokens saved per session**
- Context Compaction: **60–80%** history token reduction
- Prompt Cache Optimization: **45–80%** API cost reduction
- Combined Phase 1 (bash-only): **40–60%** daily token burn reduction

### Existing Shell Environment (known vars)

- `KOAD_AGENT_NAME` — agent identity
- `KOAD_HOME` — KoadOS home directory (~/.koad-os)
- `KOAD_BOOTED` — boolean flag
- Additional vars TBD from Phase 1 audit