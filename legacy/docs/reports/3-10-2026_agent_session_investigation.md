# Investigation Report: Agent Session & Event Stream Failures
**Date:** 2026-03-10
**Investigator:** Tyr (Captain)
**Status:** DISRUPTED (Condition Yellow)

## 1. Executive Summary
The KoadOS Web Deck currently fails to display active agents and an event stream due to a series of architectural misalignments and subscription gaps between the Spine, the Gateway, and the Frontend. While core telemetry (CPU/Memory) is functional, identity-aware data is being dropped at multiple points in the pipeline.

## 2. Root Cause Analysis

### A. Status String Mismatch (The "WAKE" vs "active" Conflict)
There is a fundamental discrepancy in how different components represent agent status:
*   **Kernel Core (`AgentSession`):** Defaults to `"active"`.
*   **Spine Diagnostics (`update_crew_manifest`):** Uses `"WAKE"` and `"DARK"`.
*   **Gateway (`kgateway`):** Hard-coded to filter for `status == "active" || status == "idle"`.
*   **Result:** Because the authoritative manifest uses `"WAKE"`, the Gateway silently filters out all agents during the initial sync and subsequent updates.

### B. Event Stream Subscription Gaps
The "Neural Bus" (Event Stream) is empty because critical channels are not being bridged:
*   **Spine gRPC:** The `stream_system_events` method was missing subscriptions to `koad:telemetry:manifest` and `koad:telemetry:services`.
*   **Gateway:** The Gateway only listens to stats and sessions, ignoring the manifest channel where identity-focused "WAKE" updates are published.
*   **Result:** The frontend never receives the payload containing the crew manifest.

### C. Data Plane vs. Control Plane Disconnect
The system treats "Sessions" (transient terminal instances) and "Identities" (permanent agents like Tyr/Sky) as two different data streams.
*   The Web Deck's "Agents in Session" panel is trying to render **Sessions**, but the system's most accurate health data is in the **Crew Manifest** (Identity-based).
*   The Gateway currently only relays `SESSION_UPDATE` events, but does not have a handler for `CREW_MANIFEST` or `SYSTEM_SYNC` updates that would provide a unified view of available agents.

### D. Frontend State Handling (useKoadFabric Race Condition)
The `useKoadFabric` hook has a logic flaw in how it handles system events:
*   It expects standard log objects with a `.message` field.
*   Many Redis telemetry events (like stats or manifest updates) are raw JSON strings or have different structures.
*   When a non-standard message arrives, the `catch` block attempts to wrap it, but the `if (data.message)` check often fails for manifest payloads, causing them to be ignored or misclassified.

## 3. Design Flaws & Vulnerabilities

1.  **Implicit Schema Dependencies:** The system relies on hard-coded string comparisons ("active", "WAKE") across three different languages/environments (Rust, WebSocket JSON, TypeScript) without a shared source of truth.
2.  **Silent Failures in Diagnostics:** The `prune_orphaned_sessions` logic was aggressively deleting session data due to a missing `created_at` field, causing "Ghosting" where an agent is awake in the terminal but dead in the database.
3.  **Missing "Authoritative" Sink:** There is no single component that aggregates both "Wake Status" (from leases) and "Session Data" (from Redis) into a format the Web Deck can easily consume.

## 4. Proposed Remediation Strategy (Pending Directive)

1.  **Standardize Status Enums:** Move to a unified `AgentStatus` enum in `koad-proto` used by all components.
2.  **Unified Telemetry Relay:** Update the Gateway to subscribe to ALL `koad:telemetry:*` channels and relay them as a standardized `SYSTEM_EVENT` type.
3.  **Frontend Manifest Integration:** Update `CrewQuarters.tsx` to render based on the `CrewManifest` (Identities) rather than just active `Sessions`.
4.  **Lease-Aware Boot:** Ensure every `koad boot` sequence explicitly acquires a lease that is maintained by the heartbeat daemon to prevent premature pruning.

---
*Report End.*
