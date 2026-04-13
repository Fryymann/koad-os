# Task Manifest: 3.2 - `koad vault skill` Implementation
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Cid/Clyde)]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `feature/cli-vault-skill`

---

## 🎯 Objective
Complete the implementation of the `koad vault skill` CLI command group. This command allows agents and users to discover, inspect, and manage the skills associated with an agent's identity and vault.

## 🧱 Context
While the architecture for skills was defined in Task 3.1, users need a way to interact with them through the CLI. This task bridges the gap between the `KoadConfig` data structures and the user's operational workflow.

## 🛠️ Technical Requirements

### 1. Implement `VaultAction::Skill` Handler
- **File:** `crates/koad-cli/src/handlers/vault.rs`
- **Requirement:** Implement a `handle_vault_skill_action` function to process the `VaultAction::Skill` subcommands.

### 2. Subcommand: `koad vault skill list` (or `ls`)
- **Requirement:** List all skills currently "Equipped" by the active agent (from their `AgentIdentityConfig`).
- **Output:** A formatted table showing: `Skill ID`, `Name`, `Level`, and `Status` (Active/Inactive).

### 3. Subcommand: `koad vault skill info <id>`
- **Requirement:** Display detailed information about a specific skill.
- **Output:** 
    - Full name and description.
    - Required runtime (WASM/Remote).
    - Current XP and Level.
    - Capabilities (e.g., `fs_read`, `signal_send`).
    - Configured settings for this instance.

### 4. Subcommand: `koad vault skill search`
- **Requirement:** List all globally available **Skill Blueprints** found in the `$KOAD_HOME/skills/` directory that are NOT yet equipped by the agent.
- **Goal:** Allow the user to discover new capabilities they can add to their agent.

### 5. Subcommand: `koad vault skill equip <id>`
- **Requirement:** Add a new `SkillInstance` (based on a blueprint) to the agent's identity TOML.
- **Requirement:** **Confirmation Step:** Before equipping, the CLI MUST display a clear list of the skill's required `capabilities` and prompt the user/agent for explicit approval (e.g., "This skill requires access to: fs_read, network_out. Proceed? [y/N]").
- **Requirement:** Scaffolding: Create any required directories in the agent's vault (e.g., `~/.koad-os/agents/tyr/skills/<skill_id>/`).

### 6. Subcommand: `koad vault skill sync`
- **Requirement:** Synchronize the local skill state with the Citadel (if online). This ensures that XP gained during a session is persisted back to the identity TOML.

### 7. Capability Registry Integration
- **Requirement:** Integrate with the `CapabilityRegistry` (to be implemented in Task 3.1) to ensure only valid, sandbox-recognized capabilities are displayed and requested during the `equip` phase.

## ✅ Verification Strategy
1.  **List Verification:** Equip a mock skill and verify it appears in `koad vault skill ls`.
2.  **Search Verification:** Place a new `.toml` blueprint in the global `skills/` folder and verify `koad vault skill search` finds it.
3.  **Equip Verification:** Run `koad vault skill equip` and verify that the agent's identity TOML is updated and the vault directories are created.

## 🚫 Constraints
- **MUST** handle cases where the Citadel is offline ("Dark Mode") by operating directly on the local identity TOML.
- **NO** destructive changes to existing identity data if the equip fails (atomic updates).

---

## 🛰️ Sovereign Review (Tyr)
- Confirm that the `ls` output is clean and provides high-signal data.
- Verify that the `equip` process includes a confirmation step explaining the skill's required `capabilities`.
