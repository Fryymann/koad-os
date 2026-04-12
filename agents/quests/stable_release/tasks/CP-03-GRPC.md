# Task Spec: CP-03-GRPC (gRPC Error Boundary Polish)

**Mission:** KoadOS v3.2.0 "Citadel Integrity"
**Agent:** Clyde (Implementation Lead)
**Status:** TODO
**Priority:** Beta

## Objective
Refactor the `koad` and `koad-agent` CLIs to intercept opaque `tonic::Status` and connection errors, translating them into human-readable, actionable guidance for users. The approach will utilize a custom `KoadGrpcError` wrapper that implements `std::error::Error` for seamless integration with `anyhow`.

## Scope & Impact
- **Affected Crates:** `crates/koad-cli` (specifically `bin/koad-agent.rs`, `main.rs`, `utils.rs`, and command handlers).
- **Core Abstraction:** Introduce `koad_cli::utils::errors::KoadGrpcError` to wrap raw `tonic` transport and status errors.
- **Impact:** Users will no longer see "status: Cancelled, message: h2 protocol error" or "Connection Refused." Instead, they will receive stylized prompts like `\x1b[31m[OFFLINE]\x1b[0m KoadOS Citadel is not reachable. Run 'koad system start' to ignite the kernel.`

## Implementation Steps for Clyde
1.  **Define Error Wrapper:** Create a new module `koad-cli/src/utils/errors.rs`. Define `pub enum KoadGrpcError` that implements `std::fmt::Display` and `std::error::Error`.
2.  **Mapping Logic:** Inside the `Display` implementation or via `From<tonic::Status>`, parse the underlying `tonic` errors:
    *   **Connection Refused (Transport):** Suggest `koad system start`.
    *   **Permission Denied (Status):** Suggest checking the agent's rank or role.
    *   **Not Found (Status):** Suggest the session or entity is missing or expired.
    *   **Unavailable (Status):** Suggest checking if the specific service (like CASS) crashed.
3.  **Update Utils:** Refactor existing client creation functions (e.g., `get_citadel_client` in `utils.rs`) to return the new error type wrapped in `Result`.
4.  **Agent Boot Flow Integration:** Modify `crates/koad-cli/src/bin/koad-agent.rs` to catch the `tonic::transport::Error` and `tonic::Status` in the `tokio::spawn` blocks and print the mapped, user-friendly message instead of the raw `eprintln!` dump.
5.  **CLI Handlers Integration:** Systematically update handlers (e.g., `boot.rs`, `system.rs`, `intel.rs`) that communicate with Citadel or CASS to surface the wrapped errors.

## Verification
-   **Manual Testing:**
    -   Stop the Citadel kernel (`koad system stop`).
    -   Run `koad-agent boot <agent>` and verify the output clearly instructs the user to start the system.
    -   Run `koad whoami` or `koad system status` and verify similar graceful failure messages.
