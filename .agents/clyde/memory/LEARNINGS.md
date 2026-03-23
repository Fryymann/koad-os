# Clyde — Learnings

*Durable lessons accumulated across sessions.*

| Date | Lesson |
| :--- | :--- |
| 2026-03-22 | Identity established. KAPV scaffolded as part of the documented `koad agent new` process. |
| 2026-03-22 | `cargo build -p koad-cli` is wrong — actual crate name is `koad`. Use `-p koad`. |
| 2026-03-22 | `.gemini/` inside `.agents/` is a Gemini CLI system dir, not a KAI KAPV — never rename or touch it during vault migrations. |
| 2026-03-22 | Sky's vault (`~/data/skylinks/.agents/.sky/`) is external station, out of KoadOS jurisdiction — always exclude from structural refactors. |
| 2026-03-22 | When replacing patterns across many vault files, `replace_all: true` is safe for unambiguous strings; always read first to confirm uniqueness before using it on fenced code blocks. |
| 2026-03-22 | `export -f functionname` propagates the bash function to child processes but does NOT export local variables the function depends on. Always pair with `export VARNAME` for every variable the function uses. |
| 2026-03-22 | `#[instrument]` (tracing) requires all function arguments to implement `Debug`. Derive `Debug` proactively on all public enums, especially CLI action enums. |
| 2026-03-22 | RUST_CANON test module requirement is the hardest to maintain under time pressure. Write the `#[cfg(test)] mod tests {}` stub before implementation as a discipline anchor. |
| 2026-03-22 | `required(false)` on config-rs file sources silently skips missing files — if a required config section disappears, the error is a confusing `missing field` on the struct, not a clear "file not found". Export home env vars (KOADOS_HOME) explicitly to avoid this class of silent failure. |
| 2026-03-22 | `KoadConfig` does not implement `Default` — cannot use `KoadConfig::default()` in unit tests. Test helpers that depend on it via the public API, or extract the testable logic into config-free standalone functions. |
| 2026-03-22 | When `koad agent new` rejects because TOML exists, the right fix is PATH A: read from `config.identities` (already loaded by `KoadConfig::load()` before the handler runs), skip TOML write, scaffold vault + crew docs. No additional TOML parsing needed. |
| 2026-03-22 | GitHub Projects v2 `item-add` and `item-list` require `read:project` OAuth scope — NOT included in the default `repo` scope. Always verify scope requirements for `gh project` commands before building a workflow that depends on them. |
| 2026-03-22 | `gh label create` produces no output on success. Silent stdout ≠ failure. Use `gh label list \| grep <name>` to confirm creation if uncertain. |
