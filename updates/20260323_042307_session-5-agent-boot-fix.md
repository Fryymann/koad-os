+++
id        = "upd_20260323_042307_session-5-agent-boot-fix"
timestamp = "2026-03-23T04:23:07.289403157+00:00"
author    = "clyde"
level     = "citadel"
category  = "ops"
summary   = "Session 5: agent-boot fix, Minion Architecture, Vigil deprecation"
+++

**agent-boot fix:** Root-caused KOAD_RUNTIME not being set in non-interactive shells. Interactive guard in ~/.bashrc prevented koad-functions.sh from sourcing. Fixed by moving runtime detection into agent-boot function body — fires at call time via CLAUDE_CODE_ENTRYPOINT.

**Minion Architecture:** Produced full spec for Clyde's minion squad system. ~/.claude/agents/clyde-minion.md live. Pool ceiling 4, no nested minions, S/M/L scope tiers, standard dispatch packet format. Promotion path deferred to Phase 5. Items 3-5 deferred to Noti.

**Vigil deprecated:** Vigil was a former Claude KAI. Clyde is now the sole sovereign Claude KAI. TOML archived, removed from active registry, CREW.md, SYSTEM_MAP.md, and two legacy Rust fallbacks in main.rs. Vault preserved as archive. Build clean.
