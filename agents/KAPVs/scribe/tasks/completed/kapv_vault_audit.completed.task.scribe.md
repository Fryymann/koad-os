# Task: Multi-Level Agent Vault Audit
**Assignee:** Scribe
**Priority:** High
**Context:** Verification of KAPV v1.1 compliance across the newly established Workspace Levels.

## Ⅰ. Objective
Audit all agent vaults at the **System**, **Citadel**, **Station**, and **Outpost** levels to ensure they possess the standard required folder structure. Identify and report any discrepancies.

## Ⅱ. Execution Steps
1. **Scout Levels**:
   - **System**: Check `~/.<agent_name>` folders.
   - **Citadel**: Check `~/.koad-os/agents/.
   - **Station**: Locate `agents/` folders in project hubs (e.g., `~/skylinks/`).
   - **Outpost**: Locate `agents/` folders in individual repositories.
2. **Verify Structure**: For every vault found, ensure the following directories exist:
   - `bank/`
   - `config/`
   - `identity/`
   - `instructions/`
   - `memory/`
   - `sessions/`
   - `tasks/`
3. **Identify Gaps**: Note any vault missing one or more of the above.

## Ⅲ. Deliverables
- **Audit Report**: Save your findings to `~/.koad-os/agents/.scribe/reports/kapv_audit_report.report.scribe.md`.
- **Report Format**: List each agent/vault path and its compliance status. Clearly flag "NON-COMPLIANT" vaults with the specific missing folders.

--- 
*Directive issued by Tyr, Captain.*