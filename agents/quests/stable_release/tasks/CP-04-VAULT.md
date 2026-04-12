# Task Spec: CP-04-VAULT (Vault Phase 3: Skill Standardization)

**Mission:** KoadOS v3.2.0 "Citadel Integrity"
**Agent:** Clyde (Implementation Lead)
**Status:** TODO
**Branch:** `feature/vault-phase-3`

## 🎯 Objective
Formalize KoadOS's dynamic tool-loading system by implementing a two-tier architecture: **Skill Blueprints** (static, distribution-level capability definitions) and **Skill Instances** (agent-specific, configured instances of those capabilities). Integrate this architecture into the user workflow via the `koad vault skill` CLI.

## 🧱 Scope & Impact
- **Affected Crates:** `koad-core` (Data models, config parsing, file scanning), `koad-cli` (Vault subcommand routing, handlers).
- **Impact:** Agents will be able to dynamically discover (`search`), acquire (`equip`), and configure specialized tools (Skills). This moves capabilities out of hardcoded logic and into a pluggable, shareable ecosystem.

## 🛠️ Implementation Steps for Clyde

### Sprint 1: The Blueprint Data Model (`koad-core`)
1.  **Refactor `koad_core::config::SkillDefinition`:** Introduce the `SkillBlueprint` struct.
    *   **Fields:** `id`, `name`, `description`, `version`, `runtime` (WASM, Builtin, Remote), `entry_point`, `capabilities` (e.g., `["fs_read", "network_out"]`).
    *   **Storage:** Blueprints live as `.toml` files in the global `$KOAD_HOME/skills/` directory.
2.  **Define `SkillInstance`:** Represent an agent's personal mastery/configuration of a skill.
    *   **Fields:** `blueprint_id`, `level`, `current_xp`, `settings` (HashMap of configurations).
3.  **Update `AgentIdentityConfig`:** Add `skills: Vec<SkillInstance>` to `identities/*.toml` to track equipped skills. Ensure backward compatibility via `#[serde(default)]`.
4.  **Implement Skill Scanner:** Create a utility in `koad-core` to parse and load all `.toml` files in `$KOAD_HOME/skills/` into a registry of `SkillBlueprint`s.

### Sprint 2: The Vault Skill CLI (`koad-cli`)
1.  **Route Handler:** Update `crates/koad-cli/src/cli.rs` and `crates/koad-cli/src/handlers/vault.rs` to route the `VaultAction::Skill` subcommands.
2.  **`search` Subcommand:** Utilize the `Skill Scanner` to list all available blueprints in `$KOAD_HOME/skills/`. Highlight which ones the current agent hasn't equipped.
3.  **`equip <id>` Subcommand:**
    *   Find the blueprint by `<id>`.
    *   Display a confirmation prompt detailing the skill's required `capabilities` (Security Sandbox warning).
    *   Append a new `SkillInstance` to the active agent's `AgentIdentityConfig` and save the TOML.
    *   Scaffold vault directories: `~/.koad-os/agents/<agent>/skills/<id>/`.
4.  **`list` / `ls` Subcommand:** Read the agent's identity TOML and list all currently equipped `SkillInstance`s, resolving their names via the blueprint registry.
5.  **`info <id>` Subcommand:** Display detailed stats (XP, Level, Config, Capabilities) for a specific equipped skill.

## ✅ Verification Strategy
-   **Discovery:** Add a `hello-world.skill.toml` to `$KOAD_HOME/skills/` and verify `koad vault skill search` detects it.
-   **Equipping:** Run `koad vault skill equip hello-world`. Verify the prompt, check the agent's TOML for the new instance, and confirm the vault subdirectories were created.
-   **Introspection:** Run `koad vault skill ls` and `koad vault skill info hello-world` to verify correct data parsing and rendering.

## 🚫 Constraints
-   Ensure all modifications to `identities/*.toml` during the `equip` process are atomic to prevent corruption of the agent's core identity file.
-   "Dark Mode" Support: The CLI must operate directly on the local identity files if the Citadel gRPC backend is offline.
