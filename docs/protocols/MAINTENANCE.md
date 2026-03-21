# KoadOS Maintenance Protocol

This document outlines the procedures for maintaining and updating the KoadOS environment and its associated shell utilities.

## Ⅰ. PimpedBash Maintenance
PimpedBash provides a unified shell environment for KoadOS agents and human contributors. It should be kept up-to-date to ensure compatibility with new Citadel features.

### 1. Update Procedure (`pimped-update`)
Run `pimped-update` to:
- Pull the latest changes from the PimpedBash repository.
- Clean up temporary and stale artifacts from the workspace.
- Refresh shell symlinks to ensure they point to the correct configuration files.
- Re-source the current environment.

### 2. Uninstall Procedure (`pimped-uninstall`)
Run `pimped-uninstall` to:
- Remove all PimpedBash symlinks from the home directory.
- Restore original `.bashrc` and other configuration backups.
- (Optional) Remove the `.pimpedbash` repository directory.

## Ⅱ. Station Management
Stations are project hubs (e.g. `SLE`, `DND`) that can have their own secrets and configuration overrides.

### 1. Creating a New Station
To define a new station:
1. Create a directory for the station hub.
2. Add a `.agent-station` file in that directory containing the station name (e.g. `DND`).
3. Add a `[projects.<name>]` entry in `registry.toml` with `station = "DND"`.
4. Add any station-specific keys to your `.env` using the `KOADOS_STATION_DND_<KEY>` pattern.

### 2. Creating a New Outpost
An Outpost is a specific project that may need unique overrides.
1. Create a `.agent-outpost` file in the project root containing the outpost name (e.g. `KOS`).
2. Add any outpost-specific keys to your `.env` using the `KOADOS_OUTPOST_KOS_<KEY>` pattern.
3. KoadOS will prioritize these over Station and Main defaults.
The `registry.toml` file is the canonical map of the KoadOS workspace. Contributors must ensure that any new projects or outposts are registered with the correct `path`, `station`, and `level`.
