# Domain: System Scripts (scripts/)
**Role:** Initialization, Maintenance & Diagnostics

This directory contains the utility scripts used for bootstrapping the environment, initializing databases, and performing system-level diagnostics.

## Ⅰ. Script Index

- `init-koad-db.sh`: Initializes the primary `koad.db` SQLite schema.
- `init-jupiter-db.sql`: Bootstraps the Jupiter-specific episodic and procedural memory tables.
- `init-map-db.sql`: Defines the schema for the navigation map (`pins` and `history`).
- `install-services.sh`: Installs and enables the `koad-citadel` and `koad-cass` systemd units.
- `koad-telemetry.sh`: Handles agent session boot/shutdown telemetry reporting.
- `koad-notion-doctor.py`: Automated diagnostic tool for Notion integration health.
- `koad-xp-audit.py`: Performs consistency checks on agent XP ledgers.

## Ⅱ. Configuration
- `procedural_seeds.json`: Seed data for initial procedural memory population.
