# KAI Sovereignty & Driver Protocol

## 1. Overview
This protocol formalizes the separation between **Koad Agent Identities (KAIs)** and the **Cognitive Drivers** (AI models) that power them. KoadOS provides the infrastructure to ensure that identities remain consistent, exclusive, and sovereign across the system.

## 2. Core Definitions
- **KAI (Koad Agent Identity):** The persistent "soul" of the agent. It contains the name, bio, role, permissions, and cognitive tier requirement (1=Admin, 2=Dev, 3=Guest).
- **Driver:** The "cognitive engine" (Gemini, Codex, Claude). Drivers are interchangeable tools used to animate a KAI.
- **Lease:** An exclusive, time-bound lock on a KAI managed by the Spine. Only one driver can hold a lease on a specific KAI at any given time.

## 3. Multi-Layered Defense Architecture
Identity protection is enforced at both the edge (CLI) and the core (Spine) to ensure maximum resilience.

### 3.1 Layer 1: Local Registry Enforcement (koad-cli)
Before attempting a remote uplink, the CLI performs a local verification against the SQLite Identity Registry:
- **Role Verification:** Ensures the requested role is authorized for the KAI.
- **Cognitive Protection:** Rejects the boot attempt if the local Model Tier is lower than the KAI's requirement (e.g., Koad requires Tier 1).

### 3.2 Layer 2: Centralized Lease Management (kspine)
The Spine serves as the final authority for identity uniqueness:
- **Mutual Exclusion:** Prevents concurrent sessions for the same KAI (`IDENTITY_LOCKED`).
- **Sovereign Guardrails:** Hard-locked logic in `KAILeaseManager` ensures `Koad` and `Ian` identities are never leased to non-Admin drivers.
- **Automated TTL:** Leases expire after 90 seconds of inactivity unless extended by a heartbeat.

## 4. Lifecycle & Heartbeat
1. **Checkout:** `koad boot` calls `InitializeSession` via gRPC to acquire the lease.
2. **Persistence:** The CLI maintains a background loop that calls `rpc Heartbeat` every 30 seconds.
3. **Extension:** Each heartbeat extends the KAI lease by an additional 90 seconds.
4. **Check-in:** Graceful termination (Ctrl+C) explicitly releases the lease.

## 5. Testing & Validation
The sovereignty protocol is verified via the E2E suite: `tests/e2e/test_kai_sovereignty.py`.
- `test_kai_mutual_exclusion`: Verifies that a locked KAI cannot be checked out by a second driver.
- `test_kai_sovereign_lock`: Confirms that Tier 2 drivers (Codex) are rejected when attempting to boot Tier 1 KAIs (Koad).
- `test_kai_lease_cleanup`: Ensures leases are cleared and reusable after session termination.

## 6. Tracing & Telemetry
The `KAILeaseManager` emits structured tracing events to the system log:
- `KAI Lease Acquired`: `[identity] -> [session_id] (Driver: [driver])`
- `KAI Lease Released`: `[identity] (Session [session_id])`
- `COGNITIVE_REJECTION`: Logged when a low-tier driver attempts to mount a sovereign identity.

## 7. Implementation Status
- [x] Unify Redis Socket Mapping (`koad.sock`)
- [x] Implement Heartbeat Loop in `koad-cli`
- [x] Refactor `spine.proto` for gRPC Lease Management
- [x] Implement `KAILeaseManager` in `kspine`
- [x] Enforce Sovereign Guardrails (Koad/Ian)
- [x] Verify Protocol via E2E Sovereignty Suite
- [x] Update `koad crew` manifest for real-time lease tracking
