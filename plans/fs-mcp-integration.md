# Implementation Plan: Filesystem MCP Server Integration (Canon-Aligned)

## Objective
Integrate the official Model Context Protocol (MCP) Filesystem Server into the KoadOS ecosystem as a native, secure, and auditable service. This implementation provides sovereign agents with standardized filesystem access scoped by the Citadel's sanctuary rules, while strictly honoring the Admiral's personal grant to Tyr.

## Key Files & Context
- **Issue Trigger:** `TRC-TYR-20260319-FS-MCP` (To be spawned)
- **KoadOS Binary:** `koad` (Will host the `bridge fs` command)
- **Supervisor Binary:** `koad-fs-mcp` (New native Rust sidecar)
- **Library:** `koad-core` (For config schema updates)
- **Tyr's Personal Grant:** `/mnt/c/ideans/`

## Proposed Solution: The "Canon Supervisor" Strategy
We will follow the **Research -> Strategy -> Execution** cycle to build a high-performance Rust supervisor that wraps the locally installed `@modelcontextprotocol/server-filesystem`.

### 1. Architectural Alignment
- **Sanctuary Rule:** Directory whitelists are enforced at the supervisor level before being passed to the Node.js sidecar.
- **Sovereignty:** Personal grants (Tyr's `/mnt/c/ideans/`) are hardcoded into the supervisor to ensure they are non-negotiable.
- **Traceability:** Every mutating filesystem call is intercepted and logged to `.koad-os/docs/audits/filesystem-ops.log`.

### 2. Configuration Strategy
- **Global Config:** `filesystem.allowed_directories` in `kernel.toml`.
- **Identity Config:** `identities.<agent>.preferences.allowed_directories` for per-agent roles.
- **Load Pattern:** "Load on Execute" ensures permissions are evaluated at the moment of server start.

## Implementation Steps

### Phase 1: Preparation & Canon Compliance
1. **Spawn Issue:** Run `koad system spawn --title "Implement Filesystem MCP Server Integration" --template feature`.
2. **Local Install:** `npm install @modelcontextprotocol/server-filesystem` in the root directory.

### Phase 2: Core Infrastructure (Rust)
1. **Update `koad-core`:**
    - Add `allowed_directories: Vec<String>` to the `FilesystemConfig` struct.
2. **Implement `koad-fs-mcp`:**
    - Create `koad-cli/src/bin/koad-fs-mcp.rs`.
    - **Logic:**
        - Resolve active agent.
        - Merge whitelists: Global + Agent-Specific + (If Tyr, `/mnt/c/ideans/`).
        - Validate and canonicalize all paths.
        - Spawn the Node.js server with resolved paths.
        - Implement a standard stdio proxy with an audit interceptor.

### Phase 3: CLI & Tooling
1. **Update `koad-cli`:**
    - Add `koad bridge fs serve` to trigger the supervisor.
2. **Audit Logging:** Initialize `.koad-os/docs/audits/` and the log rotation logic.

## Verification & Audit
- **KSTP Audit:** Run the new service through the KSTP Concurrency and Environmental tests.
- **Sovereignty Test:** Confirm Tyr can access his personal grant while other agents are blocked.
- **Canon Review:** Perform a full KSRP pass before final saveup.

## Success Criteria
- First-pass successful `list_directory` call from a sovereign agent.
- Zero "Zombie" Node.js processes left after agent disconnect.
- Audit log correctly records the Trace ID and actor for all write operations.
