# Body/Ghost Boot Sequence (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #160

## 1. Requirement
Define standard boot sequence and registration logic.

## 2. Boot Flow (Sequence)
1. **Invocation:** `koad agent prepare --agent <name> --project <root>`.
2. **Identification:** Locate `config/identities/<name>.toml` and verify the agent's identity hash.
3. **Lease (Handshake):** gRPC request to the Citadel for a new session lease (`TRC-GEN-001`).
4. **Hydration (CASS):** Fetch relevant context (Tier 2/3) from Qdrant/SQLite.
5. **Registration:** Automatic registration of the session with the Citadel registry (`koad:session:<id>`).
6. **Payload:** Generate `eval` payload (`export KOAD_SESSION_ID=...; export ...`).

## 3. Context Placement
- **Project Root:** `GEMINI.md`, `CLAUDE.md`, etc. (Agent-owned).
- **KoadOS Managed:** `~/.koad-os/registry/` (System-owned).

## 4. Zero-Trust Verification
- **Requirement:** Every boot MUST request a new lease or verify the existing one with the Citadel.
- **Constraint:** Rejection if identity or lease is expired or invalid.