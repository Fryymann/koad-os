# KoadOS v3.2 Technical Specification

## 1. Schema Definition (`config/`)

KoadOS utilizes a multi-file TOML Registry instead of a monolithic JSON blob.
- **`kernel.toml`**: System architecture defaults.
- **`registry.toml`**: High-level project and credential mappings.
- **`identities/*.toml`**: Granular agent persona and authorization definitions.
- **`interfaces/*.toml`**: Host-specific engine configurations.
- **`filesystem.toml`**: Physical-to-Virtual path translations for the Sandbox.

---

## 2. CLI Reference (`koad`)

### `koad boot [--project] [--compact] [--task <id>]`
- **Purpose**: Generates a system-level context block.
- **Project Awareness**: Automatically detects the active project via database lookup if the current path matches a registered project.
- **Compact Mode**: Outputs a dense, label-free format for minimal token usage.

### `koad host [--port <p>] [--dir <d>]`
- **Purpose**: [AGENT/HUMAN] Host a local web server for testing front-ends or the Koad Dashboard.
- **WebSockets**: Includes a `/ws` endpoint for live event broadcasting from the Koad Spine.
- **Default**: Serves `~/.koad-os/data/dashboard/` if no directory is specified.

### `koad serve [--stop]`
- **Purpose**: [ADMIN] Manage the Koad background service (Cognitive Booster).
- **PID Tracking**: Automatically creates and removes `~/.koad-os/daemon.pid` for elegant process management.
- **Live Updates**: Connects to the local `koad host` server to push real-time file change events and task statuses to the dashboard.
- **Logic**: Prevents multiple instances and allows for clean shutdown via `--stop`.

### `koad scan [<path>]`
- **Purpose**: Database-aware project registration.
- **Logic**: Searches for a `.koad` directory. If found, registers the project name and absolute path in the `projects` table for instant contextual booting.

### `koad publish [--message <msg>]`
- **Purpose**: ADMIN ONLY. Stages, commits, and pushes the KoadOS repository to GitHub.
- **Logic**: Executes `git add`, `git commit`, and `git push` from the `KOAD_HOME` directory.

### `koad saveup "<summary>" [--scope <scope>] [--facts "<f1,f2>"] [--auto]`
- **Purpose**: ADMIN/PM ONLY. Native session closure.
- **Automation**: 
    - Appends to `~/.koad-os/SESSION_LOG.md`.
    - Commits manual facts to `koad.db`.
    - **Auto-Harvest**: If `--auto` is provided, queries the `executions` table for recent successful commands and automatically converts them into verified facts.

### `koad gcloud <action>`
- **Actions**:
    - `list [--resource <type>]`: Lists Cloud Functions (default), Run services, or IAM policies.
    - `deploy <name>`: Dispatches deployment to the active operations project.
    - `logs <name> [--limit <n>]`: Reads logs for a specific resource.
    - `audit [--project <id>]`: Performs an IAM audit and broadcasts the summary to the Koad Stream.

### `koad retire <id>`
- **Purpose**: ADMIN ONLY. Deactivates a knowledge entry so it no longer appears in `boot` or `query` results.
- **Logic**: Sets `active = 0` in the database.

### `koad remember <category> "<text>" [--tags "<tags>"]`
- **Purpose**: Appends knowledge to the persistent memory ledger.
- **Categories**: `fact` (immutable truth), `learning` (session lesson).

### `koad ponder "<text>" [--tags "<tags>"]`
- **Purpose**: Record a personal reflection or interpretation (Persona Journaling).
- **Persistence**: Entries are stored in the database under the `pondering` category and are automatically surfaced during the `koad boot` sequence.

### `koad sync <source>`
- **Purpose**: Triggers data synchronization from external platforms.
- **Sources**: 
    - `airtable [--schema-only] [--base-id <id>]`
    - `notion [--page-id <id>] [--db-id <id>]`
- **Implementation**: Dispatches to global Python skills for flexible networking.

### `koad drive <action>`
- **Purpose**: Google Drive operations with path-aware authentication.
- **Actions**:
    - `list [--shared]`: Lists files in personal or shared drives.
    - `download <id> [--dest <path>]`: Downloads a specific file.
    - `sync`: Syncs metadata/files to local cache.
- **Authentication**: 
    - Defaults to `GDRIVE_PERSONAL_TOKEN`. Users can configure path-aware tokens in their environment.

### `koad harvest <path>`
- **Purpose**: ADMIN/PM ONLY. Pulls discoveries from developer documentation.
- **Trigger**: Looks for `## Discoveries` or `## Learnings` headers in Markdown files.

### `koad skill <action>`
- **List**: Lists all scripts in `~/.koad-os/skills/`.
- **Run <name> -- [args]**: Dispatches execution to a script with trailing argument support.

### `koad template <action>`
- **List**: Lists available project workflow templates in `~/.koad-os/templates/project_flow/`.
- **Use <name> [--out <path>]**: Copies a template to the current directory for project initialization.

### `koad query <term>`
- **Purpose**: Case-insensitive search across facts and learnings.

---

## 3. Database Schema (`koad.db`)

KoadOS uses a local SQLite database for fast, searchable persistence.

### Table: `knowledge`
Tracks facts and learnings with contextual tagging.
- `id`: Integer Primary Key
- `category`: Text (fact, learning)
- `content`: Text
- `tags`: Text (comma-separated)
- `timestamp`: Text
- `active`: Integer (Boolean toggle)

### Table: `projects`
Tracks local project directories and metadata.
- `id`: Integer Primary Key
- `name`: Text (Unique)
- `path`: Text (Absolute path)
- `role`: Text
- `stack`: Text
- `last_boot`: Text

### Table: `executions`
Maintains an audit trail of all Koad actions.
- `id`: Integer Primary Key
- `command`: Text (e.g., skill run, gcloud)
- `args`: Text
- `timestamp`: Text
- `status`: Text (success, failed, pending)

---

## 4. Environment Variables (Personalization)

| Variable | Description |
| :--- | :--- |
| `KOAD_NAME` | Overrides persona name. |
| `KOAD_ROLE` | Overrides persona title. |
| `KOAD_BIO` | Overrides persona background. |
| `KOAD_HOME` | Custom root for koadOS (default `~/.koad-os`). |
| `AIRTABLE_TOKEN` | Auth for Airtable sync. |
| `NOTION_TOKEN` | Auth for Notion sync. |

---

## 5. Development & Testing
- **Language**: Rust 2021.
- **Test Suite**: `cargo test` (Includes auth, serialization, and harvest logic).

---

# v4.0.x Technical Specifications

## v4.0.x Core: Unix-Based Agent Isolation (System Native) [STATUS: ABORTED/PIVOTED]

### 1. Objective (Historical)
Achieve absolute process and filesystem isolation for KoadOS agents by utilizing native Unix users.

### 2. Decision Summary (2026-03-01)
- **Outcome**: The implementation was fully developed but subsequently **aborted** following a formal **Anti-Overengineering Protocol (AOP)** review. 
- **Rationale**: Unix-user isolation failed all four AOP pillars:
    1. **Relevance**: Host-level users are human-centric artifacts, not agentic ones.
    2. **Value**: Security gain was offset by extreme brittle-state cost in E2E tests.
    3. **Utility**: Native user-space toolchains (Cargo/NVM) were broken by isolation.
    4. **YAGNI**: We don't need OS-level users for ephemeral micro-agents.
- **Pivot**: KoadOS will remain a **Consolidated User-Space Architecture**. Isolation will be handled via software-level sandboxing (Sandbox policy) and future process-level isolation using lightweight user-space tools (e.g., `bwrap`).
- **Benefits**: Improved agility, easier testing, and better alignment with the dynamic lifecycle of AI Micro-Agents.

### 3. Cleanup Complete
- [x] `koad-setup` utility removed.
- [x] `sudo` wrapping removed from `DirectiveRouter`.
- [x] Directory paths reverted to `~/.koad-os/`.
- [x] E2E Tests updated and passing in user-space.
