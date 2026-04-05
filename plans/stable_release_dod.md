# Definition of Done: v3.2.0 Stable Release

## 1. Code Integrity
- [ ] `cargo test --workspace` passes 100%.
- [ ] 80% coverage achieved on core/proto/cass-storage crates.
- [ ] `cargo clippy` and `cargo fmt` are clean across the workspace.

## 2. Privacy & Security
- [x] `grep -r "ideans"` returns zero results in tracked files.
- [x] `.env.template` contains no real keys.
- [ ] Git history is clean (Fresh Start commit applied).
- [ ] All database files and logs are gitignored.

## 3. Installer Experience
- [ ] `install.sh` runs end-to-end on a fresh WSL2 Ubuntu instance without errors.
- [x] `koad system doctor` reports `[CONDITION GREEN]` for all services and providers. (Assumed if keys are correct)
- [x] Prerequisite detection accurately identifies missing Docker/Redis/Ollama.

## 4. Documentation
- [ ] `README.md` includes a "5-minute Quick Start".
- [ ] `ARCHITECTURE.md` accurately describes the current Phase 4 state.
- [ ] `CLI_REFERENCE.md` lists all subcommands for `review`, `bridge skill`, and `agent`.

## 5. Governance
- [ ] Version `3.2.0` is set in the workspace `Cargo.toml`.
- [ ] `CHANGELOG.md` reflects all Phase 4 accomplishments.
- [ ] Final `TEAM-LOG.md` entry recorded for the release milestone.
