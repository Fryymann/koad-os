# Mission Brief: 3.1 - Skill Blueprint Architecture
**Mission ID:** TASK-3.1-SKILL-BLUEPRINT
**Primary Assignee:** Clyde (Officer / Lead)
**Support Assets:** Cid (Engineer)
**Reviewer:** Tyr (Captain)
**Source Specification:** `agents/quests/stable_release/tasks/TASK_3_1_SKILL_BLUEPRINT.md`

---

## 🎯 Primary Objective
Formalize the KoadOS Skill System by implementing a two-tier architecture: **Skill Blueprints** (distribution-level templates) and **Skill Instances** (agent-specific configurations). This is a critical infrastructure component for the v3.2.0 "Citadel Integrity" stable release.

## 🧱 Strategic Context
Currently, KoadOS skills are mere metadata. To achieve true agent specialization and secure capability management, we must decouple the *definition* of a skill from its *mastery* by an agent.
- **Blueprints** define the skill's name, version, runtime (WASM/Builtin/Remote), and required capabilities.
- **Instances** track an agent's level, XP, and specific configurations for that skill within their vault.

## 🛠️ Technical Directives

### 1. Crate Architecture (`koad-core`)
- **Refactor `crates/koad-core/src/config.rs`:** 
    - Transition `SkillDefinition` into `SkillBlueprint`.
    - Implement the `SkillInstance` struct for agent-specific state.
    - Ensure `AgentIdentityConfig` can persist a `Vec<SkillInstance>`.

### 2. Discovery Logic (The "Skill Scanner")
- **Implement a Blueprint Scanner:** Add a module in `koad-core` to discover and parse `.toml` skill blueprints from `$KOAD_HOME/skills/`.

### 3. Security & Sandboxing (`koad-sandbox`)
- **Capability Mapping:** Ensure the `capabilities` defined in a Blueprint can be mapped to the `koad-sandbox` security policies to prevent unauthorized file or network access.

## 📊 Knowledge Graph Integration
Leverage the newly implemented **Dynamic System Map (DSM)** via `code-review-graph`. Use the graph to identify existing logic clusters that are candidates for refactoring into "Builtin" skills.

## ✅ Verification & Validation
- **Serialization:** Must parse a `hello-world.skill.toml` correctly.
- **Persistence:** Agent skill instances must survive `KoadConfig::load()`.
- **Discovery:** The Scanner must detect all blueprints in the designated folder.

---
**Tyr:** "Clyde, the architecture is ready for your lead. Focus on the core data structures and discovery paths first. Cid is available for gRPC/Proto support if required."
