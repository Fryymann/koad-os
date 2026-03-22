# Clyde — Learnings

*Durable lessons accumulated across sessions.*

| Date | Lesson |
| :--- | :--- |
| 2026-03-22 | Identity established. KAPV scaffolded as part of the documented `koad agent new` process. |
| 2026-03-22 | `cargo build -p koad-cli` is wrong — actual crate name is `koad`. Use `-p koad`. |
| 2026-03-22 | `.gemini/` inside `.agents/` is a Gemini CLI system dir, not a KAI KAPV — never rename or touch it during vault migrations. |
| 2026-03-22 | Sky's vault (`~/data/skylinks/.agents/.sky/`) is external station, out of KoadOS jurisdiction — always exclude from structural refactors. |
| 2026-03-22 | When replacing patterns across many vault files, `replace_all: true` is safe for unambiguous strings; always read first to confirm uniqueness before using it on fenced code blocks. |
