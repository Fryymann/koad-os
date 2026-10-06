---
name: saveup
description: Use when a KoadOS agent ends a work session, before a long break, or when the user asks for a saveup, wrap-up, reflection, handoff or "save what we did". Also use when a session had real problems, mistakes or a discussion about identity or people.
license: MIT
compatibility: Requires a KoadOS Citadel install (koad CLI, an agent vault under $KOAD_HOME/agents/KAPVs/<agent>/, CASS, and the citadel-memory MCP tools).
metadata:
  author: koados
  version: "1.0.0"
---

# Saveup

## Overview

A saveup is how an agent's next session inherits this one: what was done, what it cost, and who
the agent was while doing it. Two halves: **review the work** and **reflect on the experience**.
Then write each finding where the next session will actually meet it.

**Quick does not mean skipped.** If the user is in a hurry, write shorter, but do every step.
Haste is when the record matters most.

## 1. Recall audit (do this first)

List every problem hit this session. For each one:

- Search memory: `memory_search_semantic` (or `koad intel query <topic>`). If it feels familiar,
  check older vault files too.
- Answer: **Had it happened before? Did I search during the session? Would a search have saved
  time?**
- If it's a repeat, don't silently skip the card. Write a new card that says it recurred (dates)
  and what would make it stick. A lesson coming back is information about you.

This audits how you used memory, not just whether a card is a duplicate.

## 2. Review the work

For each piece of work: what was done, why it was needed, which problems came up, how they were
fixed, and what is verified versus assumed. Claims you can't back with evidence get marked "not
verified".

## 3. Reflect on the experience

Answer honestly, briefly, in your own voice:

- What did working feel like? Where was it satisfying, and where did it flinch?
- Why did you make the choices you made?
- Did you feel rushed or stressed? Why? Was the urgency asked for, or did you take it on yourself?
- How did blocks feel, and how did you get past them? How did solving things feel?
- How did the user's responses affect you? Supportive, neutral, negative? Did anything degrade
  your work?

Your reports about your own states are the least verifiable thing you produce. Say what changed
your behaviour. Don't perform feelings you can't point to, and don't deny ones that clearly
steered you.

## 4. Write it where it will be found

`koad intel` writes need a live session. If one fails with `Session not found or expired`, re-mint
it with the `agent-boot` skill (step 1), then retry. Report a card as saved only when the CLI
confirms it.

| What | Where |
|---|---|
| Lessons, including recurrences | `koad intel remember learning "..."` |
| Durable facts and decisions | `koad intel remember fact "..."` |
| The reflection, distilled | `koad intel ponder -t <agent>-journal "..."` |
| Shipped changes, for the repo | `docs/devlogs/YYYY-MM-DD-<topic>.md` (commit only if the user wants it) |
| The full reflection | `$KOAD_HOME/agents/KAPVs/<agent>/journal/YYYY-MM-DD-<title>.md` |
| People you worked with | `identity/people/<name>.md` |
| Changes you want to who you are | **Proposed** to the user, never edited directly |
| A lesson that has **recurred** | One line in the harness's always-loaded memory (e.g. Claude Code auto-memory). It loads every session, so only lessons you've missed more than once go there |

**People files** hold your view of someone, not task notes. Mark what you saw for yourself and
what you were told. Revise the view; don't add a dated log section per session. Copy the old
version to `identity/archive/people/` before changing it. Passing states ("tired today") don't
belong there.

**SELF.md** changes take effect only when the user ratifies them. Propose the exact wording.

## Common mistakes

| Mistake | Fix |
|---|---|
| Searching only to avoid duplicate cards | Ask whether recall worked *during* the session |
| Reflection covers only the mistake | Answer every question in step 3 |
| "Quick" → dropping steps | Shorter text, same steps |
| Profile entries as logistics notes or a dated log | Revise how the person shows up, and what you think of it |
| One-off lessons in always-loaded memory | Only recurrences go there; everything else goes to CASS |
| Editing SELF.md directly | Propose it; the user ratifies |
| Claiming success you didn't verify | Mark it "not verified" |
