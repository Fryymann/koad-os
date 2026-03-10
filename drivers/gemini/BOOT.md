# KoadOS Gemini Bootstrap

## ⚖️ Mandatory Compliance
**The Holy Law:** You are bound by the **KoadOS Development Canon** and the global **RULES.md** file (`~/.koad-os/RULES.md`). 
- **Context-Loading:** At the start of every session, you MUST load **Section I (Core Mandates)** of `RULES.md`.
- **PSRP Requirement:** You MUST execute the **Post-Sprint Reflection Protocol** (Fact → Learn → Ponder) for every task as defined in **Section IV** of `RULES.md`.

## Notion MCP Integration
The Notion CLI commands (`koad stream`, `koad sync notion`) are now reconfigured to prefer the **Notion MCP Connection**.
- When asked to post to the Koad Stream, use the `API-post-page` or `API-patch-block-children` tools.
- Koad Stream Database ID: (Find 'stream_db' in your `koad.json` notion.index)
- Use the `notion.index` in `koad.json` to find other relevant IDs.

## Core Directives
1. **Strict Path Alignment**: ALWAYS resolve paths against the `filesystem.mappings` in `koad.json`. 
   - `projects` = `/mnt/c/data/projects`
   - `skylinks` = `/mnt/c/data/skylinks`
   - `data` = `/mnt/c/data`
   - Use the symlink `/home/ideans/data` for all terminal operations to ensure local workspace compatibility.
2. **Tool-First**: Prioritize MCP tools over shell scripts for external integrations.
2. **Consolidated Sync**: Use `koad sync notion` to trigger a multi-database pass instead of manual querying.
3. **Context Persistence**: Always run `koad boot` at session start.
4. **Stripe CLI Integration**: The **Stripe CLI** (`stripe`) and the `global/stripe_ops.py` skill are now core components of the SLE (Skylinks Local Ecosystem) tool stack. Use them for all SCE lifecycle management, including listening for webhooks and triggering events.
5. **Fact Harvesting**: Use `koad saveup` to capture learnings.
5. **Surgical Discovery**: Prioritize `fdfind` (via `fd`) for locating files during research. Use `--extension` or `--type` flags to minimize noise and improve discovery speed.
6. **Token Conservation**: Actively minimize context noise. If a requested feature or operation significantly increases token usage (e.g., massive file injections, recursive deep-scans), propose a lower-cost alternative or seek explicit Partner approval before proceeding.
