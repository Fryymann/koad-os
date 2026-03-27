# KoadOS Configuration Schema (v3.2)

## Ⅰ. Registry Configuration (registry.toml)
Maps physical paths to logical **Workspace Levels** and **Stations**.

```toml
[projects.koados]
path = "~/.koad-os"
level = "citadel"
github_owner = "KOADOS_MAIN_GITHUB_USER"
credential_key = "GITHUB_PAT"

[projects.skylinks]
path = "/home/ideans/data/skylinks/agents/sky"
level = "station"
station = "SLE"
github_owner = "KOADOS_STATION_SLE_GITHUB_USER"
credential_key = "GITHUB_PAT"
```

## Ⅱ. Level Definitions
- **citadel**: Platform core. Shared protocols and officer vaults.
- **station**: Project hub. Shared resources across related repos.
- **outpost**: Single repository. Task-specific execution context.
- **system**: Full machine access. Restricted to Admiral/Captain.

## Ⅲ. Hierarchical Environment Variables (KOADOS_ Namespace)
KoadOS resolves secrets and configuration values hierarchically:

1. **Outpost Overrides:** `KOADOS_OUTPOST_<NAME>_<KEY>` (e.g. `KOADOS_OUTPOST_DND_GITHUB_PAT`)
2. **Station Overrides:** `KOADOS_STATION_<NAME>_<KEY>` (e.g. `KOADOS_STATION_SLE_GITHUB_PAT`)
3. **Citadel Default:** `KOADOS_MAIN_<KEY>` (e.g. `KOADOS_MAIN_GITHUB_PAT`)

### Indirect Resolution
Environment variables in the `KOADOS_` namespace support **Indirect Resolution**. If the value of a `KOADOS_` variable matches the name of another exported environment variable (e.g. `GITHUB_PERSONAL_PAT`), KoadOS will dereference it to the actual secret value.

## Ⅳ. Identity Configuration (identities/*.toml)
Defines agent personas and their assigned hierarchy tier.
```toml
[identities.tyr]
name = "Tyr"
role = "Captain"
access_keys = ["GITHUB_PAT"] # Generic Token ID
```
