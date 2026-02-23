# KoadOS v2.0 Technical Specification

## 1. Schema Definition (`koad.json`)

| Field | Type | Description |
| :--- | :--- | :--- |
| `version` | String | Semantic version of the koadOS schema. |
| `identity` | Object | Persona attributes (name, role, bio). |
| `preferences` | Object | Tech stack and behavioral principles. |
| `memory` | Object | Persistent knowledge storage (facts, learnings). |
| `drivers` | Map | Agent-specific configuration hooks. |

---

## 2. CLI Reference (`koad`)

### `koad boot [--project] [--compact]`
- **Purpose**: Generates a system-level context block.
- **Compact Mode**: Outputs a dense, label-free format for minimal token usage.
- **Logic**: Ingests identity, recent memory, and optionally a project snapshot.

### `koad saveup "<summary>" [--scope <scope>] [--facts "<f1,f2>"]`
- **Purpose**: PM ONLY. Native session closure.
- **Automation**: 
    - Appends to `~/.koad-os/SESSION_LOG.md`.
    - Commits facts to `koad.db`.
    - Auto-tags facts based on scope.

### `koad retire <id>`
- **Purpose**: PM ONLY. Deactivates a knowledge entry so it no longer appears in `boot` or `query` results.
- **Logic**: Sets `active = 0` in the database.

### `koad remember <category> "<text>" [--tags "<tags>"]`
- **Purpose**: Appends knowledge to the persistent memory ledger.
- **Categories**: `fact` (immutable truth), `learning` (session lesson).

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
    - `path.contains("skylinks")` -> `GDRIVE_SKYLINKS_TOKEN`
    - Else -> `GDRIVE_PERSONAL_TOKEN`

### `koad harvest <path>`
- **Purpose**: PM ONLY. Pulls discoveries from developer documentation.
- **Trigger**: Looks for `## Discoveries` or `## Learnings` headers in Markdown files.

### `koad skill <action>`
- **List**: Lists all scripts in `~/.koad-os/skills/`.
- **Run <name> -- [args]**: Dispatches execution to a script with trailing argument support.

### `koad query <term>`
- **Purpose**: Case-insensitive search across facts and learnings.

---

## 3. Environment Variables (Personalization)

| Variable | Description |
| :--- | :--- |
| `KOAD_NAME` | Overrides persona name. |
| `KOAD_ROLE` | Overrides persona title. |
| `KOAD_BIO` | Overrides persona background. |
| `KOAD_HOME` | Custom root for koadOS (default `~/.koad-os`). |
| `AIRTABLE_TOKEN` | Auth for Airtable sync. |
| `NOTION_TOKEN` | Auth for Notion sync. |

---

## 4. Development & Testing
- **Language**: Rust 2021.
- **Test Suite**: `cargo test` (Includes auth, serialization, and harvest logic).
