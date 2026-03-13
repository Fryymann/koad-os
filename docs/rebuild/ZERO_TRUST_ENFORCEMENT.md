# Zero-Trust Enforcement (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #153

## 1. Requirement
Sanctuary Rule must be enforced at the gRPC layer, not the agent layer.

## 2. Auth Context (TraceContext)
Every gRPC call MUST carry a `TraceContext` in its metadata.
- **Fields:** `trace_id`, `origin`, `actor`, `timestamp`.
- **Action:** Rejection if missing or malformed (`UNAUTHENTICATED`).

## 3. Protected Keys & Sectors
The following Redis/State keys are restricted to Admin (Dood) only for writes:
- `identities`, `identity_roles`, `knowledge`, `principles`, `canon_rules`.
- **Action:** `PERMISSION_DENIED` if an agent attempts a write.

## 4. Jailing (Sanctuary Rule)
The Citadel server MUST look up the agent's assigned `KOAD_WORKSPACE_ROOT` from its identity registry.
- **Validation:** Every tool call involving a file path (`read`, `write`, `replace`) must be validated against this root.
- **Action:** Block operations outside the root with `OUT_OF_RANGE` or `PERMISSION_DENIED`.

## 5. Deployment
Enforce via Tonic `Interceptor` or a custom Tower `Layer` on the Citadel gRPC server.