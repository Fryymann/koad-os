# koad-agent boot: Shell Environment Hydration

## Metadata
- Category: CORE SYSTEMS & SUBSYSTEMS
- Complexity: intermediate
- Related Topics: tri-tier-model, body-ghost-model, agent-session-lifecycle
- Key Source Files: `crates/koad-cli/src/main.rs`, `crates/koad-cli/src/handlers/boot.rs`
- Key Canon/Doc References: `AGENTS.md`

## Summary
The `koad-agent boot` command is the "Link" in the Tri-Tier model and the canonical entry point for any KoadOS agent session. Its purpose is to "hydrate" a standard shell environment with the identity, credentials, and context an agent needs to operate, fusing the agent's "Ghost" with its "Body" for the duration of a session. It is designed to be called with `eval`, which allows it to export environment variables directly into the user's active shell.

## How It Works
When a user runs `eval $(koad-agent boot --agent Tyr)`, a multi-step process occurs:

1.  **Parsing:** The `koad-cli` binary parses the `boot` command and identifies the target agent (`Tyr`).
2.  **Identity Loading:** It reads the agent's identity and rank from `config/identities/tyr.toml`.
3.  **gRPC Handshake:** It initiates a `CreateLease` gRPC call to the `CitadelSession` service running in the `koad-citadel` process. This request includes the agent's name and a new, unique `body_id`.
4.  **Citadel Validation:** The Citadel receives the request, validates that the "Tyr" ghost isn't already in use by another "Body", and if not, creates a `SessionRecord` in Redis. It returns a unique `session_id` and a session `token`.
5.  **Context Hydration Request:** *(Note: This is part of the design intent, but may not be fully implemented in the CLI yet).* The bootstrapper would then make a `Hydrate` call to CASS to get the initial context packet for the agent.
6.  **Shell Script Generation:** The `koad-agent` process does not modify the shell directly. Instead, it **prints a shell script to standard output**. This script contains a series of `export` commands.
7.  **`eval` Execution:** The `eval $(...)` wrapper in the user's shell captures this output script and executes it. This is what sets the environment variables in the user's active session.

The generated script typically looks like this:
```bash
export KOAD_AGENT_NAME="Tyr";
export KOAD_AGENT_RANK="Captain";
export KOAD_SESSION_ID="sid_abcd1234...";
export KOAD_SESSION_TOKEN="tok_efgh5678...";
echo "KoadOS: Shell hydrated for agent Tyr (Session: sid_abcd1234...).";
```

This process ensures that by the time the AI CLI (e.g., Gemini CLI) initializes, the environment is already fully prepared, and the agent's system prompt can be dynamically anchored to its correct identity.

## Key Code References
- **File**: `crates/koad-cli/src/main.rs`
  - **Element**: `Commands::Boot` enum variant
  - **Purpose**: Defines the command-line interface for the boot process, including arguments like `--agent`, `--project`, etc.
- **File**: `crates/koad-cli/src/handlers/boot.rs`
  - **Element**: `handle_boot_command()` function
  - **Purpose**: Contains the core logic for the boot process: calling the Citadel, generating the output script, and printing it to stdout.
- **File**: `crates/koad-citadel/src/services/session.rs`
  - **Element**: `CitadelSessionService::create_lease()`
  - **Purpose**: The server-side gRPC endpoint that receives the boot request and creates the session lease.

## Configuration & Environment
- `KOAD_AGENT_NAME`, `KOAD_AGENT_RANK`, `KOAD_SESSION_ID`, `KOAD_SESSION_TOKEN`: The primary environment variables exported by the boot process.
- `config/identities/<agent>.toml`: The source file for the agent's core identity.

## Common Questions a Human Would Ask
- "Why do I have to use `eval`? What does it do?"
- "What happens when I run `koad-agent boot`?"
- "How does the system know who I am?"
- "Where does the session ID come from?"
- "Can I boot an agent without connecting to the Citadel?"

## Raw Technical Notes
- The `eval $(...)` pattern is a standard shell mechanism for allowing a program to modify the parent shell's environment. Without `eval`, the `export` commands would only apply to the subshell the `koad-agent` process runs in, and the variables would be lost as soon as it exits.
- The distinction between `session_id` and `session_token` is important for security. The `session_id` is a public identifier, while the `token` is the secret credential used to authenticate gRPC calls via the interceptor.
- The boot process is a critical part of the "Zero-Trust" model. It's the only time an agent is "trusted" to declare its identity, and from that point on, all actions are validated against the session token issued by the Citadel.
