# Dev Log — 2026-09-26: Dependency and Security Refresh

Author: Clyde (Claude Code, Opus 5.5). Reviewed and merged by Ian.
PRs: [#223](https://github.com/Fryymann/koad-os/pull/223), [#224](https://github.com/Fryymann/koad-os/pull/224), both into `nightly`.

## Why

The first session in about six weeks started with an audit of CASS, Citadel and their dependencies
against upstream. Three things needed action right away:

1. The MCP server told clients it supported whatever protocol version they asked for, including the
   new 2026-07-28 revision it does not implement.
2. The locked dependencies had 60+ RustSec/GHSA advisory hits. About 30 were in `wasmtime` 22.0.1,
   including sandbox escapes.
3. The container plugin path had a shell injection bug and called a function that doesn't exist.

## What changed

### MCP protocol version negotiation (#223, `fe0753d`)

`crates/koad-mcp/src/lib.rs` `initialize` used to echo the client's `protocolVersion`. A client that
asked for `2026-07-28` was told the server spoke it. That revision drops the `initialize` handshake
and requires `server/discover`, which we don't have, so a client that believed the answer would break.

- New `SUPPORTED_PROTOCOL_VERSIONS` (`2024-11-05`, `2025-03-26`, `2025-06-18`, `2025-11-25`) and
  `LATEST_PROTOCOL_VERSION` (`2025-11-25`).
- `negotiate_protocol_version()` echoes the client's version only when it is supported. Otherwise it
  offers the latest supported version, per the MCP lifecycle rules.
- `initialize` now advertises the `tools` capability. Clients that honour the capability map skip
  `tools/list` when it is absent, which leaves a connected server with zero tools.
- Verified live: asking for `2026-07-28` gets `2025-11-25`; asking for `2025-06-18` gets `2025-06-18`.

### Semver-compatible lockfile refresh (#223, `068a9ac`)

`cargo update` only, no manifest changes. Clears the advisories in openssl, rustls, rustls-webpki,
h2, quinn-proto, anyhow, crossbeam-epoch, rand and rustls-pemfile.

### wasmtime 22 → 49 (#224, `d5793eb`)

Only `koad-plugins` uses wasmtime. The API migration in `crates/koad-plugins/src/lib.rs`:

| wasmtime 22 | wasmtime 49 |
|---|---|
| `bindgen!({ async: true })` | `imports: { default: async }, exports: { default: async }` |
| hand-written `Pin<Box<dyn Future>>` host import | plain `async fn log(&mut self, msg: String)` |
| `add_to_linker_imports_get_host(linker, get_host)` | `add_to_linker::<_, HasSelf<_>>(linker, \|s\| s)` |
| `instantiate_async` → `(bindings, Instance)` | `instantiate_async` → bindings |
| `wasmtime::Error` converts to `anyhow` implicitly | explicit `.map_err(anyhow::Error::from)` |
| `Config::async_support(true)` | deprecated no-op, removed |

wasmtime 49 needs rustc ≥ 1.96. `rust-toolchain.toml` now pins **1.98.1**.

### WASM tests actually run (#224, `d07231c`)

The three tests that execute a plugin looked for a component under the gitignored
`examples/hello-plugin/target/` path and returned early when it was missing. They had been passing
without running any guest code. They now load the committed `wit/hello-plugin.component.wasm` and
assert on the guest's output (`"Hello from WASM!"` plus the echoed topic). With the fixture hidden,
3 tests fail, which is the point.

### Container invoke quoting and call syntax (#224, `18c9b75`)

`PluginRegistry::invoke` with a `container_image` builds a command for `sh -c`. It used to wrap the
topic and payload in single quotes and escape `'` as `\'`. A backslash does not escape anything inside
single quotes, so a payload like `a'; cmd #` closed the quote and ran `cmd` inside the container.
This was reproduced against `sh`.

- `shell_quote()` uses the POSIX close-escape-reopen form (`'` → `'\''`).
- The call used core-module CLI syntax for a nonexistent `on_signal` export. It is now a WAVE call to
  the world's `invoke` export: `--invoke 'invoke("topic", "payload")'`. `wave_string()` handles WAVE
  string escaping.
- Tests run hostile input through a real shell and check the exact argv wasmtime would receive.

## Verification

- Workspace: 193 passed, 4 ignored, 0 failed (was 183 in July, 190 after #223).
- OSV scan of `Cargo.lock`: no wasmtime, fxhash, openssl or rustls advisories remain.
- Deployed with `install.sh --update`, then restarted `clyde-mcp.service` (user unit) and
  `koad-cass` / `koad-citadel` (system units, restarted by Ian). All run the new binaries; health PASS.

## Still open

In rough priority order:

1. **Network exposure:** `koad-os-mcp` (port 9744) and Qdrant (6333/6334) listen on all interfaces,
   not just `127.0.0.1`.
2. **Redis 7.0.15** has been end-of-life since July 2024. Upgrade to 7.4 or 8.x.
3. **gRPC and web stack:** `tonic`/`prost` 0.12→0.14 (prost moves to `tonic-prost`), `axum` 0.7→0.8
   (route syntax `/:id` → `/{id}`), `tower` 0.4→0.5, `tower-http`, `reqwest`, `tungstenite`. These
   should move on one branch together.
4. **`ratatui` 0.26** pins `lru` 0.12.5, which has 3 advisories. `paste` is unmaintained.
5. **Container plugin path:** never exercised end to end (no Docker in WSL here). A plugin that
   imports the host `log` function probably cannot link under the plain wasmtime CLI. Not verified.
6. **MCP 2026-07-28:** stateless requests, `server/discover`, `resultType`, cacheable list results.
   Consider moving to the official Rust SDK (`rmcp`) instead of the hand-rolled server.

## Operational notes

- `install.sh --update` swaps binaries but cannot restart the system units without a sudo password.
  It prints a warning and carries on. Always follow it with
  `sudo systemctl restart koad-cass.service koad-citadel.service` and confirm that
  `readlink /proc/<pid>/exe` no longer shows `(deleted)`.
- `citadel-memory` is served by `koad-os-mcp` under the **user** unit `clyde-mcp.service`, which
  restarts without sudo: `systemctl --user restart clyde-mcp.service`.
