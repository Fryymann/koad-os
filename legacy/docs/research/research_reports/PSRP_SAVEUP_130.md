## PSRP Saveup — Issue #130 — 2026-03-10

### 1. Fact (What happened?)
I implemented the backend for the Agent-to-Agent "Signal" Protocol (A2A-S). I added the `GhostSignal` schema to `koad-core`, defined the gRPC contract in `spine.proto` (`SendSignal`, `GetSignals`, `UpdateSignalStatus`), and built the `SignalManager` within `koad-spine` to persist messages in Redis under `koad:mailbox:<agent_name>`. I encountered and resolved compiler errors related to the Rust borrow checker (moving `request` before accessing metadata) and `prost` generated enum variant names.

### 2. Learn (Why did it happen / What is the underlying truth?)
I learned that `prost` strips enum prefixes by default. For example, `SIGNAL_PRIORITY_LOW` in the `.proto` file becomes `SignalPriority::Low` in Rust, not `SignalPriority::SignalPriorityLow`. Assuming the prefix would be retained caused a compilation failure during the KSRP Lint pass. I also learned that `tonic::Request::into_inner()` consumes the request, so any necessary metadata must be extracted or cloned *before* the inner payload is accessed.

### 3. Ponder (How does this shape future action?)
This asynchronous messaging foundation is a significant step toward swarm operations. By storing signals in a Redis Hash (`koad:mailbox:<agent_name>`), agents don't need to be "Wake" to receive instructions; they will see `pending_signals` populated in their `IntelligencePackage` the next time they boot. The final step to complete Issue #130 will be implementing the CLI tools (`koad signal send`, `list`, `read`) so agents have a tactile way to interact with their mailboxes. I am confident this design respects the "One Body, One Ghost" protocol while enabling tight collaboration.
