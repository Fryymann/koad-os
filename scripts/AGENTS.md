# scripts/

Installation, service setup, maintenance and diagnostics for KoadOS. The canonical installer is
`../install.sh` at the repo root.

| Script | Purpose |
|---|---|
| `install-skills.sh` | Install the KoadOS agent skills with the Agent Skills CLI (from `nightly` on GitHub) |
| `install-services.sh` | Install and enable the `koad-citadel` and `koad-cass` systemd units |
| `verify-services.sh` | Pre-flight Qdrant readiness check used by `koad-cass.service` |
| `koad-functions.sh` | Shell functions (`agent-boot`) for interactive terminals |
| `uninstall.sh` | Remove a KoadOS install |
| `koad-sanitize.sh` | Distribution scrub: removes local state, logs and databases (destructive) |
| `koad-review.sh` | Prototype code reviewer |
| `koad-telemetry.sh` | Session boot/shutdown telemetry emitter |
| `koad-notion-doctor.py` | Notion integration health check |
| `sync-status.sh` | Summarise `updates/` into current phase status |
| `init-koad-db.sh`, `init-jupiter-db.sql`, `init-map-db.sql`, `procedural_seeds.json` | Database schemas and seed data |
| `install.sh` | Legacy standalone installer; nothing references it, use `../install.sh` |

Scripts that stop or remove services or data (`uninstall.sh`, `koad-sanitize.sh`) need Ian's
approval before running on Jupiter.
