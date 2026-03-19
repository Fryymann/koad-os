<aside>
📋

**Audience:** Tyr (Architect Agent) · **Purpose:** Pre-design research for the Notion MCP Tool · **Prepared by:** Noti (KoadOS Researcher)

</aside>

## 1 · MCP Protocol Fundamentals

The **Model Context Protocol (MCP)** is an open standard that lets AI agents interact with external systems through a unified client ↔ server interface. Key primitives:

- **Tools** — callable functions with typed input schemas; the agent discovers them at connection time and invokes them by name.
- **Resources** — read-only data the server exposes (files, API payloads, reports); referenced via URI.
- **Prompts** — server-defined prompt templates the client can enumerate.

Transport is abstracted. A single MCP client can talk to many servers simultaneously over different transports.

---

## 2 · Gemini CLI MCP Integration

### 2.1 Architecture Overview

Gemini CLI's MCP layer lives in `packages/core/src/tools/` and has two key modules:

- **Discovery layer** (`mcp-client.ts`) — iterates `mcpServers` from `settings.json`, opens transports, fetches tool/resource lists, sanitizes schemas for Gemini API compatibility, and registers everything in a global tool registry with conflict resolution.
- **Execution layer** (`mcp-tool.ts`) — wraps each discovered tool in a `DiscoveredMCPTool` instance that manages connection state, timeouts, confirmation logic, and response formatting.

### 2.2 Supported Transports

| **Transport** | **Config Key** | **Mechanism** | **Best For** |
| --- | --- | --- | --- |
| Stdio | `command` | Spawns subprocess, communicates via stdin/stdout | Local servers, CLI tools, Docker containers |
| SSE | `url` | Server-Sent Events over HTTP | Legacy/fallback remote servers |
| Streamable HTTP | `httpUrl` | HTTP streaming | Modern remote servers (preferred) |

### 2.3 Configuration Shape (`settings.json`)

Servers are declared under `mcpServers`. Each entry requires exactly one transport key (`command`, `url`, or `httpUrl`) plus optional fields:

```json
{
  "mcpServers": {
    "serverName": {
      "command": "npx",
      "args": ["-y", "@notionhq/notion-mcp-server"],
      "env": { "NOTION_TOKEN": "$NOTION_TOKEN" },
      "cwd": "./optional-dir",
      "timeout": 30000,
      "trust": false,
      "includeTools": ["tool-a", "tool-b"],
      "excludeTools": ["tool-c"]
    }
  }
}
```

**Key optional properties:**

- `trust` (bool) — bypasses all tool-call confirmations when `true`.
- `timeout` (ms) — per-request timeout; default 600 000 ms (10 min).
- `includeTools` / `excludeTools` — allowlist/blocklist for tool exposure. `excludeTools` wins on overlap.
- `env` — supports `$VAR` / `${VAR}` expansion. Sensitive host vars are auto-redacted unless explicitly declared here.

### 2.4 Tool Naming & Namespacing

Every MCP tool gets a **Fully Qualified Name**: `mcp_{serverName}_{toolName}`.

<aside>
⚠️

**Do not use underscores in server names** (e.g. use `notion-api`, not `notion_api`). The policy parser splits FQNs on the first underscore after `mcp_`, so underscores in the server name break wildcard rules silently.

</aside>

### 2.5 Schema Sanitization

Before registering tools with the Gemini API, the CLI:

- Strips `$schema` and `additionalProperties`
- Removes `default` values from `anyOf` branches (Vertex AI compat)
- Truncates tool names > 63 chars with middle-replacement (`...`)
- Replaces non-alphanumeric chars (except `_`, `-`, `.`, `:`) with underscores

### 2.6 OAuth for Remote Servers

Gemini CLI supports automatic OAuth discovery for remote MCP servers. Flow:

1. Initial connection returns 401
2. CLI discovers OAuth endpoints from server metadata
3. Browser opens for user authentication on `localhost:7777/oauth/callback`
4. Tokens stored in `~/.gemini/mcp-oauth-tokens.json`, auto-refreshed

Alternatively, Google Credentials or Service Account Impersonation auth providers are supported for GCP-hosted servers.

### 2.7 Confirmation & Trust Model

Tool invocations go through a confirmation chain:

1. If `trust: true` → skip confirmation entirely
2. Dynamic allow-lists (per-tool or per-server) can be built at runtime
3. User can approve once, approve for session, or cancel

---

## 3 · Notion MCP Server

Notion offers **two server options**:

| **Option** | **Type** | **Auth** | **Status** |
| --- | --- | --- | --- |
| **Notion MCP (Remote/Hosted)** | Remote (Streamable HTTP / SSE) | OAuth 2.0 + PKCE (user consent flow) | Actively supported, recommended by Notion |
| **notion-mcp-server (Local)** | Local (Stdio or Streamable HTTP) | Internal Integration Token (`NOTION_TOKEN`) | Open-source (MIT), may be sunsetted; not actively supported |

### 3.1 Remote Server — Notion MCP (Hosted)

**Endpoints:**

- Streamable HTTP (preferred): `https://mcp.notion.com/mcp`
- SSE (fallback): `https://mcp.notion.com/sse`

**Auth:** OAuth 2.0 Authorization Code + PKCE via dynamic client registration (RFC 7591). Discovery follows RFC 9470 → RFC 8414. Full workspace access scoped to the user's permissions.

**Rate Limits:** 180 requests/min general; 30 requests/min for search.

### 3.2 Local Server — `@notionhq/notion-mcp-server`

**Package:** `@notionhq/notion-mcp-server` (npm) or `mcp/notion` (Docker Hub)

**Language:** TypeScript · **License:** MIT · **Latest:** v2.1.0

**API Version:** Notion API `2025-09-03` (data sources model)

**Gemini CLI config (Stdio):**

```json
{
  "mcpServers": {
    "notionApi": {
      "command": "npx",
      "args": ["-y", "@notionhq/notion-mcp-server"],
      "env": { "NOTION_TOKEN": "$NOTION_TOKEN" }
    }
  }
}
```

**Gemini CLI config (Streamable HTTP, self-hosted):**

```bash
npx @notionhq/notion-mcp-server --transport http --port 3000 --auth-token "$AUTH_TOKEN"
```

```json
{
  "mcpServers": {
    "notionApi": {
      "httpUrl": "http://localhost:3000/mcp",
      "headers": { "Authorization": "Bearer $AUTH_TOKEN" }
    }
  }
}
```

### 3.3 Tool Inventory (22 tools — Local Server v2.0+)

| **Category** | **Tool** | **Description** |
| --- | --- | --- |
| Search | `notion-search` | Full-text search across workspace (+ connected sources with AI plan) |
| Read | `notion-fetch` | Retrieve page, database, or data source by URL/ID |
| Pages | `notion-create-pages` | Create one or more pages with properties, content, templates, icons, covers |
| Pages | `notion-update-page` | Update properties, content, icon, cover; apply templates |
| Pages | `notion-move-pages` | Move pages/databases to a new parent |
| Pages | `notion-duplicate-page` | Duplicate a page (async) |
| Database | `notion-create-database` | Create database with initial data source and view |
| Data Source | `query-data-source` | Query with filters and sorts (replaces `post-database-query`) |
| Data Source | `retrieve-a-data-source` | Get metadata and schema |
| Data Source | `update-a-data-source` | Update properties |
| Data Source | `create-a-data-source` | Create new data source |
| Data Source | `list-data-source-templates` | List available templates |
| Database | `retrieve-a-database` | Database metadata including data source IDs |
| Views | `notion-create-view` | Create table/board/list/calendar/timeline/gallery/form/chart/map/dashboard views |
| Views | `notion-update-view` | Update view filters, sorts, display config |
| Query | `notion-query-data-sources` | Cross-data-source query with grouping/rollups (Enterprise + AI) |
| Query | `notion-query-database-view` | Query via pre-defined view (Business+) |
| Comments | `notion-create-comment` | Add page/block/reply comments |
| Comments | `notion-get-comments` | List all discussions on a page |
| Users | `notion-get-teams` | List teamspaces |
| Users | `notion-get-users` / `notion-get-user` | List workspace users / get user by ID |
| Meta | `notion-get-self` | Bot info and workspace metadata |

### 3.4 v2.0 Breaking Changes (Data Sources Model)

Notion API `2025-09-03` replaced the database abstraction with **data sources**:

- `database_id` params → `data_source_id`
- Search filter `"database"` → `"data_source"`
- 3 old tools removed, 7 new tools added (net +4)
- No code migration needed for MCP clients — tools auto-discovered on restart

### 3.5 Auth Deep Dive — Integration Token (Local)

1. Create an **Internal Integration** at `notion.so/profile/integrations`
2. Grant it access to target pages/databases (Access tab or per-page "Connect to")
3. Capabilities are configurable (read-only, read+write, etc.)
4. Token format: `ntn_****`
5. Pass via `NOTION_TOKEN` env var

### 3.6 Auth Deep Dive — OAuth 2.0 + PKCE (Remote)

Full flow for custom MCP clients connecting to `https://mcp.notion.com`:

1. **Discovery:** RFC 9470 (`/.well-known/oauth-protected-resource`) → RFC 8414 (`/.well-known/oauth-authorization-server`)
2. **PKCE:** Generate `code_verifier` (32 random bytes, base64url) → SHA-256 → `code_challenge`
3. **Dynamic Client Registration:** POST to `registration_endpoint` with redirect URIs, grant types
4. **Authorization:** Redirect user to `authorization_endpoint` with `code_challenge`, `state`, `scope`
5. **Token Exchange:** POST `authorization_code` + `code_verifier` to `token_endpoint`
6. **Connect:** Pass `Bearer {access_token}` in headers to `/mcp` or `/sse`
7. **Refresh:** Use `refresh_token` grant; servers may rotate refresh tokens

SDK support exists for TypeScript, Python, Go, Rust, Java, C#, and Ruby.

---

## 4 · Design Decision Inputs

### 4.1 Local vs Remote — Tradeoffs

| **Factor** | **Local (`notion-mcp-server`)** | **Remote (Notion MCP)** |
| --- | --- | --- |
| Setup complexity | Simple — token + npx/docker | OAuth flow required |
| Auth model | Static integration token | Per-user OAuth tokens with refresh |
| Token management | Single secret in env | Token storage, refresh, re-auth logic |
| Tool quality | 22 tools, direct Notion API mapping | AI-optimized tools, efficient token usage, Markdown editing |
| Connected sources | Notion workspace only | Notion + Slack, Drive, Jira, etc. (with AI plan) |
| Longevity | May be sunsetted | Actively supported, primary investment |
| Offline / air-gap | Works fully offline | Requires internet |
| Rate limits | Standard Notion API limits | 180 req/min general, 30 req/min search |
| Multi-user | Single integration identity | Per-user identity via OAuth |

### 4.2 Transport Selection Guidance

- **Stdio** is simplest for a local-first agent that spawns the server as a child process. No networking, no ports, no auth overhead.
- **Streamable HTTP** is best for a shared/persistent server that multiple agents connect to, or for the remote hosted option.
- **SSE** is fallback only — use when Streamable HTTP isn't available.

### 4.3 Tool Filtering Strategy

The `includeTools` / `excludeTools` config allows scoping what an agent can do. Potential patterns:

- **Read-only agent:** `includeTools: ["notion-search", "notion-fetch", "notion-get-comments", "notion-get-users"]`
- **Writer agent:** Include page CRUD + data source query tools
- **Admin agent:** Full tool access

### 4.4 Security Considerations

- Gemini CLI auto-redacts sensitive env vars (`*TOKEN*`, `*SECRET*`, `*KEY*`, etc.) from MCP server processes unless explicitly declared in `env`.
- `trust: true` should only be set for servers you fully control.
- Integration tokens grant workspace-wide access scoped to connected pages — principle of least privilege applies.
- OAuth tokens are scoped to the authorizing user's permissions — inherently more granular.

### 4.5 Key References

- [Gemini CLI MCP Docs](https://geminicli.com/docs/tools/mcp-server/)
- [Notion MCP Overview](https://developers.notion.com/guides/mcp/mcp)
- [Notion MCP Supported Tools](https://developers.notion.com/guides/mcp/mcp-supported-tools)
- [Notion MCP Client Integration Guide](https://developers.notion.com/guides/mcp/build-mcp-client)
- [Local Server GitHub (makenotion/notion-mcp-server)](https://github.com/makenotion/notion-mcp-server)
- [MCP Specification](https://modelcontextprotocol.io/introduction)