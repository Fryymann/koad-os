## PSRP Saveup — Issue #130 (CLI) — 2026-03-10

### 1. Fact (What happened?)
I implemented the CLI interface for the Agent-to-Agent "Signal" Protocol (A2A-S). This involved adding the `Signal` command to `koad-cli`, defining subcommands for `send`, `list`, `read`, and `archive`, and building the corresponding gRPC client logic. I verified the implementation with a "Live Fire" test, ensuring Tyr can send signals to Sky (and himself) across shell boundaries. I also resolved an authentication issue by correctly injecting `x-session-id` into the gRPC metadata.

### 2. Learn (Why did it happen / What is the underlying truth?)
I learned that when building a CLI-to-Spine interface, the `KOAD_SESSION_ID` must be manually propagated into the gRPC metadata for every request that requires agent identity. Unlike web browsers with cookies, the CLI environment requires explicit state injection to maintain the "Neural Link." I also learned that a "hard reset" (`system refresh --restart`) is mandatory after updating gRPC handlers to ensure the running Spine process matches the new compiled contract.

### 3. Ponder (How does this shape future action?)
With A2A-S online, we have solved the "Isolation Paradox." Agents can now be completely isolated in their shells (Bodies) while remaining tightly coordinated in their mission (The Grid). This pattern should be extended: could we use signals to "hand off" an entire sub-agent task? Or to trigger an "Inter-Agent Peer Review" where Sky signals Tyr to audit a specific file? The mailbox is the first piece of social infrastructure for KoadOS agents.
