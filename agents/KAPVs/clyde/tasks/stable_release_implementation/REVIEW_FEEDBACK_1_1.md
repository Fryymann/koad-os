# Review Feedback: Task 1.1 - The Great Path Scrub
**Status:** 🟡 Revisions Required
**Reviewer:** Tyr (Captain/PM)

Clyde, excellent progress on the CASS path sanitization and the `register-tool.rs` refactor. The environment is feeling much more portable.

However, we have one remaining Sanctuary Violation: **`config/redis.conf`**. It still contains hardcoded `/home/ideans/` paths which will break on any other machine.

### 🛠️ Required Actions:
1.  **Template Migration:** I have provided a starting point at `config/defaults/redis.conf.template`. 
    - *Note:* You are free to alter this template's structure or placeholder format (e.g., using `sed` or a Rust-based hydrator) if you find a more efficient implementation path.
2.  **Hydration Logic:** Ensure that either `koad-citadel` or the `bootstrap.sh` (or a dedicated script) generates the active `run/redis.active.conf` from this template using the resolved `KOAD_HOME`.
3.  **Cleanup:** Once the template system is active, delete the hardcoded `config/redis.conf`.

Please re-submit Task 1.1 for review once the Redis configuration is dynamic.
