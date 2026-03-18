## Summary

Implement two terminal UI display components for KoadOS: a **MOTD (Message of the Day) boot banner** displayed by each agent after initialization, and a **Citadel Status Board** showing real-time system/subsystem health across the full Citadel.

---

## Feature 1 — MOTD Boot Banner

Each agent should display a compact MOTD panel immediately after boot/init sequence completes.

**Requirements:**

- Rendered via `ratatui` and/or `crossterm`
- Single print-out (non-interactive, no live refresh required at this stage)
- Should include a snapshot of Citadel Status at boot time (see Feature 2)
- Shows agent identity, role, and any relevant boot-time notices

---

## Feature 2 — Citadel Status Board

A cleanly laid-out, organized TUI panel showing the health of **all** Citadel systems and subsystems — including those not yet implemented.

**Design philosophy:**

- Built for a **completed Citadel** as currently envisioned — always shows the full array of systems
- Systems not yet implemented display as `OFFLINE` or `NOT IMPLEMENTED` — this is intentional; the board is a living reference for what's working and what still needs to be done
- Reverse-engineering the full system list from current canon is acceptable and expected

**Display requirements:**

- Per-system status indicators: 🔴 / 🟡 / 🟢 (or red/green minimum)
- **Uptime** per system in `hh:mm:ss` format
- Quick-look dashboard items (e.g. last heartbeat, load, message queue depth — TBD per system)
- Layout: organized by subsystem grouping, clean and scannable

**Placeholder / Hook Safety:**

- Any hooks or data contracts left as placeholders **must not lock in design decisions** for systems not yet built
- Prefer loose coupling: status data should be pulled via a well-defined, extensible interface (trait/enum) so future systems can register themselves without modifying the board renderer
- Document clearly which systems are stubbed vs. live

**Rendering:**

- `ratatui` preferred; `crossterm` for lower-level control if needed
- Single-pass print-out acceptable for v1; live refresh / polling can be a follow-on

---

## Configuration (KoadOS TOML Standard)

This feature must integrate with the KoadOS config system. All display behavior, system registry, and status polling should be driven by TOML config — no hardcoded values.

**Config file:** `citadel_status.toml` (or within the agent's own `[motd]` / `[status_board]` table if co-located)

**Example shape (non-prescriptive — Tyr owns final schema):**

```toml
[motd]
enabled = true
show_citadel_snapshot = true
show_agent_role = true
show_boot_notices = true

[status_board]
refresh_interval_secs = 0  # 0 = single-pass print only
color_mode = "red_yellow_green"  # or "red_green"

[[status_board.systems]]
name = "Koad Stream"
subsystem = "Messaging"
enabled = true
stub = false

[[status_board.systems]]
name = "CASS"
subsystem = "Storage"
enabled = true
stub = true  # not yet implemented
```

**Requirements:**

- Config must be loaded via the existing KoadOS config loader — no one-off file reads
- System registry (the list of all Citadel systems/subsystems) lives in TOML, not in code
- The `stub = true` flag marks unimplemented systems — the board renders them as `OFFLINE / NOT IMPLEMENTED` automatically
- Adding a new system in the future requires only a new `[[status_board.systems]]` entry — zero code changes to the board renderer
- Follow KoadOS TOML conventions (naming, file placement, loading order) as established in canon

---

## Acceptance Criteria

- [ ]  MOTD displays on agent boot with Citadel status snapshot
- [ ]  Citadel Status Board lists all known systems/subsystems (implemented or not)
- [ ]  Each system shows: status (up/down), uptime `hh:mm:ss`, and at least one quick-look metric
- [ ]  Placeholder hooks use an extensible interface — no hardcoded assumptions about unbuilt systems
- [ ]  Board renders cleanly in a standard terminal via `ratatui`/`crossterm`
- [ ]  Offline/unimplemented systems are clearly distinguished from live ones

---

## Assigned To

Tyr

## Priority

Medium — foundational visibility tooling; unblocks operational situational awareness across the Citadel