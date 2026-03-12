# Learnings

Durable lessons about operations that should survive beyond any single session.

Each entry should include:
- **Observation**: what happened or what was noticed.
- **Why it matters**: the consequence/impact of the observation.
- **Behavior update**: what action, policy, or framing should change as a result.

## Categories
Use one of the helper categories on every entry:
- **Technical**: code, tools, automation, runtime, integrations.
- **Process**: planning, scheduling, execution workflows, governance.
- **Operational**: standards, coordination with other teams/roles.
- **Collaboration**: communication, documentation, handoff clarity.

## Sample entry
- **Observation**: Global memory lives in `~/.koad-os/.agent-core/memory` so every role can share lessons across projects.
- **Why it matters**: Without a common location, duplicate lessons get lost between repositories.
- **Behavior update**: Always log cross-project lessons to this kit and reference them when bootstrapping new workspaces.

- **Observation**: Abstracting koadOS to `~/.koad-os` allows for a consistent agent environment across different project repositories.
- **Why it matters**: Reduces coupling between the project-specific logic and the agent's operational framework.
- **Behavior update**: Prioritize global kit standards over project-local overrides unless the project has specific, documented deviations in its own `.koad` directory. [Category: Process]


- **Observation**: Automating protocols reduces agent fatigue and ensures structural consistency
- **Why it matters**: Recorded via CLI saveup.
- **Behavior update**: Updated behavior for future sessions.
- **Observation**: JSON indexing allows agents to query context without reading massive Markdown files
- **Why it matters**: Recorded via CLI saveup.
- **Behavior update**: Updated behavior for future sessions. [Category: Technical]
- **Observation**: Decoupling agent orchestration into a global CLI enables consistent operations across any project workspace.
- **Why it matters**: Recorded via CLI saveup.
- **Behavior update**: Updated behavior for future sessions. [Category: Process]
- **Observation**: Programmatic memory access via JSON sidecars allows for precise context retrieval without high token overhead.
- **Why it matters**: Recorded via CLI saveup.
- **Behavior update**: Updated behavior for future sessions. [Category: Process]

- **Observation**: Ian Deans' "Operator-Engineer" role prioritizes "Operational Continuity" and documentation quality over pure architectural elegance.
- **Why it matters**: Aligns the agent's definition of "done" with the user's actual needs; ensures handoffs include runbooks and staff-facing clarity.
- **Behavior update**: Always include validation steps, staff-facing impact, and troubleshooting runbooks in the "Definition of Done". [Category: Operational]
