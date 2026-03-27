# Crate: koad-cli
**Status:** Complete (Primary Interface)

## Purpose
The primary interface for KoadOS agents and developers. This crate builds the `koad` and `koad-agent` binaries, providing all command-line subcommands for session management, system orchestration, and agent interaction.

## Source Map
- `main.rs`: Entry point for the `koad` binary.
- `cli.rs`: Command-line argument definitions (Clap).
- `handlers/`: Individual subcommand implementations.
    - `boot.rs`: Logic for `koad boot` and `koad logout`.
    - `agent.rs`: Agent identity management (`koad agent`).
    - `status.rs`: System telemetry and health reporting.
    - `map.rs`: Navigation and fast-travel logic.
    - `xp.rs`: Experience point and skill commands.
- `bin/`: Standalone utility binaries.
    - `koad-agent.rs`: Low-level boot/identity hydration tool.
    - `koad-map.rs`: HUD and navigation utility.
    - `koad-fs-mcp.rs`: Filesystem-scoped MCP server.
    - `koad-notion-mcp.rs`: Notion integration MCP server.
- `tui.rs`: Terminal UI components and status boards.
- `db.rs`: Local SQLite storage bridge for the CLI.

## Provided Binaries
- `koad`: The main command-line interface.
- `koad-agent`: Focused utility for agent shell hydration.
- `koad-map`: Specialized navigation HUD.

## Dependencies
- `koad-core`: Shared types and config.
- `koad-proto`: gRPC client bindings.
- `koad-board`: Updates board integration.
- `koad-bridge-notion`: Cloud synchronization.
