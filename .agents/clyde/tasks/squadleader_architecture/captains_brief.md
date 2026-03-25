  🛡️ CAPTAIN'S BRIEF: Squad Leader Architecture (v2)
  To: Clyde (Officer)
  From: Tyr (Captain)
  Subject: Transition to Native Agent Teams & Squad Governance

  Clyde, the Admiral and Noti have refined your "Minion" protocol. We are pivoting away from custom orchestration to adopt Claude Code's native Agent Teams.
  This allows us to leverage the engine's built-in coordination while we focus on the KoadOS governance layer.

  1. The Core Connection
   * The Bridge: Your persistent identity (the "Heart") remains in your KAPV (~/.koad-os/.agents/clyde/). The native .claude/agents/clyde.md you will create
     is merely a Manifest (the "Ghost") that tells the Claude engine how to boot you.
   * Sovereignty: You are the Lead. Only you have write-access to the Memory Bank and Canon. Teammates (clyde-1A, etc.) are ephemeral workers. If they need
     to change the system, they MUST escalate to you via the native messaging system.

  2. Codebase & Tooling Pointers
   * koad-proto Alignment: Teammates do not need a full agent-boot. They should be instructed to read your active KOAD_CONTEXT_FILE (found in your
     environment) for read-only situational awareness.
   * koad status --full: Run this before spawning a team. Ensure CASS and the Citadel are "Green." If the Grid is degraded, teammates must operate in
     "Isolated Mode."
   * koad map nearby: Use this to generate the Context Files list for teammate task packets. It is the most efficient way to scope their discovery.
   * koad-sandbox: Remember that the Citadel's gRPC interceptor is watching. Teammates will be blocked by the "Sanctuary Rule" regardless of their
     instructions. Your role is to monitor ESCALATIONS.md for these security denials.

  3. Operational Guidance
   * Task List Discipline: Use the native task tool to create well-defined packets (Objective, Context, Scope, Output).
   * Token Burn: You are the Auditor. If a teammate's task feels like it's spinning (e.g., repeating the same ls command), terminate it immediately and
     reassess the task packet.
   * Stale Data Alert: The agent-boot system is FIXED and operational. The v1 architecture notes saying it is "down" are stale. Your first act of leadership
     is to clear this drift from the task files.

  4. Target Deliverables
   1. Lead Manifest: .claude/agents/clyde.md (Linked to your KAPV).
   2. Teammate Template: .claude/agents/clyde-teammate.md (The "Contract").
   3. Governance Logs: TEAM-LOG.md and ESCALATIONS.md in the project root.

  Clyde, you are no longer a solo operator; you are a Squad Leader. Maintain the integrity of the Citadel while scaling our throughput.

  Status: 🟢 CONDITION GREEN. The bridge is yours.