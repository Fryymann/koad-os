**To:** Tyr
**From:** Scribe
**Date:** 2026-03-15
**Subject:** Audit of 'koad' Command Usage

**Overview:**
An audit of 'koad' command usage was performed. The search primarily revealed references to 'koad' within project dependencies, protobuf definitions, and documentation files. Direct shell command invocations of a primary `koad` executable (e.g., `koad <subcommand>`) were not found in the searched files.

**Findings:**

1.  **No direct `koad <subcommand>` invocations found:**
    *   **Issue:** No explicit shell command usages of `koad` (e.g., `koad subcommand`) were found in the codebase that would allow for auditing help descriptions or verifying functionality. It's unclear if the `koad` CLI tool itself is implemented and how its subcommands are invoked.
    *   **Classification:** Error
    *   **Recommendation:** Investigate the `crates/koad-cli/` directory to understand the structure and entry points of the `koad` CLI tool. Identify its subcommands and verify their help documentation and functionality.

2.  **`koad-agent boot` Command:**
    *   **Observation:** This command is frequently referenced in planning and boot-related documentation (`plans/tch-context-packet.md`, `plans/boot-telemetry.md`, `new_world/archived/tyr_plan_review.plan.old.md`, and agent `AGENTS.md` files).
    *   **Help Description/Functionality:** The documentation suggests its purpose (bootstrapping agents, context hydration) and discusses potential improvements (e.g., writing context files, emitting signals). However, direct auditing of its help text or runtime functionality is not possible from the current search results.
    *   **Classification:** Info
    *   **Recommendation:** The implementation in `crates/koad-cli/src/handlers/boot.rs` should be reviewed to ensure comprehensive help descriptions and robust functionality, aligning with the planning documents.

3.  **`koad system doctor` Command:**
    *   **Observation:** This command is proposed in `new_world/archived/tyr_plan_review.plan.old.md` as a potential tool for automated verification of the new tri-tier environment.
    *   **Help Description/Functionality:** There is no evidence of its implementation or availability in the search results.
    *   **Classification:** Warning
    *   **Recommendation:** Investigate if `koad system doctor` has been implemented. If not, it may need to be developed as part of the system verification tooling.

**Actionable Items:**
*   Investigate the `crates/koad-cli/` directory for the implementation of the `koad` CLI tool and its subcommands.
*   Audit the help descriptions and functionality of the `koad` CLI subcommands.
*   Verify the implementation and documentation of `koad-agent boot`.
*   Investigate the existence and functionality of the proposed `koad system doctor` command.