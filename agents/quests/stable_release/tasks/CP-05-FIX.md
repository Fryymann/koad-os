# Task Spec: CP-05-FIX (Resolve Autonomic Recovery Compile Errors)

**Mission:** KoadOS v3.2.0 "Citadel Integrity"
**Agent:** Clyde (Implementation Lead)
**Status:** TODO
**Priority:** Blocker (Nightly Build Failing)

## 🎯 Objective
Resolve compile errors introduced during the implementation of the "Autonomic Recovery" feature in `crates/koad-cli/src/handlers/status.rs`. The crate currently fails to compile due to `fred` stream `Unpin` trait requirements, invalid `RedisMap` iteration, and missing type inferences.

## 🧱 Scope & Impact
- **Affected Crates:** `crates/koad-cli` (specifically `src/handlers/status.rs`).
- **Impact:** Fixing these compile errors will unblock the `koad` CLI build, allowing the CI/CD pipeline for the stable release to proceed.

## 🛠️ Implementation Steps for Clyde

### 1. Fix Stream Unpin Error
- In `crates/koad-cli/src/handlers/status.rs` (around line 58), the `hscan` stream must be pinned before it can be awaited.
- **Action:** Wrap the `hscan` call in `Box::pin`:
  ```rust
  let mut scan_stream = Box::pin(client.pool.next().hscan(REDIS_KEY_STATE, "koad:session:*", None));
  ```

### 2. Fix RedisMap Iteration and Type Inference
- In the same block, `fields` (which is a `RedisMap` returned by `page.results()`) cannot be iterated over directly using `for (key, val) in fields`. Furthermore, `key.to_string()` and `val.as_str()` fail because their types aren't inferred correctly from the generic map entry.
- **Action:** Convert the `RedisMap` to an iterator and explicitly handle the `fred::types::RedisKey` and `fred::types::RedisValue` types (usually via `.into_iter()` and `.as_str()`).
  ```rust
  if let Some(fields) = page.results() {
      for (key, val) in fields.into_iter() {
          if let (Some(k), Some(v)) = (key.as_str(), val.as_str()) {
              if let Ok(session) = serde_json::from_str::<serde_json::Value>(v) {
                  let heartbeat = session["heartbeat"].as_u64().unwrap_or(0);
                  let timeout = session["deadman_timeout"].as_u64().unwrap_or(45);
                  if heartbeat > 0 && now - heartbeat > timeout + 30 {
                      stale_sessions.push(k.to_string());
                  }
              }
          }
      }
  }
  ```

## ✅ Verification Strategy
-   Run `cargo check -p koad` from the root of the `.koad-os` directory.
-   Ensure the crate compiles with zero errors and no new warnings.
