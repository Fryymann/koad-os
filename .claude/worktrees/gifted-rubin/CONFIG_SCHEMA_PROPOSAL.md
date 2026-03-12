# KoadOS Unified Configuration Schema Proposal

**Goal:** Centralize all system parameters, identity definitions, and project mappings into a single, human-readable configuration file (e.g., `koad.toml` or `koad.yaml`) to eliminate hardcoded Rust values and avoid requiring rebuilds for environment or variable changes.

## Proposed TOML Schema (`koad.toml`)

```toml
[system]
version = "4.0"
# Base paths for the system; avoids hardcoding in Rust
home_dir = "~/.koad-os"
log_dir = "~/.koad-os/logs"
db_path = "~/.koad-os/koad.db"

[network]
# Replaces hardcoded addresses and ports in constants.rs
gateway_addr = "0.0.0.0:3000"
spine_grpc_addr = "http://127.0.0.1:50051"
redis_socket = "~/.koad-os/koad.sock"
spine_socket = "~/.koad-os/kspine.sock"

[github]
# Replaces DEFAULT_GITHUB_OWNER / DEFAULT_GITHUB_REPO
default_owner = "DoodzCode"
default_repo = "koad-os"
api_base = "https://api.github.com"

[projects]
# Replaces the hardcoded `if current_dir.contains("skylinks")` block in main.rs
# The CLI will iterate these to detect the active project context based on current path.
[projects.koados]
path = "~/.koad-os"
github_owner = "DoodzCode"
github_repo = "koad-os"
default_project = 2
credential_key = "GITHUB_PERSONAL_PAT"

[projects.skylinks]
path = "~/data/skylinks"
github_owner = "Skylinks-Golf"
# If github_repo is omitted, it dynamically resolves to the current directory name
dynamic_repo_resolution = true 
default_project = 2
credential_key = "GITHUB_SKYLINKS_FULLACCESS_TOKEN"

[identity]
# Default agent identity if not overridden by boot flags
name = "Tyr"
role = "Captain"
bio = "Flagship KoadOS Agent. Principal Systems & Operations Engineer..."

[preferences]
languages = ["Rust", "Node.js", "Python"]
booster_enabled = true
style = "programmatic-first"
principles = [
    "Optimized Kernel: Simplicity over complexity. Don't waste cycles on dead code.",
    "Uplink Protocols: Plan before build. Blind jumps into the grid cause system crashes."
]

[drivers]
# Model/Driver specific tool configurations and bootstrap paths
[drivers.gemini]
bootstrap = "~/.koad-os/drivers/gemini/BOOT.md"
mcp_enabled = true
tools = ["save_memory", "google_web_search", "run_shell_command", "read_file", "write_file"]

[drivers.claude]
bootstrap = "~/.koad-os/drivers/claude/BOOT.md"
tier = 1
mcp_enabled = false
tools = ["Bash", "Read", "Write", "Edit"]

[notion]
api_base = "https://api.notion.com/v1"
mcp_enabled = true

[notion.index]
koad = "30cfe8ec-ae8f-808b-8eff-fa75e1cb0572"
stream = "310fe8ec-ae8f-8046-9172-000bfe5966cd"
projects = "1f4300ee-bd64-41cf-a85e-94e8ee7e485e"

[filesystem]
workspace_symlink = "~/data"

[filesystem.mappings]
data = "/mnt/c/data"
projects = "/mnt/c/data/projects"
skylinks = "/mnt/c/data/skylinks"

[integrations.airtable.index.sgc_members]
base_id = "app0aisi1RwbK8yhM"
table = "SGC Members"
```

## Architectural Shift
1. **Unification:** Merge `KoadConfig` (currently environment variables) and `KoadLegacyConfig` (currently `koad.json`) into a single `koad_core::config::Config` struct using the `serde` and `figment` or `config` crates.
2. **Path Context:** Instead of hardcoding path checks in `main.rs`, the core boot logic will iterate through `config.projects` to determine context variables dynamically.
3. **No Hardcoded Defaults:** Values in `constants.rs` become fallback values in the struct implementation (`Default` trait) rather than hardcoded rules, allowing users to override anything via the configuration file without recompiling KoadOS.