# koad-agent boot: Shell Environment Hydration

> The "Link" command that fuses an agent's persistent identity (Ghost) with an active shell session (Body) by generating environment variable exports through a gRPC handshake with the Citadel.

**Complexity:** intermediate
**Related Articles:** [The Body/Ghost Model](../architecture/body-ghost-model.md), [The Tri-Tier Model](../architecture/tri-tier-model.md), [Agent Session Lifecycle](./agent-session-lifecycle.md), [koad-citadel](./koad-citadel.md)

---

## Overview

`koad-agent boot` is the canonical entry point for any KoadOS agent session. It is the "Link" tier in the [Tri-Tier Model](../architecture/tri-tier-model.md) — the command that connects an agent's persistent [Ghost](../architecture/body-ghost-model.md) (identity and accumulated memory) to an active shell [Body](../architecture/body-ghost-model.md) (the current terminal session).

The boot command is always invoked with `eval`:

```bash
eval $(koad-agent boot --agent Tyr)
```

The `eval` is essential. `koad-agent` is a subprocess — it cannot modify the parent shell's environment directly. Instead, it prints a shell script containing `export` statements to stdout. The `eval $(...)` wrapper captures this output and executes it in the parent shell, injecting the environment variables into the active terminal.

After boot, the shell holds the agent's identity and session credentials. Any tool or process that subsequently reads `KOAD_SESSION_TOKEN` from the environment can authenticate its gRPC calls to the Citadel as this agent.

The boot process is designed to be fast and token-efficient. By the time an AI CLI (like Gemini CLI) initializes, the environment is already fully prepared — the system prompt can dynamically reference `KOAD_AGENT_NAME` and `KOAD_SESSION_ID` to anchor the agent's identity without burning context tokens on orientation.

## How It Works

Running `eval $(koad-agent boot --agent Tyr)` triggers a multi-step sequence:

### Step 1: Parse and Identify

The `koad-cli` binary receives the `boot` subcommand with `--agent Tyr`. The `Commands::Boot` variant in `main.rs` dispatches to `handle_boot_command()` in `handlers/boot.rs`.

### Step 2: Load the Ghost Identity

The handler reads the agent's identity from `config/identities/tyr.toml`. This TOML file defines Tyr's name, rank, permissions, and capability tier — the static definition of the Ghost. A fresh `body_id` UUID is generated at this point to uniquely identify this specific terminal instance.

### Step 3: gRPC Handshake with the Citadel

The handler makes a `CreateLease` gRPC call to the Citadel's `CitadelSession` service, sending:
- The agent's name (`"Tyr"`)
- The freshly generated `body_id`
- The agent's rank (read from the identity TOML)

The Citadel receives this request through its Zero-Trust interceptor. At `CreateLease`, the request doesn't yet have a session token — this is the one gRPC call that's authenticated differently (using the identity TOML as the credential, or an initial bootstrap token). The Citadel then:
1. Checks whether "Tyr" already has an active session in Redis (enforcing "One Body, One Ghost")
2. If not, creates a `SessionRecord` in `koad:state` Redis and a lease key at `koad:session:<new_session_id>` with the configured TTL
3. Returns the new `session_id` and `session_token`

### Step 4: (Design Intent) Context Hydration

The full design calls for `koad-cli` to make a `Hydrate` call to CASS at this point, receiving the agent's context packet (recent session summaries, high-confidence facts, local file context). This context is written to a temporary file or injected into the environment for the AI CLI to consume.

> **Note:** The CASS hydration call from `koad-cli` is part of the design intent but may not be fully wired in the current CLI implementation. The `CassHydrationService` exists and is functional; the integration with `koad-cli` is pending. Check `handlers/boot.rs` for current status.

### Step 5: Generate the Shell Script

`handle_boot_command()` does not modify the shell directly. It constructs a shell script as a `String` and prints it to stdout:

```bash
export KOAD_AGENT_NAME="Tyr";
export KOAD_AGENT_RANK="Captain";
export KOAD_SESSION_ID="sid_abcd1234...";
export KOAD_SESSION_TOKEN="tok_efgh5678...";
echo "KoadOS: Shell hydrated for agent Tyr (Session: sid_abcd1234...).";
```

### Step 6: `eval` Executes the Script

The parent shell's `eval $(...)` captures the stdout output of the `koad-agent` subprocess and executes it as shell commands. The `export` statements set the environment variables in the active session. `koad-agent` exits; the variables remain.

From this point on, any process in the shell that reads `KOAD_SESSION_TOKEN` from the environment has the credential it needs to authenticate gRPC calls to the Citadel as Tyr.

### Security: `session_id` vs `session_token`

The boot process produces two session-related values:

| Variable | Nature | Purpose |
|---|---|---|
| `KOAD_SESSION_ID` | Public identifier | References the session in Redis; used for logging, audit, display |
| `KOAD_SESSION_TOKEN` | Private credential | The secret validated by the Citadel interceptor on every gRPC call |

The `session_id` is safe to log and display. The `session_token` is the actual security credential and should be treated like a password — it grants full agent-level access to the Citadel for the duration of the session.

## Configuration

| Variable | Set by | Description |
|---|---|---|
| `KOAD_AGENT_NAME` | `koad-agent boot` | The name of the booted agent (e.g., `"Tyr"`) |
| `KOAD_AGENT_RANK` | `koad-agent boot` | The agent's rank (e.g., `"Captain"`) |
| `KOAD_SESSION_ID` | `koad-agent boot` | The public session identifier from the Citadel |
| `KOAD_SESSION_TOKEN` | `koad-agent boot` | The private credential for gRPC authentication |
| `config/identities/<agent>.toml` | Developer | Source of the agent's Ghost identity; must exist before boot |

## Failure Modes & Edge Cases

**`koad-agent boot` fails with "agent already active".**
The Citadel found an existing live session for this agent name. Either another terminal has this agent booted, or a previous session crashed without being cleaned up. Wait for the reaper to purge the stale session (controlled by `dark_timeout_secs` in `kernel.toml`), or ask the Admiral to manually clear it from Redis.

**`koad-agent boot` fails with a gRPC connection error.**
The Citadel is not running or is not reachable at the configured address. Start `koad-citadel` first before attempting to boot an agent. Verify the address in `config/kernel.toml` matches where the Citadel is actually listening.

**`eval` is omitted — variables don't appear in the shell.**
Without `eval`, running `koad-agent boot --agent Tyr` prints the export script to the terminal but doesn't execute it. The user sees the script as text output but the variables are never set. The fix is always to use `eval $(koad-agent boot --agent Tyr)`.

**The identity TOML file is missing or malformed.**
`handle_boot_command()` fails at Step 2 with a config parse error before any gRPC call is made. The agent TOML must exist in `config/identities/` and must parse as a valid `Identity` struct. Check the file path and TOML syntax.

**The Citadel is running but CASS is down.**
If the CASS hydration call is implemented and CASS is unreachable, the boot process will either fail (if hydration is treated as required) or succeed with an empty context packet (if it's treated as optional). The session lease is issued by the Citadel independently of CASS. Check `handlers/boot.rs` to see whether the hydration call is made and how errors are handled.

## FAQ

### Q: Why do I have to use `eval`? What does it do?
`eval $(koad-agent boot ...)` is a standard Unix pattern for allowing a subprocess to modify the parent shell's environment. A child process cannot directly set environment variables in its parent — those changes would be lost when the child exits. The workaround: print `export VAR=value` statements to stdout, and use `eval` in the parent to execute them. Without `eval`, the export script is printed as visible text but never executed. With `eval`, the variables are set in your current shell session.

### Q: What happens when I run `koad-agent boot`?
Six things happen in sequence: (1) the CLI reads the agent's identity from its TOML file, (2) generates a fresh `body_id` UUID for this terminal, (3) makes a `CreateLease` gRPC call to the Citadel, (4) the Citadel validates the request, creates a Redis session record, and returns a `session_id` and `session_token`, (5) optionally makes a `Hydrate` call to CASS for the initial context packet, and (6) prints a shell script of `export` commands to stdout for `eval` to execute in your shell.

### Q: How does the system know who I am?
You declare your identity via the `--agent` flag (e.g., `--agent Tyr`). The CLI reads the corresponding `config/identities/tyr.toml` file for the identity details. The Citadel doesn't independently "know" who you are — it trusts the identity you declare at `CreateLease` time and issues a session token tied to that identity. From that point on, the `KOAD_SESSION_TOKEN` in your environment is the credential that proves your identity on every subsequent call.

### Q: Where does the session ID come from?
The `session_id` is generated by the Citadel in `CitadelSessionService::create_lease()` and returned in the `LeaseResponse`. It's a unique identifier assigned to this specific session. The `body_id` (the UUID representing your terminal) is generated by `koad-cli` before contacting the Citadel. Both are stored in the `SessionRecord` in Redis.

### Q: Can I boot an agent without connecting to the Citadel?
No — not in the current design. The `CreateLease` gRPC call to the Citadel is required to obtain a valid `session_token`. Without a valid token, every subsequent call to any Citadel service will fail the interceptor check. You could theoretically source the environment variables manually, but the session wouldn't be tracked in Redis, heartbeats would fail, and the CASS hydration call would fail. The boot process requires a running Citadel.

## Source Reference

- `crates/koad-cli/src/main.rs` — `Commands::Boot` enum variant; defines the CLI arguments for the boot command
- `crates/koad-cli/src/handlers/boot.rs` — `handle_boot_command()`; the full boot sequence implementation
- `crates/koad-citadel/src/services/session.rs` — `CitadelSessionService::create_lease()`; the server-side lease creation handler
- `config/identities/` — TOML identity files; one per agent, must exist before booting
