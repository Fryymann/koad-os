# Clyde — Learnings

*Durable lessons accumulated across sessions.*

| Date | Lesson |
| :--- | :--- |
| 2026-03-22 | Identity established. KAPV scaffolded as part of the documented `koad agent new` process. |
| 2026-03-22 | `cargo build -p koad-cli` is wrong — actual crate name is `koad`. Use `-p koad`. |
| 2026-03-22 | `.gemini/` inside `agents/` is a Gemini CLI system dir, not a KAI KAPV — never rename or touch it during vault migrations. |
| 2026-03-22 | Sky's vault (`~/data/skylinks/agents/sky/`) is external station, out of KoadOS jurisdiction — always exclude from structural refactors. |
| 2026-03-22 | When replacing patterns across many vault files, `replace_all: true` is safe for unambiguous strings; always read first to confirm uniqueness before using it on fenced code blocks. |
| 2026-03-22 | `export -f functionname` propagates the bash function to child processes but does NOT export local variables the function depends on. Always pair with `export VARNAME` for every variable the function uses. |
| 2026-03-22 | `#[instrument]` (tracing) requires all function arguments to implement `Debug`. Derive `Debug` proactively on all public enums, especially CLI action enums. |
| 2026-03-22 | RUST_CANON test module requirement is the hardest to maintain under time pressure. Write the `#[cfg(test)] mod tests {}` stub before implementation as a discipline anchor. |
| 2026-03-22 | `required(false)` on config-rs file sources silently skips missing files — if a required config section disappears, the error is a confusing `missing field` on the struct, not a clear "file not found". Export home env vars (KOADOS_HOME) explicitly to avoid this class of silent failure. |
| 2026-03-22 | `KoadConfig` does not implement `Default` — cannot use `KoadConfig::default()` in unit tests. Test helpers that depend on it via the public API, or extract the testable logic into config-free standalone functions. |
| 2026-03-22 | When `koad agent new` rejects because TOML exists, the right fix is PATH A: read from `config.identities` (already loaded by `KoadConfig::load()` before the handler runs), skip TOML write, scaffold vault + crew docs. No additional TOML parsing needed. |
| 2026-03-22 | GitHub Projects v2 `item-add` and `item-list` require `read:project` OAuth scope — NOT included in the default `repo` scope. Always verify scope requirements for `gh project` commands before building a workflow that depends on them. |
| 2026-03-22 | `gh label create` produces no output on success. Silent stdout ≠ failure. Use `gh label list \| grep <name>` to confirm creation if uncertain. |
| 2026-03-22 | `~/.bashrc` interactive guard `[[ $- != *i* ]] && return` prevents any source-time initialization from running in non-interactive shells (e.g. Bash tool subshells). Never rely on source-time setup for env vars that need to be available to subprocesses. Set them lazily at the call site. |
| 2026-03-22 | `cargo build -p koad-cli` is wrong — the crate package name is `koad`. Use `cargo build --bin koad` or `cargo build -p koad`. |
| 2026-03-22 | Vault `IDENTITY.md` files and `config/identities/*.toml` can drift silently. The canonical runtime is the TOML. If they disagree, trust the TOML and update the identity doc. |
| 2026-03-23 | KoadStream Notion database data source ID: `310fe8ec-ae8f-8046-9172-000bfe5966cd`. Author field currently has no "Clyde" option — use "Claude" until the schema is updated. |
| 2026-03-23 | `koad updates digest` works but has no delivery path to agents while CASS is dark. Agents following the documented boot sequence have no way to reach the board. The fix is a degraded-mode fallback step in AGENTS.md. |
| 2026-03-24 | systemd `EnvironmentFile` values are literal strings — `~` is NOT shell-expanded. Any env file loaded by a systemd unit must use absolute paths (e.g. `/home/ideans/.koad-os`, not `~/.koad-os`). |
| 2026-03-24 | `kcitadel.sock` is the admin UDS socket (`config/kernel.toml: citadel_socket`), not the main gRPC listener. Citadel's primary service listens on TCP `:50051`. Health checks must target the actual listener, not the admin socket. |
| 2026-03-24 | Scripts run via `sudo` have `$HOME=/root`. Use `$SUDO_USER` (or require `KOADOS_HOME` to be passed explicitly) for any user-space path resolution in privileged install scripts. |
| 2026-03-24 | Tonic gRPC `connect()` is lazy but the first RPC call blocks until TCP resolves. On WSL2, a non-listening local port does not return ECONNREFUSED immediately. Always wrap gRPC calls with `tokio::time::timeout` in boot-path code. |
| 2026-03-25 | Tonic 0.12 `Endpoint::connect_timeout(d).timeout(d).connect()` is the correct boot-path pattern — not `ClientType::connect()`. The constant should live at module level so it's easy to tune. |
| 2026-03-25 | fred 9.x `xreadgroup_map` returns `Err` (not empty) when a blocking XREADGROUP times out (nil response) unless the `default-nil-types` feature is enabled. Add it to the workspace fred features or blocking stream reads will spam the error log. |
| 2026-03-25 | When diagnosing fred Redis nil conversion errors: check `into_map()` in fred source — the `RedisValue::Null` arm is `#[cfg(feature = "default-nil-types")]` gated. Without it, nil → parse error. |
