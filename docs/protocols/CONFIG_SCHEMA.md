# KoadOS Configuration Schema (v3.2)

## Ⅰ. Registry Configuration (registry.toml)
Maps physical paths to logical **Workspace Levels**.

```toml
[projects.koados]
path = "~/.koad-os"
level = "citadel"
github_owner = "Fryymann"

[projects.skylinks]
path = "/home/ideans/data/skylinks/.agents/.sky"
level = "station"
github_owner = "Skylinks-Golf"
```

## Ⅱ. Level Definitions
- **citadel**: Platform core. Shared protocols and officer vaults.
- **station**: Project hub. Shared resources across related repos.
- **outpost**: Single repository. Task-specific execution context.
- **system**: Full machine access. Restricted to Admiral/Captain.

## Ⅲ. Identity Configuration (identities/*.toml)
Defines agent personas and their assigned hierarchy tier.
