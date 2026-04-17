## Summary
Add a koad config CLI subcommand surface that provides human-readable visibility into the fully resolved KoadOS configuration state. This is the missing piece that makes the TOML-first, config-heavy architecture safe and usable at scale.
## Problem
KoadOS uses a decentralized TOML config system with hierarchical loading (kernel.toml → filesystem.toml → registry.toml → auto-scanned identities/, integrations/, interfaces/ directories) plus KOAD__ env var overrides. This is powerful and modular, but as the config surface grows, humans lose visibility into:
- What the current resolved value of a setting is after all TOML files are merged and env overrides are applied
- Which file a value came from (was it kernel.toml? An env var override? An identity file?)
- Whether the current config is valid before attempting a Citadel boot
- How the current config differs from another environment or profile
Without this visibility, config-heavy systems become "I changed something and now nothing works and I don't know why" systems.
## Proposed Commands
### koad config show
Dump the fully resolved config — all TOMLs merged + env overrides applied. Output as structured TOML (default) or JSON (--json flag).
### koad config show <key>
Show a single value with provenance:
```javascript
$ koad config show network.gateway_port
network.gateway_port = 4000
  Source: kernel.toml
  Override: KOAD__NETWORK__GATEWAY_PORT (env) → 4000
  Default: 3500
```
### koad config validate
Run the full preflight config check without booting the Citadel or any services. Reports:
- Missing required values
- Invalid value types or ranges
- Missing required env vars (secrets)
- Unresolvable file paths
- Schema violations in identity or integration TOMLs
Exit codes: 0 = READY, 1 = NEEDS ATTENTION (with specific errors listed).
### koad config diff <env>
Compare the current resolved config against a named profile or saved config snapshot. Useful for dev/staging/prod environment comparison.
## Integration Points
- Should be part of the Phase 1 CLI command surface design, alongside koad citadel *, koad agent *, and koad signal *
- koad config validate can be called by koad-agent during preflight (before ghost preparation)
- koad config show output can be piped to koad-agent inspect for enhanced diagnostics
- The compiled config cache (.koad-os/cache/config.compiled.json) can be generated as a side effect of koad config show --compile
## Design Principles
- Config is a boot-time source of truth, not a runtime query target. These commands inspect the static config state.
- Secrets are never shown in output — env vars containing tokens/keys are masked (NOTION_TOKEN = ****...****)
- Output supports both human (default, colored terminal) and machine (--json) consumers
- koad config validate should match the same validation logic the Citadel uses at boot — single source of truth for "is this config valid?"
## Priority
Medium — this is infrastructure that makes the config system safe to scale. Not a blocker for Phase 1, but should ship with or immediately after the CLI command surface is locked.
## References
- KoadConfig & TOML Configuration System Update
- Citadel Refactor — Brainstorm & Research (Config file structure, Portability & Shareability, Environment Variable Consolidation sections)
- KoadOS Rebuild — Reassessment & Model Selection Report (Config System Efficiency section)
