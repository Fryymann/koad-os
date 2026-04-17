## Purpose
Implement a github-audit skill that gives a designated KoadOS agent the ability to audit a GitHub repository's Issues backlog and GitHub Project board — producing a structured, actionable intel report that Ian (and other agents) can consume for prioritization, triage, and sprint planning.
This skill is intended to replace ad-hoc, manual issue review sessions with a repeatable, agent-driven audit process aligned with the KoadOS Canon and Git Flow Specification.
---
## Problem Statement
The agents-os and sws-airtable-api repos accumulate GitHub Issues and Project board items faster than they can be manually reviewed. Currently:
- There is no automated or agent-driven process to audit issue health (staleness, duplicates, missing labels, orphaned items).
- GitHub Project boards drift out of sync with actual development state — cards go stale, columns don't reflect reality.
- Ian must manually triage issues before sprint planning, which is a high-context, time-consuming task that is well-suited for agent delegation.
- No KoadOS agent currently has a defined skill for GitHub Project or issue-level analysis.
A github-audit skill addresses all four problems in a single, repeatable invocation.
---
## Skill Anatomy
```javascript
.koad-os/skills/<agent>/github-audit/
├── SKILL.md                      # Skill definition + trigger patterns
├── scripts/
│   ├── audit-issues.sh           # Main orchestrator
│   ├── fetch-issues.py           # GitHub API client — issues + labels + assignees
│   ├── fetch-project.py          # GitHub API client — Project board cards + columns
│   ├── classify-issues.py        # Classify each issue: stale, duplicate, missing-label, etc.
│   └── generate-report.sh        # Render final markdown report
├── drivers/
│   └── <driver_type>/
│       └── infer.py              # AI-assisted classification / summarization (driver-specific)
├── config/
│   └── skill.toml                # Skill config: driver_type, target repos, thresholds
├── references/
│   └── audit-log.md              # Auto-generated log of last audit
└── _eval/
    ├── test-prompts.md
    └── grading-schema.md
```
---
## SKILL.md Specification
### Frontmatter
```yaml
name: github-audit
driver: <driver_type>      # Set at deployment per agent's provider
description: >
  Audit a GitHub repository's Issues backlog and Project board.
  Produces a structured report classifying issues by health, staleness,
  label coverage, and project board alignment. Use when Ian asks to
  review the backlog, audit issues, check project board health,
  or prepare for sprint planning.
trigger_patterns:
  - "Audit the GitHub issues"
  - "Review the backlog"
  - "Check the project board"
  - "Prepare for sprint planning"
  - "What issues need triage?"
  - "How healthy is the repo?"
  - "Audit * repo"
requires:
  - python3
  - GITHUB_TOKEN               # Read-only PAT with repo + project scope
tier: crew
context_cost: medium           # Report is rendered to file; only summary enters context
author: tyr
version: 1.0.0
```
### skill.toml
```toml
[skill]
name = "github-audit"
version = "1.0.0"
driver_type = "<driver_type>"     # Set at deployment

[config]
default_repo = "Skylinks-Golf/agents-os"
staleness_threshold_days = 30
max_issues_per_run = 200
report_output = ".koad-os/docs/audits/github-audit-<date>.md"

[[repos]]
name = "agents-os"
owner = "Skylinks-Golf"
repo = "agents-os"
project_number = 1            # Update with actual GitHub Project number

[[repos]]
name = "sws-airtable-api"
owner = "Skylinks-Golf"
repo = "sws-airtable-api"
project_number = 2            # Update with actual GitHub Project number
```
---
## Audit Report Structure
The skill outputs a structured markdown report to .koad-os/docs/audits/github-audit-<date>.md:
```javascript
# GitHub Audit Report — <repo> — <date>

## Executive Summary
- Total issues scanned: [N]
- 🟢 Healthy: [N]
- 🟡 Needs attention: [N]
- 🔴 Stale / blocked: [N]
- Project board drift: [N] cards out of sync

## Triage Queue (Action Required)
| Issue # | Title | Age | Problem | Suggested Action |
|---|---|---|---|---|

## Stale Issues (> 30 days, no activity)
| Issue # | Title | Age | Last Actor |
|---|---|---|---|

## Missing Labels
| Issue # | Title | Suggested Labels |
|---|---|---|

## Project Board Health
| Column | Expected | Actual | Drift |
|---|---|---|---|

## Orphaned Cards (Project ↔ Issue mismatch)
| Card | Issue Status | Project Status |
|---|---|---|

## Recommendations
<!-- Prioritized next steps for Ian/Dood to approve -->
```
---
## Open Questions
---
## Constraints & Rules of Engagement
- Read-only by default. The skill must never create, edit, close, or label issues without an explicit --write flag and Ian's confirmation.
- Driver-Variant compliant. This skill uses AI inference for classification and summarization. It must follow the Driver-Variant Skill Pattern (see: KoadOS Feature Request — Single-Provider Skill Variants per Sovereign Agent).
- Canon compliance. Tyr must open a GitHub Issue in agents-os before writing any code.
- Report to file. The full audit report is written to disk. Only the Executive Summary enters the agent's context to control token cost.
- GitHub token security. The GITHUB_TOKEN must be read from environment or .koad-os/config/secrets/ only. Never logged or committed.
---
## Success Criteria
- Ian can say "Audit the GitHub issues" and the skill produces a complete, structured report.
- Stale issues (> 30 days inactive) are correctly identified and listed.
- Project board drift (cards whose linked issues are closed or missing) is detected.
- The Executive Summary fits in agent context without loading the full report.
- The skill is driver-variant compliant and passes eval at ≥ 80% on the test scenario suite.
- No write operations occur without explicit --write flag + Ian confirmation.
---
> Note to Tyr: This skill is blocked on model selection. Do not begin implementation until Ian resolves the open questions above. Once the target agent and driver are confirmed, update skill.toml and the drivers/ directory accordingly.
