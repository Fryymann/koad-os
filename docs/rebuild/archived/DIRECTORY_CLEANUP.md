# Directory Cleanup & Harmonization (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #156

## 1. Requirement
Resolve discrepancy between `personas/` (Flight Manual) and `config/identities/` (Codebase).

## 2. Decision
- **Winner:** `config/identities/` is the canonical source.
- **Retirement:** `personas/` is retired to `legacy/`.

## 3. Action Plan
1. Move all existing files in `personas/` to `legacy/personas/`.
2. Update all documentation references (Notion, flight manual, `GEMINI.md`) to use `config/identities/`.
3. Ensure `kernel.toml [registry]` points to `config/identities/` as the identity source.

## 4. Verification
- `grep -r "personas/" .koad-os/` returns zero results in the active workspace.
- `koad agent prepare` correctly locates identities in `config/identities/`.