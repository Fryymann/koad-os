# Plan: Issue #162 - Admin Override (UDS Maintenance Path)

## Objective
Ensure emergency maintenance capability by implementing a dedicated Unix Domain Socket (UDS) listener in the Citadel that bypasses standard gRPC authentication for administrative tasks.

## Key Files
- `proto/citadel.proto`: Add `Admin` service and `ADMIN_OVERRIDE` role.
- `crates/koad-citadel/src/services/admin.rs`: Implement the `Admin` service.
- `crates/koad-citadel/src/kernel.rs`: Set up the UDS listener and serve the `Admin` service.
- `crates/koad-citadel/src/services/mod.rs`: Export the new `admin` module.

## Implementation Steps

### 1. Update Protobuf Definitions
- Add a `Role` enum to `proto/citadel.proto` if it doesn't exist, or add `ROLE_ADMIN_OVERRIDE`.
- Define an `Admin` service with maintenance methods (e.g., `Shutdown`, `GetSystemStatus`, `ForcePurgeSession`).

### 2. Implement Admin Service
- Create `crates/koad-citadel/src/services/admin.rs`.
- Implement the `Admin` trait for `AdminService`.
- Logic for `Shutdown` should trigger the kernel's shutdown signal.
- Logic for `GetSystemStatus` should provide high-level metrics.

### 3. Update Kernel for UDS Support
- In `KernelBuilder::start`:
    - Check if `admin_uds_path` is provided.
    - If yes, set up a `UnixListener` (using `tokio::net::UnixListener`).
    - Create a second `tonic::transport::Server` instance for the UDS listener.
    - This server will *not* use the standard `auth_interceptor` (or will use a specialized one).
    - Serve the `Admin` service and potentially other services (Session, Sector, etc.) on this UDS listener.

### 4. Verification & Testing
- Unit test the `AdminService` logic.
- Integration test: Attempt to connect to the UDS socket and call an `Admin` method without standard headers.
- Verify that the UDS socket is only accessible to the local user (default behavior of UDS in the designated directory).

## Security Considerations
- The UDS socket should be placed in a directory with restricted permissions (e.g., `~/.koad-os/`).
- The `ADMIN_OVERRIDE` bypass in the interceptor should be hardened to only work if the connection is indeed local/UDS (though serving on a separate listener without the interceptor is cleaner).
