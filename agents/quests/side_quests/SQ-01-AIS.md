# Task Spec: SQ-01-AIS (Agent Information System Refactor)

**Mission:** KoadOS Side Quests
**Agent:** Tyr / Clyde
**Status:** TODO
**Priority:** High (Identity & Command Cohesion)

## 🎯 Objective
Establish a single, authoritative "Crew Contract" (`CREW.md`) to resolve identity and rank confusion among agents. Implement an Agent Information System (AIS) within the Citadel Kernel that dynamically parses this manifest, updates the active Redis state (`crew_manifest`), and ensures every agent receives accurate hierarchy data during their hydration phase.

## 🧱 Scope & Impact
- **Affected Areas:** `CREW.md` (new file), `koad-core/src/identity.rs`, `koad-citadel/src/kernel.rs` (AIS Watcher), `koad-cli/src/bin/koad-agent.rs` (Hydration logic).
- **Impact:** Agents (specifically Tyr) will no longer hallucinate ranks or misidentify the User (Admiral). The CLI status board will reflect accurate, live data driven by a human-readable markdown manifest.

## 🛠️ Implementation Steps

### 1. Formalize the Crew Manifest (`CREW.md`)
- Create a definitive `CREW.md` file in the root of `.koad-os`.
- **Structure:**
  - Define the "Chain of Command" explicitly.
  - Define "The Admiral" (User) as the ultimate strategic authority.
  - Detail specific roles for active agents (Tyr as Captain/PM, Clyde as Implementation Lead, etc.).
- Delete or archive the stale `agents/crews/CITADEL_JUPITER.md`.

### 2. Refactor `koad_core::identity::Rank`
- Update the `Rank` enum in `crates/koad-core/src/identity.rs` to perfectly match the new hierarchy:
  - `Admiral` (User / Fleet Commander)
  - `Captain` (Project Manager / Tyr)
  - `Officer` (Domain Leads)
  - `Engineer` (Implementation)
  - `Crew` (General tasks)

### 3. Implement the AIS Parser (`koad-core`)
- Create a new module: `koad-core/src/ais.rs`.
- Implement a parser that reads `CREW.md` and extracts the roster into a structured data format (`Vec<CrewMember>`).
- Implement logic to merge the static roster from `CREW.md` with dynamic session data (Wake/Dark status) from `CitadelSessionService`.

### 4. Citadel Kernel AIS Watcher (`koad-citadel`)
- In `crates/koad-citadel/src/kernel.rs`, implement a background task (the "AIS Watcher").
- This task periodically (or via file-watch) parses `CREW.md`.
- It queries the `CitadelSessionService` for active sessions.
- It compiles the complete "Live Manifest" and writes it to the \`koad:state\` hash under the key \`crew_manifest\` in Redis, replacing the current empty/stale logic.

### 5. Hydration Injection (`koad-cli`)
- In `crates/koad-cli/src/bin/koad-agent.rs`, update the `anchor_content` generation.
- The `GEMINI.md` / `CLAUDE.md` generation must read a summary from the AIS (or directly inject the "Chain of Command" from `CREW.md`) to ensure the agent understands who the User is upon every boot.

## ✅ Verification Strategy
- Run `koad system status --full` and verify the "Crew Manifest" section correctly lists the agents from `CREW.md` and their live "WAKE/DARK" statuses.
- Boot an agent (`koad-agent boot tyr`) and verify the generated `GEMINI.md` explicitly lists the User as the "Admiral" and the agent's correct rank.
