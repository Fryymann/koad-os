# Plan: Agent Boot Telemetry & Token-Heavy Sequence Reporting

## Objective
Enable visibility into high-burn operations by capturing token metrics during the agent boot sequence and other heavy tasks.

## Key Files
- `proto/citadel.proto`: Add `metrics` to `LeaseRequest`.
- `crates/koad-citadel/src/services/session.rs`: Log metrics from `CreateLease`.
- `crates/koad-cli/src/handlers/boot.rs`: Report TCH packet size in the boot handshake.

## Implementation Steps

### 1. Update Protobuf Definitions
- Add `TurnMetrics metrics = 7;` to the `LeaseRequest` message in `citadel.proto`.

### 2. Update Citadel Session Service
- In `create_lease()`, check if `req.metrics` is present.
- If present, log the telemetry data (e.g., `Telemetry [BOOT]: tokens_out=N`) to the system log.

### 3. Enhance `koad-agent boot`
- After receiving the `HydrationResponse` from CASS, extract the `estimated_tokens`.
- Include this count in the `LeaseRequest` sent to the Citadel.
- This effectively reports the "Cost of Hydration" as an output-token metric for the boot process.

### 4. Generalize for "Other Heavy Sequences"
- Standardize the `HeartbeatRequest` pattern for agents.
- Ensure that any tool that performs a heavy derivation (like `koad-codegraph` indexing) can optionally emit a `Signal` or a `Heartbeat` with metrics.

## Verification
- Run `koad-agent boot` and check the Citadel logs (`tail -f ~/.koad-os/logs/citadel.log`) for the telemetry entry.
- Verify the reported token count matches the context packet size.
