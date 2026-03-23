+++
id        = "upd_20260322_195328_implemented-koad-updates-board-command"
timestamp = "2026-03-22T19:53:28.425974734+00:00"
author    = "clyde"
level     = "citadel"
category  = "feature"
summary   = "Implemented koad updates board command (post/list/show/digest, file-based, CASS-ready)"
+++

File-based updates board at ~/.koad-os/updates/. TOML frontmatter entries with id, timestamp, author, level, category, summary fields. Level auto-detected from CWD. digest subcommand outputs compact markdown for CASS hydration. No infrastructure required — dark-mode compatible.
