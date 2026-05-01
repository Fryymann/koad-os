# Incident Report: Service Offline (Path Mismatch & Config Deserialization)
**Date:** 2026-04-25
**Agent:** tyr (Principal Engineer)

## 🚨 Symptom
Citadel and CASS were reported as `[OFFLINE]` during the agent boot sequence. Port scans (`ss`/`netstat`) showed no listeners on `50051` or `50052`, despite `systemctl` reporting services as "active" or "activating".

## 🔍 Root Cause Analysis
1. **Path Decay:** The systemd unit files (`koad-citadel.service`, `koad-cass.service`) were pointing to the deprecated `~/.koad-os` directory. The active deployment has migrated to `~/.citadel-jupiter`.
2. **Partial Configuration Failure:** When the binaries were pointed to the correct home directory, they failed to deserialize `KoadConfig`.
   - **Reason:** The `KoadConfig::load()` method globs all TOML files in `config/identities/`.
   - **Conflict:** Identity files (e.g., `tyr.toml`, `clyde.toml`) only contain agent-specific fields. The `KoadConfig` struct required `system`, `network`, and `storage` blocks in *every* merged file, causing a "missing field `system`" error during the merge.

## 🛠️ Resolution
### 1. Systemd Correction
Updated unit files in `/etc/systemd/system/` to reflect the new deployment root:
- `WorkingDirectory=/home/ideans/.citadel-jupiter`
- `EnvironmentFile=/home/ideans/.citadel-jupiter/.env`
- `ExecStart=/home/ideans/.citadel-jupiter/bin/koad-citadel` (and `koad-cass`)

### 2. Codebase Fix (Koad-Core)
Modified `crates/koad-core/src/config.rs` to allow partial configuration merging. Added `#[serde(default = "...")]` attributes to the primary configuration blocks:

```rust
pub struct KoadConfig {
    #[serde(default = "default_system")]
    pub system: SystemConfig,
    #[serde(default = "default_network")]
    pub network: NetworkConfig,
    #[serde(default = "default_storage")]
    pub storage: StorageConfig,
    // ...
}
```

This ensures that when an identity file is merged, missing global system fields fall back to established defaults rather than failing the entire boot process.

## 📈 Long-term Recommendation
- **CLI Migration Tool:** Implement a `koad system migrate-paths` command to automate unit file updates when `KOADOS_HOME` changes.
- **Config Validation:** Add a validation step after merging that logs *which* file caused a deserialization error to prevent silent failures in the glob loop.
