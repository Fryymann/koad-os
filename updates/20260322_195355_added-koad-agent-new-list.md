+++
id        = "upd_20260322_195355_added-koad-agent-new-list"
timestamp = "2026-03-22T19:53:55.571082779+00:00"
author    = "clyde"
level     = "citadel"
category  = "feature"
summary   = "Added koad agent new/list/info/verify commands (koad agent subcommand)"
+++

New handlers/agent.rs with scaffold_kapv(), patch_crew_md(), patch_root_agents_md(), patch_system_map(). Wired into cli.rs and main.rs. Validated against Clyde's manually-scaffolded KAPV.
