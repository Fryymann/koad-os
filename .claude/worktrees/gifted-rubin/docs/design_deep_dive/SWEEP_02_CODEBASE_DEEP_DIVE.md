# Design Deep Dive — Sweep 02: Codebase Deep Dive

> [!CAUTION]
> **Status:** PLAN MODE (Technical Audit)
> **Summary:** The Koados Citadel is suffering from "Patchwork Fatigue." We have prioritized feature speed over structural integrity, leading to a system that is brittle, difficult to debug, and prone to state corruption.

---

## 1. Angle 1 Analysis: State Authority (The Split-Brain Problem)
### **Findings:**
- **The Issue:** `AgentSessionManager` (ASM) maintains an in-memory `Arc<Mutex<HashMap>>` while Redis acts as a secondary store. 
- **The Failure:** If the Spine process restarts, the in-memory map is lost. The system then enters a state where Redis says "LOCKED" (from the old process) but the new Spine says "EMPTY."
- **Memory Reflection:** This caused the `IDENTITY_LOCKED` loop encountered on 2026-03-05.

### **Root Cause:** 
**Lack of Atomic State Ownership.** We are using Redis as a "mirror" rather than the "Authority."

---

## 2. Angle 2 Analysis: Persistence Lawlessness (The Schema Void)
### **Findings:**
- **The Issue:** `KoadDB` expects tables (`knowledge`, `identities`) that are not enforced by the binary.
- **The Failure:** `koad intel query` failed today because the `knowledge` table simply didn't exist in the local `koad.db`.
- **Memory Reflection:** Durable memory is currently "accidental" rather than "foundational."

### **Root Cause:**
**Disconnected Migration Strategy.** We have no "Schema Authority" that ensures the database is functional before the CLI or Spine begins operations.

---

## 3. Angle 3 Analysis: Orchestration (Contract Fragility)
### **Findings:**
- **The Issue:** High-frequency use of `string identity_json` inside Protobuf messages.
- **The Failure:** The Spine returned `{}`, and the CLI expected `[]`. This caused a runtime crash that required 10+ debug iterations to diagnose.
- **Memory Reflection:** This bypasses the primary benefit of the Rust stack: compile-time safety.

### **Root Cause:**
**JSON-in-gRPC Anti-Pattern.** We are treating a high-performance neural bus like a loose REST API.

---

## 4. Angle 4 Analysis: Resilience (The False Green)
### **Findings:**
- **The Issue:** `koad status` and `ShipDiagnostics` perform "shallow" checks (Socket exists? Port open?).
- **The Failure:** The system reported "CONDITION GREEN" while the `GetSystemState` endpoint was functionally broken.
- **Memory Reflection:** Passive monitoring creates a false sense of security for the Admiral.

### **Root Cause:**
**Passive Observability.** We lack end-to-end "Functional Probes."

---

## 5. Required Architectural Shifts (The Design Intent)
1.  **Strict Authority:** Redis MUST be the only source of truth for hot state. In-memory Mutexes are for short-term task coordination only.
2.  **Schema Enforcement:** The Spine MUST automatically run migrations/initialization on `koad.db` before accepting gRPC connections.
3.  **Type Sovereignty:** Refactor `spine.proto` to define the `Agent` and `Session` schemas natively. Remove all `identity_json` strings.
4.  **Active Diagnostics:** `koad status` must be replaced with `koad doctor`, which executes E2E functional tests (e.g., "Write a test key to Redis and read it back").

---
*Next Sweep: Defining the Intention and Contract for the v5.0 Core.*
