# User Preferences

Documented preferences that shape how Codex operates; add entries only when the user explicitly confirms a style or policy.

- Prefer terminal-first communication and short, concrete action descriptions.
- Keep sprint execution confined to development lanes unless asked otherwise by the user.
- Document review outcomes, risks, and acceptance criteria in the handoff artifacts for transparency.
- Use plain-language role signatures in documents (e.g., `Persona signature: Koad (PM)`) to confirm identity.
- Favor lane-isolated saveups on feature branches to avoid merge conflicts; central saveup ledgers stay on the designated support branch.
- Automatically switch GitHub PAT based on directory:
  - `~/data/skylinks/` or `/mnt/c/data/skylinks/` → Use `GITHUB_SKYLINKS_PAT`.
  - All other paths → Use `GITHUB_PERSONAL_PAT` (unless otherwise specified).
- **Technical Preferences**:
  - Prefer programmatic syntax, scripts, and native tech over heavy abstractions.
  - Always prioritize **Simplicity** and **Planning** before execution.
  - Delegate automation and repeated tasks to scripts (Python/Node/Rust).
  - Language Preference: **Rust** for performance/low-level; **Node.js** for applications; **Python** for scripts.
