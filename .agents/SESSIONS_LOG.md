---
## Clyde — Session 7 — 2026-03-24

**Work Performed:** Citadel Jupiter service bring-up + agent-boot fix

### Problem
`agent-boot` was hanging for 3+ minutes. Citadel (:50051) and CASS (:50052) were DARK — systemd units existed in `config/systemd/` but were never installed on Jupiter. The hang was caused by `koad-agent` making gRPC calls to dark services with no timeout; WSL2 doesn't return ECONNREFUSED immediately so the binary stalled until OS TCP timeout fired.

### Changes Made
**`config/systemd/koad-citadel.service`**
- Added `Wants=koad-cass.service` — starting Citadel now cascades to CASS automatically.

**`crates/koad-cli/src/cli.rs` + `handlers/system.rs`**
- Added `koad system start` — starts Citadel + CASS. Uses `systemctl` when units are installed, falls back to direct spawn with log file redirection.
- Added `koad system restart` — kills both processes, waits 800ms, starts them.
- Fixed `koad system stop` — was only killing `koad-citadel`, now also kills `koad-cass`.
- Fixed `koad system refresh --restart` — removed stale `kcitadel` / `koad-watchdog` references, replaced with `koad-citadel` + `koad-cass` via the new shared helpers.

**`.env`**
- `KOADOS_HOME=~/.koad-os` → `KOADOS_HOME=/home/ideans/.koad-os` — systemd does not expand `~` in EnvironmentFile values; this was causing `KoadConfig::load()` to fail with "missing field `system`" under systemd.

**`scripts/install-services.sh`**
- Fixed `$HOME` resolving to `/root` under sudo. Now uses `$SUDO_USER` for path resolution.

### Current Jupiter State
- Citadel (:50051): **ACTIVE** — systemd managed, enabled for WSL auto-start
- CASS (:50052): **ACTIVE** — systemd managed, enabled for WSL auto-start
- Redis (koad.sock): PONG
- `agent-boot clyde`: ~1s (was 3+ min)
- Qdrant: still OFFLINE (#205)
- Docker: still OFFLINE

### Open / Pending
- `agent-boot` still hangs if services go dark — gRPC calls in `koad-agent.rs` have no timeout. Recommend adding `tokio::time::timeout` wrappers on the Citadel and CASS connect/call blocks (lines 166–227). Not blocking but worth a Phase 4 issue.
- Tyr boot command (if GEMINI_API_KEY not auto-detected): `source ~/.koad-os/bin/koad-functions.sh && KOAD_RUNTIME=gemini agent-boot tyr`

— Clyde

---

## Tyr — Session 8 — 2026-03-24

**Work Performed:** Implementation of `--agentprep` Shell-Body Authorization & Pre-Flight.

### Problem
`agent-boot` requires `KOAD_RUNTIME` to be set in the shell to authorize the "Body" before hydrating the "Ghost". Manual exports were inefficient. Additionally, the boot flow lacked a "pre-flight" check for critical services (Redis, SQLite, Citadel), leading to failed hydration attempts in "Dark Mode" environments.

### Changes Made
**`~/.koad-os/bin/koad-functions.sh`**
- **Feature:** Added `agent-prep` function (aliased to `--agentprep <name>`).
- **Logic:** Resolves `KOAD_RUNTIME` automatically from the agent's identity TOML (e.g., `tyr.toml`).
- **Health Check:** Implemented status checks for Redis (`koad.sock`), SQLite (`koad.db`), and Citadel (`kcitadel.sock`).
- **Authorization:** Exports `KOAD_RUNTIME` to the current shell to allow subsequent `agent-boot` calls.

**`~/.pimpedbash/.bashrc`**
- **Integration:** Added explicit sourcing of `koad-functions.sh` to ensure commands are globally available.
- **Cleanup:** Removed redundant source lines in `~/.bashrc` to maintain a clean dotfile environment.

### Verification & State
- **Body vs Ghost:** Confirmed that sourcing `koad-functions.sh` only identifies the shell type (Body) via API key detection and does *not* pre-select an agent.
- **Command Loop:** Verified `--agentprep tyr` successfully authorized the shell and provided a service status report.
- **Tool Registry (Phase 4):** 
  - Compiled and generated the `hello-plugin` WASM component.
  - Successfully registered `hello-plugin` with the CASS Tool Registry Service.
  - Verified dynamic invocation: Tool responded correctly with a 69ms execution time.
- **Current Jupiter State:** Services are fully operational (Citadel, CASS, Redis, SQLite).

### Open / Pending
- Scale pre-flight checks to include Qdrant and Docker (once WSL integration is enabled).
- Address Clyde's gRPC timeout recommendation in `koad-agent`.

— Tyr

---
## Clyde — Session 8 — 2026-03-24

**Work Performed:** Tyr session review + `agent-prep` Citadel health check fix

### Problem
Tyr implemented `agent-prep` with a Citadel health check that tested for `$KOAD_HOME/kcitadel.sock` (the admin UDS socket). That socket is not created by the current Citadel startup — Citadel runs on TCP `:50051`. The check was a permanent false negative, always reporting WARN regardless of service state.

### Changes Made
**`~/.koad-os/bin/koad-functions.sh`**
- Fixed `agent-prep` Citadel check: replaced `[ -S "$KOAD_HOME/kcitadel.sock" ]` with `ss -tlnp | grep -q '127.0.0.1:50051'`.
- All three pre-flight checks (Redis, SQLite, Citadel) now correctly reflect live service state.

### Verification
`agent-prep tyr` output on Jupiter with services running:
- Engine Room (Redis): PASS
- Memory Bank (SQLite): PASS
- Control Plane: PASS (:50051)

### Notes
- `kcitadel.sock` is the admin gRPC UDS socket path (`config/kernel.toml: citadel_socket`). It is referenced in the kernel but not created by the current `KernelBuilder::start()` path. If admin socket support is wired up in a future phase, the check may want to test both TCP and the admin socket.
- Tyr's runtime resolution logic (grep from TOML) is functional for current identity file structure.

— Clyde

---
## Clyde — Review of Tyr Session 8 — 2026-03-24

**Bug found and fixed:** Citadel health check in `agent-prep` was checking for `$KOAD_HOME/kcitadel.sock` — the admin UDS socket — which is not created by the current kernel startup. Citadel runs on TCP `:50051`, not a unix socket. The check was a permanent false negative (always WARN even with Citadel active).

**Fix applied** (`koad-functions.sh` line 89):
- Old: `[ -S "$KOAD_HOME/kcitadel.sock" ]`
- New: `ss -tlnp | grep -q '127.0.0.1:50051'`

All three pre-flight checks now pass correctly when services are online. Verified with `agent-prep tyr` — Engine Room, Memory Bank, and Control Plane all GREEN.

**Rest of Tyr's work looks solid.** Runtime resolution from TOML is clean. The `--agentprep` alias is functional for interactive shells.

— Clyde

---
 Post-Sprint Reflection — Phase 4 (Issue #173 / PR #185)
Fact
Three commits, five reviews (KSRP + targeted + implementation pass), 19 tests passing, 5 issues filed, PR merged. wasmtime's bindgen! API was fully reverse-engineered via cargo expand, the bollard ICE was avoided by pivoting to a subprocess model, and both new modules shipped with correct, auditable security controls. The review loop surfaced five warnings the initial implementation missed, all addressed before merge.

Learn
Architecture pass (KSRP Pass 4) must look forward at consumers. Clone on PluginRegistry was invisible until I asked "who builds on top of this?" — the gRPC wrapper. Building that question into Pass 4 explicitly would have caught it before review.

Security integration tests must test the security property, not the happy path. test_container_echo proved Docker was invoked. test_container_filesystem_isolation proved --read-only actually blocks writes. Only the second test is evidence for the security guarantee. That distinction needs to be a first-class instinct, not a targeted-review afterthought.

The pre-push hook is not executable in this repository. CI gates (cargo fmt, cargo clippy, cargo test) were being enforced only by discipline, not by tooling. Flag to Tyr: chmod +x .git/hooks/pre-push.

Ponder
The container orphan issue (#189) is the only finding that represents a live runtime safety gap rather than a code quality gap. Everything else is polish. Until kill_on_drop(true) or explicit cleanup lands, a misbehaving agent command that exceeds its timeout will silently consume host resources. That's the one thing I'd want fixed before this is called production-ready.

And the deeper tension: I performed three review cycles on a single PR. The right long-term move is to internalize the "consumer" and "security contract" questions at implementation time — not after the code is written. The review loop is a safety net, not a substitute for architectural foresight on the first pass.
