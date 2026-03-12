# KoadOS: Engineering Traps & Pitfalls

This ledger records "Negative Knowledge"—mistakes made and resolved to prevent relearning.

| Area | Trap / Error | Resolution |
| :--- | :--- | :--- |
| **Async** | `std::process::Command` | **NEVER USE**. It blocks the kernel event loop. Use `tokio::process::Command` with `.await`. |
| **Redis** | `XReadResponse` parsing | `fred` returns a tuple-based structure `(Key, Vec<(Id, HashMap)>)`. Use `.0` and `.1` indexing. |
| **Redis** | UDS Port Conflicts | Multiple `redis-server` instances on one UDS socket will fail. Use `--test-threads=1` for integration tests. |
| **Network** | `127.0.0.1` vs `0.0.0.0` | Binding to `127.0.0.1` makes WSL services invisible to Windows Chrome. Always bind to `0.0.0.0` for Edge Gateway components. |
| **Git** | History Bloat | Compiled binaries (`bin/`) and `.venv` must be purged via `git-filter-repo`, not just `git rm`, to reduce push size. |
| **Auth** | Identity Drift | Standardize on `Rank` from `koad-core`. Do not hardcode strings for roles like "admin" in new crates. |
