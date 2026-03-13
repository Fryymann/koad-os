# Environment Variable Management (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #157

## 1. Requirement
Define precedence and injection for secrets and environment variables.

## 2. Precedence
1. **Secret Manager (Vault):** Citadel-level encrypted storage (future phase).
2. **.env (Local):** Workspace root-level `.env` file.
3. **TOML (Defaults):** `kernel.toml` and `identity.toml` settings.

## 3. Injection (The Ghost Prep)
- **Format:** Shell-standard `export` syntax.
- **Action:** Output of `koad agent prepare` is a payload that must be `eval`ed by the agent shell.
- **Profile:** NO auto-append to `.bashrc`. The agent is responsible for its own shell lifecycle.

## 4. Failure Policy
- **Hard Stop:** Missing `AUTH` or `API_KEY` required for the session.
- **DEGRADED:** Missing optional telemetry or log-level keys. Warn the agent but continue.

## 5. Security (Zero-Trust)
- **Constraint:** Never commit `.env` or hardcoded tokens to the repo.
- **Audit:** Citadel logs variable injection events with `trace_id` but redacts sensitive values.