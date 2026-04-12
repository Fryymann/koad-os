# Task Manifest: 3.1 - Skill Blueprint Architecture
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Cid/Clyde)]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `feature/skill-blueprint-v1`

---

## 🎯 Objective
Formalize the KoadOS Skill System by implementing a two-tier architecture: **Skill Blueprints** (distribution-level templates) and **Skill Instances** (agent-specific configurations). 

## 🧱 Context
Skills are currently just metadata. We need a way for an agent to "equip" a skill that actually grants them access to new gRPC methods, CLI subcommands, or automated workflows. A Blueprint defines *what* a skill is; an Instance defines *how* an agent has mastered it.

## 🛠️ Technical Requirements

### 1. Refactor `SkillDefinition` (`koad-core/src/config.rs`)
- **Requirement:** Update `SkillDefinition` to include execution metadata:
    ```rust
    pub struct SkillBlueprint {
        pub id: String,
        pub name: String,
        pub description: String,
        pub version: String,
        pub runtime: SkillRuntimeType, // WASM, Builtin, Remote
        pub entry_point: String,     // Path to WASM or remote URL
        pub capabilities: Vec<String>, // e.g., ["fs_read", "network_out"]
    }
    ```
- **Requirement:** Add `SkillInstance` for agent-specific progress:
    ```rust
    pub struct SkillInstance {
        pub blueprint_id: String,
        pub level: u32,
        pub current_xp: u32,
        pub settings: HashMap<String, String>, // Per-agent skill config
    }
    ```

### 2. Update `AgentIdentityConfig`
- **Requirement:** Add a `skills: Vec<SkillInstance>` field to track which skills an agent currently possesses.

### 3. Skill Repository Discovery
- **Requirement:** Implement a "Skill Scanner" in `koad-core`. It should look for `.toml` blueprints in the `$KOAD_HOME/skills/` directory.

### 4. WASM Capability Mapping
- **Requirement:** Ensure that when a skill is "Equipped," its `capabilities` are validated against the `koad-sandbox` security policy.

## ✅ Verification Strategy
1.  **Serialization Test:** Create a `hello-world.skill.toml` blueprint and verify that `KoadConfig` can parse it.
2.  **Equip Test:** Programmatically add a `SkillInstance` to the `tyr.toml` identity and verify it survives a `KoadConfig::load()`.
3.  **Discovery Test:** Verify that the "Skill Scanner" finds and returns a list of all Blueprints in the `skills/` folder.

## 🚫 Constraints
- **NO** breaking changes to the existing `identities/*.toml` files; use optional fields or smart defaults for the new `skills` field.
- **MUST** support the "Remote" runtime type for future Cloud integration.

---

## 🛰️ Sovereign Review (Tyr)
- Confirm that the `capabilities` list provides enough granularity for security sandboxing.
- Verify that `SkillBlueprint` is shareable across Citadels, while `SkillInstance` remains private to the agent vault.
