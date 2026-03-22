# Skill: GitHub Project Auditor
> **Description:** Realigns GitHub repo issues and project boards with the canonical KoadOS roadmap. Specialized for Flash-Lite agents to handle high-volume mechanical execution.
> **Requires:** [gh-cli, citadel-roadmap, write:repo, project]
> **Tier:** Crew (Scribe)
> **Context Cost:** Medium

## Ⅰ. Purpose
To ensure that the GitHub repository state (issues, labels, milestones) and the Project Board (items, columns) perfectly mirror the current architectural roadmap. This skill offloads the mechanical overhead of bulk issue management from Officer agents.

## Ⅱ. Execution Phases

### Phase 1: Orientation (The Roadmap Source)
1. **Identify the Truth**: Locate the latest roadmap (usually `~/.koad-os/new_world/DRAFT_PLAN_3.md` or as directed by Tyr).
2. **Extract Criteria**: Identify keywords, phase numbers, and "Legacy" indicators from the roadmap.
3. **Map columns**: Identify which roadmap phases map to which Project Board columns.

### Phase 2: Discovery (The Repo State)
1. **Fetch Issues**: Run `gh issue list --limit 100 --state open --json number,title,labels,body`.
2. **Fetch Project**: Run `gh project item-list <project_number> --owner <owner>`.
3. **Cross-Reference**: Identify "Ghost Issues" (in repo but not on board) and "Drift Issues" (pertaining to legacy systems like 'Spine').

### Phase 3: Alignment (Execution)
1. **Bulk Cleanup**: For issues identified as Legacy/Redundant:
   - Close with a standardized comment: `"Absorbed/Deprecated by Citadel Rebuild strategy. See Phase X."`
2. **Board Injection**: Add missing roadmap-relevant issues to the board using `gh project item-add`.
3. **Column Sync**: Move items to the correct project column based on their Phase #.

### Phase 4: Reflection (Self-Improvement)
1. **Pattern Extraction**: Identify 3 keywords that consistently signaled a redundant issue in this run.
2. **Log Learnings**: Append findings to `~/.scribe/memory/LEARNINGS.md`.
3. **Escalate**: If an issue's relevance is ambiguous, DO NOT close. Flag it for Tyr in the final summary.

## Ⅲ. Anti-Patterns (What NOT to do)
- **DO NOT** close issues that are marked `critical` or `security` without explicit human/Tyr confirmation.
- **DO NOT** create new milestones; only use existing ones or leave unassigned.
- **DO NOT** assume all "Spine" mentions are legacy if they are within a "Migration" context.

## Ⅳ. Failure Modes & Fallbacks
- **No Project Scope**: If `gh project` fails due to scopes, execute only the Issue Cleanup pass and provide a manual "Board Action List" for Ian.
- **Rate Limiting**: If `gh` hits a secondary rate limit, pause for 60 seconds or summarize progress and stop.

---
*Created: 2026-03-14 | Version: 1.0*
