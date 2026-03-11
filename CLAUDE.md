# KoadOS Kernel — Core Development Context

This context is for development of the KoadOS Spine, Engine Room, and Sentinel.

## Tech Stack
- **Engine Room:** Redis (Hot Stream state management)
- **Backbone:** Rust (High-performance kspine bus)
- **Memory Bank:** SQLite (Long-term persistent storage)
- **Interface:** Bash/Python (CLI tools and bridges)

## Architectural Constraints
- **One Body, One Ghost:** Never allow session leakage. Every turn must be uniquely identifiable via `KOAD_SESSION_ID`.
- **Sanctuary Rule:** Core system files in `~/.koad-os/` (except `logs/`) and `koad.json` must only be modified via official `koad` subcommands or surgical, approved manual edits.
- **Cognitive Isolation:** Ensure SQLite partitions remain strictly separated by agent name.

## Code Conventions
- Follow existing Rust idioms in `crates/`.
- Maintain comprehensive logging in `logs/` for every state transition.
- Use structured JSON for all internal messaging over the bus.

## Verification Requirements
- Every change to the Spine requires a full `koad doctor` health check.
- New features must include a reproduction script or automated test in `tests/`.
