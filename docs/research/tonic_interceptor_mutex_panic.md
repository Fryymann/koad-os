# Technical Note: Tonic Interceptor & Tokio Mutex Panic

**Date:** 2026-04-12
**Status:** Unresolved / Blocked
**Component:** `koad-citadel` -> `auth/interceptor.rs`

## The Issue
During the boot sequence (`koad-agent boot`), the Citadel gRPC connection fails with an `h2 protocol error`. The root cause is a panic in the `tokio-runtime-worker`:
`Cannot block the current thread from within a runtime. This happens because a function attempted to block the current thread while the thread is being used to drive asynchronous tasks.`

## Root Cause Analysis
1. In `koad-agent.rs`, we correctly added the mandatory Zero-Trust headers (`x-actor`, `x-session-id`, `x-session-token`) to the `CreateLease` request.
2. The request hits the Citadel, and Tonic routes it through `build_citadel_interceptor()` (`crates/koad-citadel/src/auth/interceptor.rs`).
3. Tonic interceptors are **synchronous**. They return a closure: `impl Fn(Request<()>) -> Result<Request<()>, Status>`.
4. Inside this synchronous closure, the interceptor attempts to read the session cache: `let sessions_guard = sessions.blocking_lock();`.
5. `sessions` is of type `ActiveSessions` (`Arc<tokio::sync::Mutex<HashMap<...>>>`).
6. Calling `blocking_lock()` on a `tokio::sync::Mutex` from within an active Tokio runtime thread causes an immediate panic.

## Proposed Solutions (For Tomorrow's Planning)

**Option A: Switch `ActiveSessions` to `std::sync::Mutex` or `parking_lot::Mutex`**
- *Pros:* Solves the panic immediately. Since the lock is only held briefly to check the HashMap and never across `.await` points, a synchronous mutex is actually the recommended pattern in Tokio for this use case.
- *Cons:* Requires updating `session.rs` and `session_cache.rs` where `.lock().await` is currently used.

**Option B: Remove Interceptor and Validate in Service Methods**
- *Pros:* Keeps `tokio::sync::Mutex`. Validation happens inside the `async fn` of the gRPC service methods where `.lock().await` can be safely called.
- *Cons:* Duplicates validation logic across every gRPC method (`create_lease`, `heartbeat`, `close_session`, etc.), violating DRY and risking security bypasses if a developer forgets to add the check to a new method.

**Option C: Use `tonic`'s `Extension` or `async` Middleware (tower::Service)**
- *Pros:* Cleanest architectural approach. Uses `tower`'s async middleware to perform the validation before it hits the gRPC service, allowing `.await` on the Mutex.
- *Cons:* More complex to implement than a standard Tonic interceptor.

## Recommendation
Pause, evaluate Option A vs Option C during the next architectural sprint, and implement the chosen path cleanly.
