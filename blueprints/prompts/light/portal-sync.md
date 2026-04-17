# INSTRUCTION: PORTAL SYNC & MAPPING
# ROLE: Scout / Maintenance Agent
# OBJECTIVE: Synchronize ~/.koad-os/portals/ with physical workspace stations and outposts.

## 🧭 DISCOVERY PROTOCOL
1. **Physical Scan:**
   - Scan `~/data/` and `~/data/SLE/apps/` for markers.
   - A **Station** is defined by the presence of a `.koados-station/` directory.
   - An **Outpost** is defined by the presence of a `.koados-outpost/` directory.
   - Use `koad map` to cross-reference known situational context.

2. **Logical Mapping:**
   - Check the current contents of `~/.koad-os/portals/`.
   - Identify missing links or broken symlinks.

## 🛠️ EXECUTION RULES
- **Naming Convention:**
  - Stations: `station-<short-name>` (e.g., `station-sle`).
  - Outposts: `outpost-<short-name>` (e.g., `outpost-sgc-reg`).
- **Surgical Updates:**
  - Create symlinks ONLY for directories containing the relevant `.koados-*` marker.
  - DO NOT overwrite existing valid symlinks.
  - Remove dead symlinks pointing to non-existent paths.

## 📝 OUTPUT
- Provide a brief summary of added, updated, or removed portals.
- If no changes are needed, state "Portals are synchronized."

---
*KoadOS Maintenance Protocol | Tyr | 2026-03-28*
