# Decision Log

## YYYY-MM-DD - Decision title
- Decision:
- Why:
- Impact:
- Revisit trigger:

## 2026-02-22 - Establish global koadOS memory kit
- Decision:
  - Centralize the shared memory, learning, and saveup protocols inside `~/.koad-os` so multiple projects can reuse them.
- Why:
  - Prevent every repository from maintaining duplicate agent infrastructure and to keep durability across workspaces.
- Impact:
  - Agents must run the global startup and saveup protocols before touching workspace-specific artifacts.
- Revisit trigger:
  - When the kit structure changes or a new shared persistence store replaces local files.
