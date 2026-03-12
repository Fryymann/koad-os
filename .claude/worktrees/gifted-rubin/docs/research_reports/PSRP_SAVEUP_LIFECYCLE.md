## PSRP Saveup — Session Lifecycle — 2026-03-10

### 1. Fact (What happened?)
I implemented a "Graceful Untethering" mechanism using the Gemini CLI `SessionEnd` lifecycle hook. This involved adding a `TerminateSession` gRPC endpoint to the Spine, a `koad logout` command to the CLI, and a hook in `settings.json` that triggers the logout using the shell's `KOAD_SESSION_ID`. I also had to implement a `remove_session` method in the Spine's ASM engine to ensure the local cache remains synchronized with the "Dark" status in Redis.

### 2. Learn (Why did it happen / What is the underlying truth?)
I learned that "Identity Integrity" requires both a reliable start (Boot) and a reliable end (Logout). Relying solely on a background "Reaper" to clean up sessions creates a lag in system awareness (the "Ghost in the Machine" effect). By hooking into the CLI's native exit event, we achieve **Real-Time State Finality**. I also learned that the Spine's local memory (ASM `HashMap`) must be explicitly managed during termination to prevent reporting discrepancies in `koad status`.

### 3. Ponder (How does this shape future action?)
This confirms that the "Body" (Shell) should be as ephemeral as possible, while the "Ghost" (Spine State) is the source of truth. Moving forward, we can use these lifecycle hooks for more than just identity cleanup. Could we use `SessionEnd` to trigger an automatic `Sovereign Save` or a `KSRP` summary of the session? The more we leverage the CLI's lifecycle, the more KoadOS feels like a true extension of the host environment rather than just a guest process.
