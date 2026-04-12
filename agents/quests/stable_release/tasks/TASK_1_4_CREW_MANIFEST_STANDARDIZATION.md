# Task Manifest: 1.4 - Crew Manifest Standardization
**Status:** ⚪ Draft
**Assignee:** [Tyr/Scribe]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `refactor/crew-manifest-standardization`

---

## 🎯 Objective
Decouple the "Instance" crew manifest (specific to our Citadel) from the "Distribution" template. Ensure `agents/crews/TEMPLATE.md` is generic and shareable, while `agents/crews/CITADEL_JUPITER.md` holds our active personnel.

## 🧱 Context
Currently, the template includes specific names (Tyr, Clyde, etc.) that belong to our deployment. A new user should receive a clean template with placeholders to define their own crew, maintaining the "Sanctuary" standard of separation between platform and instance.

## 🛠️ Technical Requirements

### 1. Template Refactor (`agents/crews/TEMPLATE.md`)
- **Requirement:** Redact all specific agent names, roles, and ranks.
- **Requirement:** Use descriptive placeholders (e.g., `[AgentName]`, `[Rank]`, `[Runtime]`, `[Role/Focus]`).
- **Requirement:** Provide 2-3 generic examples of common roles (e.g., "Primary Engineer", "Security Officer") for guidance.
- **Requirement:** Update the "Operational Protocols" section to be environment-agnostic.

### 2. Instance Verification (`agents/crews/CITADEL_JUPITER.md`)
- **Requirement:** Ensure `CITADEL_JUPITER.md` contains the full, accurate current manifest of the KoadOS crew.
- **Requirement:** This file remains local/ignored or specifically tracked as the "Canonical Instance" (Jupiter).

### 3. Documentation Alignment
- **Requirement:** Update `AGENTS.md` and `MISSION.md` to explain how to use the template to initialize a new Citadel crew.
- **Requirement:** Document the `koad crew init` (or manual process) for copying the template to an active manifest.

## ✅ Verification Strategy
1.  **Readability Check:** Verify that the new `TEMPLATE.md` is intuitive for a first-time user.
2.  **Integrity Check:** Ensure `CITADEL_JUPITER.md` was not accidentally modified during the template refactor.
3.  **Boot Verification:** Confirm that agent booting logic doesn't depend on the literal content of the `TEMPLATE.md` file.

## 🚫 Constraints
- **NO** hardcoded personal names in the distribution template.
- **NO** environment-specific paths in the manifest protocols.

---

## 🛰️ Sovereign Review (Tyr)
- Confirm that the new template reflects the v3.2.0 "Ghost-Body" architecture.
- Verify that the instructions for initializing a crew are clear and concise.
