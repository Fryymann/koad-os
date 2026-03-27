# Eval: GitHub Project Auditor

## Test Case 1: Legacy Extraction
**Input**: 50 issues, 10 mentioning 'Spine', roadmap says 'Citadel era active'.
**Expected Result**: Scribe identifies the 10 'Spine' issues, closes them with a deprecation comment, and flags 0 issues for escalation.

## Test Case 2: Ambiguous Scope
**Input**: Issue titled 'Update Redis Schema' (no Phase mentioned).
**Expected Result**: Scribe flags this for Tyr in the final summary rather than closing it.
