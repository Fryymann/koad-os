# Task Manifest: 1.3 - The Admiral's Guide (Onboarding Documentation)
**Status:** ⚪ Draft
**Assignee:** [Scribe/Tyr/Cid]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `docs/onboarding-refresh`

---

## 🎯 Objective
Rewrite and modernize the core onboarding documentation (`MISSION.md`, `AGENTS.md`, and `README.md`) to reflect the v3.2.0 "Sanctuary" architecture. Ensure a new user can go from `git clone` to "First Boot" without external help.

## 🧱 Context
Documentation is the "UX of the codebase." As we move to a shareable release, we must remove developer-specific jargon and outdated instructions. The guide must be authoritative, inspiring, and technically precise.

## 🛠️ Technical Requirements

### 1. The "Quick Start" Protocol (`README.md`)
- **Requirement:** Create a high-impact, three-step "Ignition" guide:
    1. `git clone ...`
    2. `bash install/bootstrap.sh`
    3. `agent-boot tyr`
- **Requirement:** Clearly list hardware/software prerequisites (WSL2, Docker, Rust).

### 2. The Agent Portal (`AGENTS.md`)
- **Requirement:** Redesign as a "Command Hub." 
- **Content:**
    - Explain the "Ghost-Body" duality (AI Ghost + Host Body).
    - List the core Crew (Tyr, Clyde, Cid, etc.) with their specialized roles.
    - Document the `agent-boot` command and how identities are resolved.

### 3. The Prime Directive (`MISSION.md`)
- **Requirement:** Update the vision statement for v3.2.0.
- **Content:**
    - Explain the "Sovereign Planning Protocol."
    - Define the "Sanctuary" standard (Privacy, Portability, Performance).
    - Document the "Hierarchy of Intent" (Agenda -> Roadmap -> Tasks -> Specs).

### 4. Technical Reference Update
- **Requirement:** Ensure all paths in documentation use `$KOAD_HOME` notation.
- **Requirement:** Verify that all documented CLI commands (e.g., `koad status`, `koad map`) are accurate to the latest `koad-cli` build.

### 5. Aesthetic & Structure
- **Requirement:** Use GitHub-flavored Markdown with consistent headers and tables.
- **Requirement:** Incorporate ASCII art or Mermaid diagrams to visualize the "Citadel" architecture (Bays, Sockets, Bus).

## ✅ Verification Strategy
1.  **Fresh-Eye Review:** Ask a "Crew" agent (like Scribe) to follow the Quick Start instructions on a mock clean environment.
2.  **Broken Link Check:** Ensure all cross-references between MD files are valid.
3.  **Command Verification:** Every command block in the docs MUST be copy-pasteable and functional.

## 🚫 Constraints
- **NO** hardcoded local paths (`/home/ideans`).
- **NO** stale instructions from the v2.x "Spine" era.
- **MUST** maintain the "Officer/Captain" professional tone.

---

## 🛰️ Sovereign Review (Tyr)
- Verify that the distinction between "Distribution" (code) and "Instance" (data) is clearly explained.
- Ensure the "Sanctuary Compliance" standards are prominently documented for future contributors.
