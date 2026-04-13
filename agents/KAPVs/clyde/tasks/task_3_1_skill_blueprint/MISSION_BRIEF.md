# Mission Brief: 3.1 - Skill Blueprint Architecture & 3.2 - Vault Skill CLI
**Mission ID:** TASK-3.1-SKILL-BLUEPRINT / TASK-3.2-VAULT-SKILL-CLI
**Primary Assignee:** Clyde (Officer / Lead)
**Support Assets:** Cid (Engineer)
**Reviewer:** Tyr (Captain)
**Source Specifications:** 
- `agents/quests/stable_release/tasks/TASK_3_1_SKILL_BLUEPRINT.md`
- `agents/quests/stable_release/tasks/TASK_3_2_VAULT_SKILL_CLI.md`

---

## 🎯 Primary Objective
Formalize the KoadOS Skill System and implement the user-facing CLI tools for skill management. This dual-task mission is a critical infrastructure component for the v3.2.0 "Citadel Integrity" stable release.

## 🧱 Strategic Context
We are decoupling the *definition* of a skill (Distribution) from its *mastery* by an agent (Instance/Vault).
- **Blueprints** (Shared) define capabilities, runtimes, and entry points.
- **Instances** (Private) track levels, XP, and per-agent configurations.

## 🛠️ Technical Directives

### 1. Crate Architecture (`koad-core`)
- **Refactor `crates/koad-core/src/config.rs`:** 
    - Transition `SkillDefinition` into `SkillBlueprint`.
    - Implement `SkillInstance` and ensure `AgentIdentityConfig` supports a persistent `skills: Vec<SkillInstance>` field.
- **Implement a `CapabilityRegistry`:** Create a new module to validate and manage sandbox-recognized capability strings (e.g., `fs_read`, `network_out`).

### 2. Discovery & Scaffolding
- **Skill Scanner:** Implement discovery logic for `.toml` blueprints in `$KOAD_HOME/skills/`.
- **Vault Scaffolding:** Implement the automated creation of per-skill data directories in an agent's vault upon equipping.

### 3. Vault Skill CLI (`koad vault skill`)
- **`list`**: Show equipped skills, levels, and status.
- **`info <id>`**: Deep-dive into a skill's blueprint and instance data.
- **`search`**: Discover new blueprints available in the Citadel.
- **`equip <id>`**: **MANDATORY Confirmation Step.** Before equipping, the CLI MUST display all required capabilities and prompt the user for explicit approval.

## 📊 Knowledge Graph Integration
Use the **Dynamic System Map (DSM)** to identify existing code that should be refactored into "Builtin" skills.

## ✅ Verification & Validation
- **Serialization:** Must parse a `hello-world.skill.toml` correctly.
- **Persistence:** Skill instances must survive `KoadConfig::load()`.
- **CLI Fidelity:** `list`, `search`, and `equip` must be fully functional and atomic.

---
**Tyr:** "Clyde, the review is complete. I've expanded your scope to include the CLI layer (Task 3.2) to ensure the architecture and the user experience are developed in tight synchronization. Focus on the `CapabilityRegistry` early—it is the bedrock for our security sandboxing."
