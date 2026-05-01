<aside>
⚡

**Handoff doc for Tyr, Clyde, and Citadel Jupiter crew.** Spec for integrating four external token-efficiency tools into KoadOS Citadel Jupiter. Researched and authored by Noti, 2026-04-30. Canonical implementation decisions belong to Tyr.

</aside>

---

## Overview

Between March 23 and April 23, 2026, Anthropic identified four root causes of runaway Claude Code quota consumption: **cache misses**, **context bloat**, **wrong model selection**, and **wrong input format**. Community tooling now exists to address each. This spec evaluates four tools against the KoadOS Citadel Jupiter architecture and defines concrete integration paths for each.

**Scope:** Citadel Jupiter only (Desktop / Tyr as captain). Io parity is out of scope for this spec.

**Primary beneficiary:** Clyde (Claude Code). Tyr (Gemini CLI) is unaffected by Claude-specific optimizations unless noted.

---

## Priority Stack

| **Tool** | **Root Cause Addressed** | **Token Impact** | **Effort** | **Priority** |
| --- | --- | --- | --- | --- |
| **rtk** (Rust Token Killer) | Context bloat — CLI output noise | ~89% reduction on CLI output | Low — single binary + hook or boot script | 🔴 **P1 — Do first** |
| **OpenRouter** (`ANTHROPIC_BASE_URL`) | Wrong model / effort | Cost routing — cheap models for simple tasks | Medium — boot env + IDENTITY.toml pattern | 🔴 **P2 — Do second** |
| **caveman** skill | Context bloat — verbose output tokens | ~65–75% output token reduction | Low — skill install | 🟡 **P3 — Do third** |
| **agent-browser** (Vercel Labs) | Wrong input format — browser screenshots | ~82% vs. Playwright MCP (browser tasks only) | Low — MCP add | 🟢 **P4 — Backlog** |

---

## P1 — rtk (Rust Token Killer)

### What It Is

[github.com/rtk-ai/rtk](http://github.com/rtk-ai/rtk) — A single Rust binary CLI proxy that intercepts command output before it reaches Claude's context window. Filters and compresses `cargo`, `git`, `docker`, `pytest`, and other dev commands.

### Measured Compression

| **Command** | **Before** | **After** | **Reduction** |
| --- | --- | --- | --- |
| `cargo test` | ~4,800 tokens | ~11 tokens (`✓ cargo test: 262 passed`) | −99% |
| `git status` | ~119 chars | ~28 chars | −76% |
| `git diff` | ~21,500 tokens | ~1,259 tokens | −94% |
| `git push` | ~200 tokens | ~10 tokens | −95% |
| **Session total (measured)** | ~11.4M tokens / 2 weeks | ~1.2M tokens | **−89%** |

### Why It Matters for Clyde

Clyde runs in Claude Code, which returns raw `BashTool` outputs directly into the context window. Every `cargo test` run, `git status` check, and `docker build` call is a context injection event. This is the single largest source of context bloat in a typical KoadOS coding session. rtk addresses this at the source — before the data ever enters the model's context.

rtk is written in Rust, has zero external runtime dependencies, and ships as a single static binary. It is a natural fit for the KoadOS toolchain.

### Integration Plan

**Option A (Recommended) — Boot script injection**

Add `rtk init -g` to the `koad boot --agent clyde` sequence. This installs the global shell hook so Claude Code auto-rewrites `cargo`, `git`, `docker`, and other supported commands to `rtk` equivalents transparently.

```bash
# In koad boot --agent clyde sequence
rtk init -g   # Installs global hook; no further config needed
```

**Option B — PreToolUse hook**

If boot-level injection is not desirable, register a `PreToolUse` hook that pipes `BashTool` outputs through `rtk filter` before they are returned to Claude:

```json
{
  "hooks": {
    "PostToolUse": [{
      "matcher": "BashTool",
      "hooks": [{
        "type": "command",
        "command": "rtk filter",
        "async": false
      }]
    }]
  }
}
```

**Analytics:** Run `rtk gain` periodically to measure actual token savings per Clyde session.

### Risks & Mitigations

- **Suppressed debug output:** rtk may filter output that is needed during a debugging session. Mitigation: call commands with `rtk --raw <cmd>` to bypass filtering when Clyde is in active debug mode, or when `KOAD_DEBUG=1` is set.
- **Filter lag on new cargo versions:** rtk's `cargo` filter may need updates when Cargo output format changes. Monitor the upstream repo.

---

## P2 — OpenRouter as `ANTHROPIC_BASE_URL`

### What It Is

Swap `ANTHROPIC_BASE_URL` from Anthropic's API endpoint to [openrouter.ai/api](http://openrouter.ai/api). Claude Code sends requests identically — it does not know the difference. OpenRouter routes them to the selected model from a pool of 400+.

### Why It Matters for KoadOS

This addresses **Root Cause 03 (wrong model / effort)** at the infrastructure level. KoadOS already contemplates single-provider model lock per sovereign (see [Single-Provider Skill Variants FR](https://www.notion.so/KoadOS-Feature-Request-Single-Provider-Skill-Variants-per-Sovereign-Agent-b5bfc7d0dd5a480291c487a6bf0c031f?pvs=21)). OpenRouter does not break this — it is an API-layer shim, not a provider change. Clyde still targets `anthropic/claude-opus-4` for complex refactors; OpenRouter simply makes it possible to route cheaper models for lightweight tasks.

**Approximate cost comparison (OpenRouter pricing, April 2026):**

| **Task type** | **Recommended model** | **Relative cost vs. Opus** |
| --- | --- | --- |
| Complex multi-file refactors, architecture | `anthropic/claude-opus-4` (or `claude-sonnet-3-7`) | 1× |
| Boilerplate, tests, single-function impl | `google/gemini-flash-1.5` or `openai/gpt-4o-mini` | ~0.05–0.10× |
| Status checks, CASS queries, doc gen | `google/gemini-2.0-flash-lite` or `GLM-5.1` | ~0.02–0.08× |

### Integration Plan

**Step 1 — Add to agent-boot env var layer**

OpenRouter key and base URL are set per-agent at boot, alongside existing KoadOS env vars:

```toml
# blueprints/squad-leader/IDENTITY.toml
[env]
ANTHROPIC_BASE_URL = "https://openrouter.ai/api"
ANTHROPIC_API_KEY  = " vault:openrouter_api_key "
ANTHROPIC_MODEL    = "anthropic/claude-sonnet-3-7"  # default; override per task
```

**Step 2 — Gate behind a flag for initial rollout**

Add `--openrouter` flag to `koad boot`. Do not make it the default until at least one full sprint is validated on Jupiter.

```bash
koad boot --agent clyde --openrouter
```

**Step 3 — Model routing in [CLAUDE.md](http://CLAUDE.md)**

Once stable, encode model selection guidance directly in Clyde's `CLAUDE.md`:

```markdown
## Model Selection
You are running via OpenRouter. Default model is `anthropic/claude-sonnet-3-7`.
For simple, scoped tasks (single-file edits, boilerplate, test generation),
request model downgrade to `google/gemini-flash-1.5` via task header:
  <!-- model: google/gemini-flash-1.5 -->
For complex multi-crate refactors or architecture decisions, use default or
explicitly request `anthropic/claude-opus-4`.
```

### Risks & Mitigations

- **Network hop reliability:** OpenRouter adds a relay. If OpenRouter is down, Clyde cannot work. Mitigation: keep `ANTHROPIC_API_KEY` (direct) in Vault as fallback; add `koad boot --direct` flag.
- **Prompt injection via third-party relay:** OpenRouter is a third-party service in the request path. Do not send secret keys, credentials, or PII through OpenRouter-routed sessions. Citadel safety rules already prohibit this, but confirm `PreToolUse` gates are enforced.
- **Model behavior drift:** Routing to non-Claude models changes output style. Caveman skill mitigates this for output; input behavior may differ. Test on Cid-class tasks first before routing Clyde tasks to non-Claude models.

---

## P3 — caveman Skill

### What It Is

[github.com/juliusbrussee/caveman](http://github.com/juliusbrussee/caveman) — A Claude Code skill (`.claude/skills/`) that instructs Claude to respond in terse, caveman-style English. Strips pleasantries, hedging, verbose explanation, and filler text while preserving full technical accuracy and code blocks.

**Install (one-liner):**

```bash
npx claudepluginhub juliusbrussee/caveman
```

Installs to `.claude/skills/` (project) or `~/.claude/skills/` (global).

The ecosystem also includes:

- **cavemem** — session memory compression (~46% reduction on input tokens each session)
- **cavekit** — build tooling helpers

### Why It Matters for Clyde

Output tokens are a smaller fraction of total quota than input tokens. However, in multi-agent coordinator workflows — where Clyde's task notifications and turn-completion summaries are consumed by Tyr's Mailbox — shorter outputs reduce downstream context bloat in a compounding way. Each Clyde response that enters a sub-agent or coordinator context window as a tool result is subject to this savings.

**cavemem** is likely the higher-impact companion: it compresses the session context before each API call, reducing the input token footprint of Clyde's running conversation history.

### Integration Plan

**Add to Skills Manifest (`skills/registry.toml`):**

```toml
[skills.caveman]
source   = "juliusbrussee/caveman"
tier     = "standard"
tags     = ["efficiency", "output-compression"]
harness  = ["claude-code"]
rank_gate = "squad-leader"  # Clyde and above

[skills.cavemem]
source   = "juliusbrussee/cavemem"
tier     = "standard"
tags     = ["efficiency", "context-compression"]
harness  = ["claude-code"]
rank_gate = "squad-leader"
```

**Recommended activation levels:**

- Default: caveman level-1 (mild compression) always-on for Clyde.
- During `--coordinator` sessions: caveman level-3 for worker sub-agents to keep Mailbox notifications compact.
- For Cid (Codex): check caveman Codex plugin compatibility before enabling.

### Risks & Mitigations

- **Readability for human review:** Caveman output can be terse to the point of ambiguity in multi-step reasoning chains. When Ian is reviewing Clyde's plan-mode output, caveman level-1 is preferable to deeper levels. Encode this in Clyde's `CLAUDE.md` as a mode-based override.
- **Actual bill impact is modest:** Measured output-token savings are real (~65–75%) but output tokens are a smaller cost driver than input tokens. Primary value is context chain compression in multi-agent flows, not direct API cost reduction.

---

## P4 — agent-browser (Vercel Labs)

### What It Is

[github.com/vercel-labs/agent-browser](http://github.com/vercel-labs/agent-browser) — A browser automation CLI that uses the **accessibility tree** rather than screenshots. Claimed 82% fewer tokens per browser interaction compared to Playwright MCP.

### Why It Is P4 (Backlog)

Citadel Jupiter's primary workload is Rust/code-focused. Agent-browser's token savings only apply when Clyde is performing browser-based verification tasks — e.g., confirming a deployed Skylinks Admin Chrome Extension is live, or verifying that an SWS web endpoint renders correctly.

This is a real use case but not a high-frequency one in the current sprint backlog.

**Caveat on the 82% savings claim:** At least one measured comparison shows higher token usage in specific scenarios compared to Playwright MCP. Results are task-dependent; the accessibility tree is more compact per page but agent-browser's state reporting can add overhead on complex SPAs.

### Integration Plan

**Register as an MCP server on Clyde:**

```bash
claude mcp add agent-browser --transport stdio "npx @vercel-labs/agent-browser"
```

**Add to Skills Manifest as experimental:**

```toml
[skills.agent-browser]
source    = "vercel-labs/agent-browser"
tier      = "experimental"
tags      = ["browser", "verification", "web"]
harness   = ["claude-code"]
rank_gate = "squad-leader"
notes     = "MCP server, not a skill file. Register via claude mcp add."
```

Do not enable globally. Activate per-task when Clyde is doing browser verification work.

---

## Global Configuration Changes Summary

| **File / System** | **Change** | **Owner** |
| --- | --- | --- |
| `koad boot` script | Add `rtk init -g` to Clyde boot sequence; add `--openrouter` flag wiring `ANTHROPIC_BASE_URL` | Tyr |
| `blueprints/squad-leader/IDENTITY.toml` | Add `[env]` block with `ANTHROPIC_BASE_URL`, `ANTHROPIC_MODEL` fields | Tyr |
| `.claude/CLAUDE.md` (Clyde) | Add model selection guidance section; add caveman level override by mode | Clyde (authored by Tyr) |
| `skills/registry.toml` | Add entries for `caveman`, `cavemem`, `agent-browser` (experimental) | Tyr |
| `~/.claude/settings.json` | Add `PostToolUse` BashTool hook for rtk if boot-level init is not used (Option B) | Tyr |
| Vault | Add `openrouter_api_key` secret; retain direct `anthropic_api_key` as fallback | Tyr |

---

## Open Questions for Tyr

1. **rtk integration point:** Boot script (Option A) vs. `PostToolUse` hook (Option B)? Option A is simpler and more complete; Option B is more surgical but requires hook config maintenance.
2. **OpenRouter rollout scope:** Clyde-only initially, or does Cid (Codex) also benefit from `ANTHROPIC_BASE_URL` routing via OpenRouter?
3. **cavemem activation:** Should cavemem run as a `PreCompact` hook or as a standalone skill triggered by Clyde? Clarify ownership in the hooks config.
4. **KOAD_DEBUG bypass for rtk:** Confirm the mechanism for bypassing rtk filtering during active debug sessions — env var flag or explicit `--raw` prefix convention?
5. **OpenRouter security posture:** Confirm that existing `PreToolUse` Citadel safety gates are sufficient to prevent credential/PII exfiltration through OpenRouter-routed sessions before enabling in production.

---

## References

- [rtk —](https://github.com/rtk-ai/rtk) [github.com/rtk-ai/rtk](http://github.com/rtk-ai/rtk)
- [caveman —](https://github.com/juliusbrussee/caveman) [github.com/juliusbrussee/caveman](http://github.com/juliusbrussee/caveman)
- [agent-browser —](https://github.com/vercel-labs/agent-browser) [github.com/vercel-labs/agent-browser](http://github.com/vercel-labs/agent-browser)
- [OpenRouter —](https://openrouter.ai) [openrouter.ai](http://openrouter.ai)
- [Claude Code Harness — Architecture & KoadOS Integration Report](https://www.notion.so/Claude-Code-Harness-Architecture-KoadOS-Integration-Report-9762b77d8a3949faa961f78cf03e1256?pvs=21)
- [KoadOS Agent Skills Manifest — Deep Dive & Architecture Proposal](https://www.notion.so/KoadOS-Agent-Skills-Manifest-Deep-Dive-Architecture-Proposal-0a0194ba68a24e0290291f9c452fd54a?pvs=21)