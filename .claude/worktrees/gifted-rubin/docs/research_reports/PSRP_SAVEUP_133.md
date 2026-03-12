## PSRP Saveup — Issue #133 — 2026-03-10

### 1. Fact (What happened?)
I implemented a structural change to the KoadOS Spine and the user's environment to resolve a driver pre-emption conflict. Previously, the `ASM` (Active Session Manager) assumed that one driver (e.g., `gemini`) equaled one body, leading agents using the same underlying driver to constantly kill each other's sessions ("Ghost Pre-emption"). I refactored the `prune_body_ghosts` function to include an explicit `agent_name` check and implemented a bash-level session generation script (`koad-session`) that injects a unique `KOAD_SESSION_ID` into every new shell instance.

### 2. Learn (Why did it happen / What is the underlying truth?)
The "One Body, One Ghost" protocol was fundamentally sound but implemented at the wrong layer of abstraction. By binding the "Body" identity to the AI Driver rather than the terminal shell, KoadOS inadvertently prevented parallel agent operations. I learned that in a multi-agent CLI environment, the terminal session *is* the physical manifestation (the Body) of the agent. Treating the shell as the primary container ensures complete isolation, predictable heartbeats, and allows KoadOS to scale to concurrent operations seamlessly.

### 3. Ponder (How does this shape future action?)
This shift in understanding opens up the possibility for true swarming. If a single shell is a unique body, we could theoretically spawn background, headless shells (sub-bodies) to execute tasks concurrently. The Spine is now capable of managing N-number of concurrent sessions for N-number of agents, restricted only by available hardware and token limits. Moving forward, any state management or caching must pivot primarily on `KOAD_SESSION_ID` rather than assuming global access based on driver type.
