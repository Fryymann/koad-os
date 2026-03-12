# Rigorous Documentation Protocol (RDP)

## 1. The Visual Mandate
Code alone is insufficient. Human administrators must be able to visually "steer the ship." Therefore, **all major architectural components MUST have a corresponding Mermaid.js visualization** mapping their internal flow and external connections.

## 2. Directory Structure over Monoliths
Architecture documentation will not be a single monolithic `ARCHITECTURE.md` file. It will be a directory tree (`docs/architecture/`) segmented by layer:
- `docs/architecture/00_TOP_LEVEL.md`: The macro organism (The Dual-Bus Perspective).
- `docs/architecture/control_plane/`: Spine, Command Manager, Protocol Enforcers.
- `docs/architecture/data_plane/`: Redis structures, Hot Memory registries, Context Orchestrator.
- `docs/architecture/agents/`: ASM, Micro-Agents, Agent "LinkedIn" profiles.
- `docs/architecture/interfaces/`: Edge Gateway, Web Deck, TUI.

## 3. The "Design Before Build" Law
Whenever Koad and the Admin ponder a change or addition to KoadOS, the architecture MUST be strongly considered first. 
1. The relevant `docs/architecture/...` file must be loaded into context.
2. The visual diagram must be updated to reflect the proposed change.
3. The change must be reviewed against the Top-Level Dual-Bus architecture to ensure it does not break the CQRS (Command/Query) separation.
4. Only upon approval of the *diagram* does code modification begin.

## 4. Documentation is Code
Documentation files are treated with the same severity as `.rs` files. A PR or Sprint is not "Condition Green" until the visual maps and component definitions are fully synced with the implementation.
