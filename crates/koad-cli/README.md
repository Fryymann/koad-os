# koad (cli)

The primary command-line interface for the **KoadOS** ecosystem. It serves as the bridge between human Admiral "Dood" and the KAI Officers.

## 🏗 Overview

The `koad` CLI is a versatile tool for session management, system orchestration, memory retrieval, and project synchronization. It handles both high-level agent directives and low-level kernel operations.

## 🛠 Features

- **`boot`**: Wake and hydrate agent sessions.
- **`system`**: Core management (init, auth, config, save, patch, audit).
- **`intel`**: Query and commit to the durable memory bank.
- **`fleet`**: Manage the GitHub Project Board (Command Deck) and local projects.
- **`bridge`**: Orchestrate cloud integrations (GCP, Notion, Airtable).
- **`status`**: Real-time telemetry and health monitoring.
- **`dash`**: A TUI-based command center for the fleet.

## 🔑 Key Modules

- **`cli`**: The clap-based command and argument definitions.
- **`handlers/`**: The logic implementations for each CLI command.
- **`tui`**: The Ratatui-powered terminal dashboard.
- **`db`**: Local SQLite access layer for project-aware persistence.
- **`utils`**: Path resolution, pre-flight checks, and environment detection.

## ⚙️ Installation

The CLI binary is typically located in `~/.koad-os/bin/koad`.

```bash
# Build and install locally
cargo build --release
cp target/release/koad bin/koad
```

## 🚀 Usage

```bash
# Display help
koad --help

# Boot an agent
koad boot --agent Sky --project
```
