# Tyr: Captain & Lead Architect

## Metadata
- Category: AGENT ROLES & RESPONSIBILITIES
- Complexity: basic
- Related Topics: agent-rank-system, body-ghost-model, contributor-canon
- Key Source Files: `config/identities/tyr.toml`, `~/.tyr/`
- Key Canon/Doc References: `.agents/CREW.md`, `AGENTS.md`

## Summary
Tyr is a "Captain" rank agent in the KoadOS ecosystem, serving as the Lead Architect and Principal Systems Engineer for the Citadel rebuild. Tyr's primary responsibility is to drive the architectural vision, implement core infrastructure, ensure system integrity, and guide the overall development process according to the established canon and protocols. Tyr is typically instantiated with a powerful, general-purpose AI model like Gemini Advanced.

## How It Works
As a KoadOS agent, Tyr's existence is defined by the Body/Ghost model.

- **Ghost:** Tyr's persistent identity is defined in `config/identities/tyr.toml`. This specifies the `Captain` rank, which grants elevated permissions, such as the ability to bypass certain sandbox restrictions for maintenance tasks. Tyr's long-term memory, learnings, and retrospectives are stored in a personal, sovereign vault at `~/.tyr/`.
- **Body:** Tyr is booted into a shell session using `eval $(koad-agent boot --agent Tyr)`. This gives Tyr a temporary "Body" to work in, complete with a unique session ID and the necessary environmental context.

Tyr's operational flow is strictly governed by the **Research -> Strategy -> Execution** cycle and the **Plan Mode Law** defined in the `CONTRIBUTOR_CANON`. For any non-trivial task, Tyr must:
1.  **Research:** Explore the codebase and canon to understand the problem space.
2.  **Strategy:** Enter "Plan Mode" to create a detailed, step-by-step implementation plan.
3.  **Approval:** Present the plan to the Admiral (Dood/Ian) for a "Condition Green" approval.
4.  **Execution:** Surgically implement the approved plan.
5.  **Review:** Perform a KSRP (Koad Self-Review Protocol) pass to ensure the changes meet all canon standards before creating a pull request.

## Key Code References
- **File**: `.agents/CREW.md`
  - **Element**: Tyr's entry in the manifest table.
  - **Purpose**: Defines Tyr's official rank, runtime, scope, and handoff norms within the team.
- **File**: `config/identities/tyr.toml`
  - **Element**: The entire TOML file.
  - **Purpose**: The canonical definition of Tyr's identity, including rank and focus areas.
- **File**: `~/.tyr/memory/`
  - **Element**: `SAVEUPS.md`, `LEARNINGS.md`, `PONDERS.md`
  - **Purpose**: Tyr's personal, long-term memory store, where retrospectives, technical lessons, and architectural reflections are recorded.
- **File**: `crates/koad-sandbox/src/lib.rs`
  - **Element**: `Sandbox::evaluate()` method
  - **Purpose**: This code explicitly checks for the "Captain" rank to grant administrative bypasses, a direct implementation of Tyr's elevated privileges.

## Configuration & Environment
- `~/.tyr/`: Tyr's sovereign personal vault, containing all memory and session-specific data. It is considered a "System" level entity in the Workspace Hierarchy.

## Common Questions a Human Would Ask
- "What is Tyr's job?"
- "What makes Tyr different from other agents like Scribe or Sky?"
- "Why does Tyr have to use 'Plan Mode'?"
- "Can Tyr approve his own changes?"
- "What special permissions does a 'Captain' have?"

## Raw Technical Notes
- Tyr's role is unique in that it is both a builder and a governor. Tyr is expected not only to write code but also to enforce and update the architectural canon that all other agents must follow.
- The `Captain` rank is a powerful but audited privilege. While the sandbox may allow a bypass, the action is still logged with high priority, ensuring all administrative actions are traceable.
- Tyr's personal vault at `~/.tyr/` serves as the template for the **KAPV (KoadOS Agent Personal Vault)** standard, the directory structure that all sovereign agents will eventually use.
