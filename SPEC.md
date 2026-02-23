# KoadOS v2.0 Technical Specification

## 1. Schema Definition (`koad.json`)

| Field | Type | Description |
| :--- | :--- | :--- |
| `version` | String | Semantic version of the koadOS schema. |
| `identity` | Object | Persona attributes (name, role, bio). |
| `preferences` | Object | Tech stack and behavioral principles. |
| `memory` | Object | Persistent knowledge storage. |
| `drivers` | Map | Agent-specific configuration hooks. |

### Memory Object
- `global_facts`: Array of `Fact` objects.
- `Fact`: `{ "id": "f_1", "text": "...", "timestamp": "YYYY-MM-DD" }`

---

## 2. CLI Reference (`koad`)

### `koad boot [--agent <name>]`
- **Purpose**: Generates a system-level context block for ingestion.
- **Logic**: Reads `koad.json`, detects current directory for Auth selection, and reverses memory order to prioritize recent facts.

### `koad fact "<text>"`
- **Purpose**: Appends knowledge to the persistent memory ledger.
- **Logic**: Increments ID, generates timestamp, and performs a safe JSON overwrite.

### `koad auth`
- **Purpose**: Manual check of environment-aware authentication.
- **Rules**: 
    - `path.contains("skylinks")` -> `GITHUB_SKYLINKS_PAT`
    - Else -> `GITHUB_PERSONAL_PAT`

### `koad skill list`
- **Purpose**: Lists all available skills in the `skills/` directory tree.

### `koad skill run <name> [args]`
- **Purpose**: Dispatches execution to a skill script.
- **Path Resolution**: Looks in `~/.koad-os/skills/<name>`.

---

## 3. Implementation Details

- **Language**: Rust (edition 2021).
- **Binary Path**: `~/.koad-os/core/rust/target/release/koad`
- **Skills Root**: `~/.koad-os/skills/`
- **Dependencies**: 
    - `serde`: Serialization/Deserialization.
    - `clap`: CLI argument parsing.
    - `dirs`: Cross-platform path resolution.
    - `chrono`: Local time management.

---

## 4. Development & Testing
- **Test Command**: `cargo test`
- **Code Style**: Idiomatic Rust with full doc-comment coverage.
- **Validation**: Every `koad fact` call triggers a serialization check to prevent memory corruption.
