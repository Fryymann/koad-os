# Domain: Protocols (proto/)
**Role:** Canonical Service Definitions

## Ⅰ. Active Definitions
- [citadel.proto](citadel.proto): Control plane, session management, and auth.
- [cass.proto](cass.proto): (Phase 2) Memory query, hydration, and inter-agent signals.

## Ⅱ. Guidelines
- All mutations MUST include `TraceContext`.
- Zero-trust architecture: Auth is enforced at the service boundary.
