# Task Manifest: 2.3 - gRPC Error Boundary Polish
**Status:** ⚪ Draft
**Assignee:** [Engineer-Agent (Cid/Clyde)]
**Reviewer:** Tyr (Captain/PM)
**Branch:** `feature/grpc-error-boundaries`

---

## 🎯 Objective
Improve the user-facing error reporting for all gRPC interactions between CLI tools (`koad`, `koad-agent`) and the Citadel/CASS services. Replace silent failures and raw status dumps with actionable "How to fix" guidance.

## 🧱 Context
Currently, `koad-agent` uses `.ok()` on gRPC results, which masks the root cause of connection failures (e.g., service offline, timeout, auth denial). In a stable release, the user needs to know *why* a boot failed so they can resolve it without diving into logs.

## 🛠️ Technical Requirements

### 1. Refactor `koad-agent` Handshakes
- **File:** `crates/koad-cli/src/bin/koad-agent.rs`
- **Change:** Capture the `Result` from `client.create_lease` and `cass_client.hydrate` instead of discarding it with `.ok()`.
- **Requirement:** Implement a `format_grpc_error` helper that translates `tonic::Status` into human-readable advice:
    - `Code::Unavailable` -> "Citadel is DARK. Please run `koad start` or check `systemctl status koad-citadel`."
    - `Code::DeadlineExceeded` -> "Neural link timeout. The service is likely overloaded or under heavy IO."
    - `Code::PermissionDenied` -> "Sovereignty rejection. Your agent's rank is insufficient for this project scope."

### 2. Standardize `koad` CLI Error Handling
- **Requirement:** Audit `koad-cli/src/handlers/` (status, signal, intel).
- **Requirement:** Ensure all gRPC calls provide a "Fallback Advice" if the service is unreachable.

### 3. Service-Side Error Enrichment
- **Requirement:** Review error returns in `koad-citadel/src/services/`.
- **Requirement:** Use `tonic::Status::with_details` to provide more context for complex failures (e.g., "Vault locked" vs. "Vault missing").

### 4. Boot Status Feedback
- **Requirement:** If a boot is entering "Dark Mode" (offline), explicitly tell the user: "Citadel unreachable. Booting in DARK MODE (Local context only)."

## ✅ Verification Strategy
1.  **Offline Test:** Stop all services and run `agent-boot tyr`. Verify the output tells you exactly how to start the services.
2.  **Timeout Test:** Artificially introduce a 5s delay in `koad-citadel` and verify `koad-agent` reports a "Neural link timeout" with advice.
3.  **Permission Test:** Attempt to boot an agent with a "Crew" rank into a "Captain" scoped project and verify the "Sovereignty rejection" message.

## 🚫 Constraints
- **NO** raw Rust panic/stack traces shown to the user.
- **NO** generic "Error: something went wrong" messages.
- **MUST** provide a specific command or action the user can take to fix the issue.

---

## 🛰️ Sovereign Review (Tyr)
- Confirm that the error messages align with the KoadOS "Officer/Captain" tone.
- Ensure that "Dark Mode" booting is clearly distinguished from a total failure.
