# EndOfWatch (EoW) Schema (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #155

## 1. Requirement
Strictly enforced schema for session wrap-up summaries.

## 2. Format
Markdown file (`.md`) with TOML frontmatter.

## 3. TOML Frontmatter (Mandatory)
```toml
[summary]
task_id = "#146"
status = "IN_PROGRESS"  # IN_PROGRESS | BLOCKED | COMPLETED | ABORTED
trace_id = "TRC-HELM-a1b2"
timestamp = 1773369516
session_id = "20260312_1900"

[[learnings]]
topic = "gRPC Interceptors"
message = "Discovered that Tonic interceptors cannot easily mutate metadata in-flight without a custom layer."

[[next_steps]]
target = "tyr"
action = "Review citadel.proto final signature."
priority = "HIGH"
```

## 4. Markdown Body
The body contains the human-readable log of the session. It must be present and non-empty.

## 5. Validation
Enforced in the `koad_session_save` MCP tool (CASS). CASS will reject submissions that:
1. Lack required TOML fields.
2. Have an empty or invalid `trace_id`.
3. Fail to parse as valid TOML.