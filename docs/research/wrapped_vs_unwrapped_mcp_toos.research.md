## Filesystem MCP Server — KoadOS Integration Deep Dive

**Mode: Engineering / KoadOS · Devil's Advocate: Active**

---

### 1. How Each Harness Handles MCP Natively

#### Claude Code

Claude Code has the deepest, most native MCP integration of the three. It speaks STDIO and SSE natively, discovers tools at connection time via the MCP handshake, and exposes them to the model as first-class tool calls — no wrapper layer, no translation shim. The model reasons about tool schemas exactly as they're published by the server.

Key facts relevant to KoadOS:

- MCP tool schemas are injected into the **system prompt** at session start. Every tool registered counts as input tokens — whether called or not.
- Claude Code's hooks (`PreToolUse`, `PostToolUse`, `PreEdit`, `Stop`) fire **around** tool calls, giving you an intercept point without wrapping the server itself.
- Claude Code uses auto-compaction (200K context window) and has its own internal token tracking, but **does not expose per-tool token metrics** externally without instrumentation.
- A wrapper sitting between Claude Code and the MCP server would need to speak valid MCP protocol — otherwise it breaks the native discovery and tool-call flow entirely. A broken shim = broken tool calls.

**Bottom line on Claude Code:** It has the best native MCP wiring of the three. A wrapper *can* work but must be a fully MCP-compliant proxy — not a passthrough script. The hook system (`PreToolUse`/`PostToolUse`) is the safer and less invasive intercept point for metrics.

---

#### Gemini CLI

Gemini CLI's MCP layer (from my research brief on it)[[1]](https://www.notion.so/Research-Brief-Notion-MCP-via-Gemini-CLI-5a78804bd5e7460cb6116334f0f07c36?pvs=21) lives in `mcp-client.ts` and `mcp-tool.ts`. It:

- Discovers tools, sanitizes schemas for Gemini API compatibility, and registers them in a global tool registry.
- Supports `includeTools` / `excludeTools` per-server in `settings.json` — built-in allowlisting.
- Has a `trust` flag that bypasses confirmation entirely.
- Has a **confirmation chain** before every tool invocation — a natural intercept point.

Unlike Claude Code, Gemini CLI does **not** have a hooks system. Your only intercept points are:

1. The MCP server itself (wrap the server)
2. The transport layer (STDIO proxy)
3. Shell-level process wrappers around the CLI invocation

**Bottom line on Gemini CLI:** A **transparent MCP proxy** (a shim server that speaks MCP, forwards calls to the real Filesystem MCP Server, and logs metrics) is the cleanest approach here. There's no hook system to exploit.

---

#### Codex (OpenAI Codex CLI)

Codex CLI is the youngest and thinnest of the three. Its MCP support was added more recently and is less battle-tested than Claude Code's. It:

- Supports MCP via `settings.json` / `.codex/config.toml`
- Has a simpler approval model (full-auto, suggest, or ask mode)
- **Does not have hooks or a middleware layer** comparable to Claude Code

Same story as Gemini CLI — a transparent MCP proxy is your best intercept option.

---

### 2. The Core Token Efficiency Problem

Here's the real issue you need to internalize from the research:[[2]](webpage://?url=https%3A%2F%2Fwww.reddit.com%2Fr%2FClaudeAI%2Fcomments%2F1rzz784%2Fmcp_is_costing_you_37_more_tokens_than_necessary%2F)

> **MCP tool schemas are injected as input tokens at session start, regardless of whether the tools are called.**
> 

A Filesystem MCP Server with 20+ tools means 20+ schema definitions stuffed into every session's context. This is unavoidable in the base MCP protocol — the client has to enumerate tools. Benchmarks have shown **+36.7% input token overhead** vs. CLI-equivalent operations, scaling linearly with tool count.

The strategies to fight this, ranked by effectiveness:

| Strategy | Mechanism | Token Savings | Complexity |
| --- | --- | --- | --- |
| **Tool allowlisting** (`includeTools`) | Only expose tools the agent actually needs | High — linear reduction | Low |
| **Response compression in wrapper** | Intercept server responses, strip noise, summarize | High — 95–98% on large responses | Medium |
| **Caching repeated reads** | Wrapper caches file reads, returns cached result | Medium | Medium |
| **Result pagination / truncation** | Wrapper truncates large directory listings, file reads | Medium | Low-Medium |
| **CLI fallback for simple ops** | For known-cheap operations (e.g., `cat small_file`), bypass MCP entirely | Medium | High |

---

### 3. The Wrapper Decision

**Should KoadOS wrap the Filesystem MCP Server?**

**Answer: Yes — but as a transparent MCP proxy, not a script wrapper.**

Here's why and how:

A **transparent MCP proxy** sits between the agent harness and the real Filesystem MCP Server. It speaks full MCP protocol on both ends. From the harness's perspective, it *is* the MCP server. From the real server's perspective, it's a client. In the middle, it:

- **Intercepts tool schemas** at discovery time → strips or excludes tools not needed for the current agent role
- **Intercepts tool call requests** → logs: agent ID, tool name, timestamp, estimated input tokens
- **Intercepts tool responses** → compresses/truncates large payloads before they hit the model context, logs: response size, estimated output tokens
- **Aggregates metrics** → writes to CASS or a sidecar metrics store (Redis, SQLite on Jupiter)
- **Enforces Sanctuary Rule** → can reject calls to protected paths before they reach the real server (this is currently done via hooks in Claude Code, but the proxy makes it harness-agnostic)

This approach is **harness-agnostic**. All three harnesses wire to the proxy. The proxy connects to the real Filesystem MCP Server. One integration point for all metrics, all compression, all access control.

**Architecture:**

```
[Claude Code]   ─┐
[Gemini CLI]    ─┼──► [KoadOS FS-MCP Proxy] ──► [Filesystem MCP Server]
[Codex CLI]     ─┘         │
                            └──► CASS / Metrics Sink
```

Transport recommendation:

- **STDIO** for Claude Code (spawns proxy as subprocess, proxy spawns real server)
- **STDIO or Streamable HTTP** for Gemini CLI and Codex (same proxy, different transport binding)
- The proxy can speak STDIO on the harness-facing side and STDIO or HTTP on the server-facing side — fully configurable

---

### 4. Claude Code Specifics — Does a Wrapper Interfere?

This is the key question. The answer is **no, if done correctly**.

Claude Code's native MCP wiring discovers tools from whatever server it connects to. If the proxy faithfully re-publishes the Filesystem MCP Server's tool manifest (with allowed tools only), Claude Code has no idea there's a proxy in the middle. It calls tools, gets responses — nothing breaks.

**What would break:**

- A proxy that mangles tool schemas (Claude Code is strict about `input_schema` format)
- A proxy that delays responses past Claude Code's timeout thresholds
- A proxy that drops required MCP handshake messages (initialize, initialized, etc.)

**What you can still use with the proxy in place:**

- Claude Code hooks (`PreToolUse`, `PostToolUse`) — these fire on Claude Code's side, before the call reaches the proxy. You can do Claude-Code-specific logging/enforcement here *in addition* to the proxy. For KoadOS, hooks handle Sanctuary Rule enforcement as a second layer; the proxy handles cross-harness metrics.
- [CLAUDE.md](http://CLAUDE.md) instructions — unaffected
- Body/Ghost boot sequence — unaffected

**What the proxy gives you that hooks alone can't:**

- Metrics from Gemini CLI and Codex (no hooks exist there)
- Response compression (hooks fire before/after, but can't modify the response body that hits the context — the proxy can)
- Harness-agnostic Sanctuary Rule enforcement

---

### 5. Metrics KoadOS Should Capture

Per agent call through the proxy:

```
{
  agent_id: "tyr-session-abc123",
  harness: "claude_code" | "gemini_cli" | "codex",
  tool_name: "read_file" | "list_directory" | ...,
  timestamp_request: ISO8601,
  timestamp_response: ISO8601,
  latency_ms: number,
  request_token_estimate: number,   // schema + params
  response_token_estimate_raw: number,    // before compression
  response_token_estimate_sent: number,   // after compression
  token_savings: number,
  path_accessed: string,            // sanitized, Sanctuary check result
  sanctuary_blocked: boolean,
  cache_hit: boolean,
  session_id: string                // links to CASS session
}
```

Aggregate to CASS:

- Per-session token budget consumed
- Per-tool call frequency (identify over-called tools → candidates for allowlist pruning)
- Compression ratio over time
- Sanctuary violation attempts

---

### 6. Implementation Phasing

**Phase 1 — Wire Claude Code (no proxy yet)**

Use `PreToolUse`/`PostToolUse` hooks in [CLAUDE.md](http://CLAUDE.md) to log FS tool calls to a local JSON file or Redis. This gets you metrics for Claude Code immediately with zero proxy complexity. Tool allowlisting via `settings.json` `includeTools` reduces schema tokens right now.

**Phase 2 — Build the FS-MCP Proxy (Rust or Python)**

Build a minimal MCP proxy that:

- Speaks STDIO on both ends
- Forwards tool discovery with allowlist filtering
- Compresses/truncates large responses (directory listings > N items, files > N bytes → truncated with a `fetch_more` hint)
- Writes metrics to Redis/SQLite

Wire Gemini CLI and Codex to the proxy. Migrate Claude Code from hook-only to proxy + hooks (additive, not replacing).

**Phase 3 — CASS Integration**

Proxy writes metrics to CASS via `koad_metric_write` MCP tool (or direct Redis). Session-level token budgets enforced. Citadel dashboard reads from CASS.

---

### 🔥 Devil's Advocate

**1. The proxy adds a failure point.** Every agent harness now depends on your proxy being alive. If the proxy crashes, all filesystem operations for all harnesses fail simultaneously. You need process supervision (systemd, Docker restart policy) and a fallback config that bypasses the proxy. Don't deploy this without a dead-man's switch.

**2. Token estimate accuracy is fake without API access.** "Estimated tokens" from a proxy means you're running a tokenizer locally (e.g., `tiktoken` for Claude, which is close but not exact). The real token count is only known by the model provider. You'll get directionally correct numbers but not billing-accurate. Don't build financial reporting on proxy estimates — use them for optimization signals only.

**3. Response compression is a footgun.** If the proxy truncates a file read response and the agent doesn't know the data was truncated, it will make decisions on incomplete data. Every compressed response must include explicit truncation metadata that the agent can reason about. This requires either: (a) model-aware truncation with a summary header, or (b) pagination support in the proxy. Neither is trivial. Shipping naive truncation without this causes subtle, hard-to-debug bugs.

**4. The Sanctuary Rule already works via hooks in Claude Code.** Building Sanctuary enforcement into the proxy is the right long-term call for harness-agnostic coverage, but it duplicates logic that already exists in hooks. Until Gemini CLI and Codex are active on sensitive paths, this duplication is low-risk — just document which layer is authoritative (proxy wins if both fire).

**5. Tool allowlisting is the highest-ROI, lowest-complexity move.** Before building a proxy at all: figure out which of the Filesystem MCP Server's tools each agent actually needs. A read-only research agent needs `read_file`, `list_directory`, `search_files`. It does not need `write_file`, `move_file`, `delete_file`. Stripping those from the discovery payload alone cuts schema tokens by 40–60%. Do this first, on all three harnesses, in an afternoon. The proxy can wait.

---

### Summary Recommendation
| Priority | Action |
| --- | --- |
| **Do now** | Allowlist tools per harness in `settings.json` / config. Cut schema tokens immediately. |
| **Do now (Claude Code)** | Add `PostToolUse` hook to log FS tool calls to a local metrics file. Zero-proxy metrics for Claude Code today. |
| **Build next** | KoadOS FS-MCP Proxy — Rust preferred (fast, fits Citadel stack). Harness-agnostic intercept, compression, CASS metrics write. |
| **Do not** | Build a non-MCP-compliant wrapper script. It will break Claude Code's native discovery and tool routing entirely. |
| **Design now** | Truncation/compression protocol — define the metadata contract before writing a single line of proxy code. Agents must know when data is truncated. |

The proxy is the right architecture. Claude Code won't fight it. But allowlisting + hooks gets you 80% of the value in a fraction of the time. Build those now while the proxy is in design.