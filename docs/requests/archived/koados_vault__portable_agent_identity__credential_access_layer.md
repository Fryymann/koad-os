---
## Problem Statement
The current KoadOS identity system assumes a relatively fixed filesystem layout. Agent configs, skill instances, and credentials live under ~/.koad-os/config/identities/<agent>/, and boot context hydration reads from known absolute paths.
This creates hard constraints:
1. Deployment location is coupled to identity. An agent deployed into a project station (~/projects/skylinks-golf/) or a temporary sandbox can't access its vault unless it knows the absolute path back to ~/.koad-os/.
1. Remote deployment is blocked. An agent running in a container, on a remote server, or in a CI pipeline has no filesystem access to ~/.koad-os/ at all.
1. Micro agents inherit the problem. A sovereign that spawns a micro agent in a sandboxed workspace can't easily grant vault access without exposing the full filesystem tree.
1. No access control granularity. Filesystem permissions are binary (readable or not). There's no way to grant an agent access to its own skill instances while blocking access to another agent's identity directory.
1. No audit trail. Filesystem reads are invisible. There's no way to know what an agent accessed, when, or from where.
---
## Proposed Solution: koad vault CLI
The Vault is a command-based access layer that resolves agent identity, skills, config, and credentials regardless of the agent's working directory or deployment environment.
### Core Principle
The agent never needs to know where its vault lives on disk. It needs one bootstrap value — KOAD_VAULT_URI — and the vault command handles resolution.
### KOAD_VAULT_URI — The Single Bootstrap Requirement
When an agent boots anywhere in the filesystem, it reads KOAD_VAULT_URI to know where its vault lives:
As long as KOAD_VAULT_URI is set, the agent can boot and access everything it needs regardless of working directory.
### Proposed CLI Surface
```bash
# Identity
koad vault whoami                      # Return current agent identity and vault root
koad vault env                         # Export vault-derived environment variables

# Read
koad vault get <path>                  # Read a file/value from the agent's vault
koad vault ls [path]                   # List vault contents (skills, config, secrets)

# Skills (integrates with Blueprint/Instance model)
koad vault skill <skill-name>          # Load a skill instance from the vault
koad vault skill list                  # List all skill instances in the vault
koad vault skill sync <name> --from blueprint  # Pull blueprint updates

# Secrets
koad vault secret get <key>            # Read a scoped secret (API key, token)
koad vault secret set <key> <value>    # Write a secret (operator only)
koad vault secret ls                   # List available secret keys (not values)

# Sync & State
koad vault sync                        # Pull latest vault state from Citadel
koad vault pack                        # Create a ghost config bundle for offline use
koad vault status                      # Show vault connection status and cache freshness

# Security
koad vault seal                        # Lock sensitive vault sections
koad vault unseal                      # Unlock (requires operator auth)
```
---
## Architecture
### Vault Contents
The vault is a logical container for everything an agent needs to be itself:
```plain text
vault/<agent>/
├── identity/                # Who am I?
│   ├── agent.toml           # Name, role, tier, rank
│   ├── personality.md       # Voice, tone, behavioral traits
│   └── permissions.toml     # What am I allowed to do?
├── skills/                  # What can I do? (Instance copies)
│   ├── code-review/
│   │   ├── SKILL.md
│   │   └── _eval/
│   └── notion-sync/
│       ├── SKILL.md
│       └── _eval/
├── config/                  # How am I configured?
│   ├── boot.toml            # Boot sequence config
│   ├── memory.toml          # CASS connection settings
│   └── integrations/        # MCP server configs, API endpoints
├── secrets/                 # Credentials (encrypted at rest)
│   ├── github_token
│   ├── notion_api_key
│   └── qdrant_api_key
└── cache/                   # Local cache for vault operations
    ├── last_sync.json       # Timestamp + hash of last Citadel sync
    └── ghost_bundle.tar.gz  # Packed offline snapshot
```
### Resolution Chain
When koad vault get <path> is called, the resolver follows this chain:
1. Local cache — If the requested item is cached and fresh (within TTL), return immediately.
1. Primary source — Read from the URI specified by KOAD_VAULT_URI.
1. Ghost fallback — If primary source is unreachable, fall back to the ghost bundle.
1. Fail explicit — If all sources fail, return a clear error with the failed resolution chain. Never silently return stale or empty data.
### Caching Strategy
---
## Access Control
The vault command enforces scoped access — agents can only access what they're authorized for:
- Own vault: full read. An agent can read anything in its own vault.
- Other agent vaults: blocked. No agent can read another agent's vault. This enforces the CASS memory isolation model at the identity layer.
- Secrets: scoped. Secrets are tagged with access scopes. An agent can only read secrets that match its scope (e.g., scope: github grants access to github_token but not notion_api_key).
- Micro agents: delegated. When a sovereign spawns a micro agent, it can grant a vault subset — a read-only view of specific skills and config, without exposing the full vault. Implemented via a scoped token: koad vault delegate --to micro-123 --grant skills/code-review,config/boot.toml.
- Operator: full access. Ian can read/write any vault, any section. The koad vault secret set command is operator-only.
---
## Integration Points
---
## Risks & Mitigations
### Latency
Risk: Filesystem reads are ~microseconds. Citadel API calls are ~milliseconds. If vault access is on the hot path, remote agents pay a performance tax.
Mitigation: Aggressive caching with session-long TTLs for identity and skills. The ghost bundle acts as a complete offline cache. Only secrets and config require live resolution.
### Single Point of Failure
Risk: If vault access goes through the Citadel and the Citadel is down, all remote agents lose identity access.
Mitigation: Ghost mode fallback. koad vault pack at boot creates a local snapshot. Agents degrade to ghost mode, not crash. Same pattern as the Citadel disconnect architecture.
### Secret Exposure
Risk: Vault commands could leak secrets to logs, environment variables, or process lists.
Mitigation: Secrets are never written to disk cache. koad vault secret get writes to stdout only (for piping), never to files. koad vault env exports non-secret config only — secrets require explicit koad vault secret get calls. Process-level masking for ps output.
### Cache Invalidation
Risk: Stale cache could cause an agent to operate with outdated identity, revoked credentials, or old skill versions.
Mitigation: koad vault status shows cache freshness. koad vault sync --force bypasses cache. Critical sections (secrets) are never cached. A vault:invalidate signal from the Citadel can push cache busts to connected agents.
---
## Relationship to koad config
koad config and koad vault are complementary surfaces for the same underlying system:
---
## Phasing Recommendation
- Phase 2 (Citadel Core): Implement KOAD_VAULT_URI resolution and file:// scheme. Local vault access via command replaces hardcoded paths in boot pipeline.
- Phase 3 (CASS Online): Add koad vault skill integration with Blueprint/Instance model. Skill loading goes through vault.
- Phase 4 (koad-agent): Add http:// scheme for Citadel-served vault. Remote deployment becomes possible.
- Phase 5 (Integration): Add koad vault delegate for micro agent spawning. Scoped access tokens.
- Phase 6+ (Memory Advanced): Add ghost:// scheme. Full offline bundle support. vault:invalidate push signals.
---
## Open Questions for Tyr
1. Vault storage backend — Should the vault be a thin wrapper over the existing filesystem layout, or should it be a proper key-value store (e.g., a SQLite db per agent) that happens to be populated from TOML files?
1. Secret encryption — What encryption scheme for secrets at rest? Age? SOPS? KoadOS-native?
1. Vault delegation token format — JWT? Signed TOML? Something simpler?
1. Boot ordering — The vault needs KOAD_VAULT_URI before anything else loads. How does this integrate with the existing boot sequence? Is it the very first thing koad-agent reads, before even parsing kernel.toml?
1. Vault versioning — Should the vault itself be versioned (for rollback)? If an operator accidentally overwrites a secret or corrupts identity config, can we koad vault rollback --to <timestamp>?
---
## Rules of Engagement
1. No implementation. This is research and proposal. No feature code.
1. Noti's role: Architecture proposal, integration mapping, risk analysis.
1. Tyr's role: Evaluate feasibility, propose storage backend, determine boot integration.
1. Dood approves the path forward after Tyr's evaluation.
