#!/usr/bin/env bash
# scripts/install-skills.sh — Install the KoadOS agent skills with the Agent Skills CLI.
#
# Installs from the merged nightly branch on GitHub (not the local working tree),
# so the skills are recorded in ~/.agents/.skill-lock.json and `npx skills update`
# refreshes them. Local-path installs are not lock-tracked.
#
# Targets Claude Code and Codex. Hermes Agent is not a target yet: Hermes runs
# tailored forks (docs/reviews/2026-09-26-skills-standardization-review.md, step 4).
#
# Override the source for testing: KOAD_SKILLS_SOURCE=/path/to/repo scripts/install-skills.sh

set -euo pipefail

SKILLS_CLI="skills@1.7.0"
SKILLS_SOURCE="${KOAD_SKILLS_SOURCE:-https://github.com/Fryymann/koad-os/tree/nightly/skills}"
SKILLS_AGENTS=(claude-code codex)

cmd=(npx -y "$SKILLS_CLI" add "$SKILLS_SOURCE" -g -a "${SKILLS_AGENTS[@]}" -s '*' -y)

if ! command -v npx >/dev/null 2>&1; then
  echo "npx not found; skills were not installed. Run: ${cmd[*]}" >&2
  exit 1
fi

echo "Installing KoadOS skills from $SKILLS_SOURCE for: ${SKILLS_AGENTS[*]}"
"${cmd[@]}"
