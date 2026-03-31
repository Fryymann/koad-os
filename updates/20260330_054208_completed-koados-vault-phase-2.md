+++
id        = "upd_20260330_054208_completed-koados-vault-phase-2"
timestamp = "2026-03-30T05:42:08.932258285+00:00"
author    = "tyr"
level     = "citadel"
category  = "infra"
summary   = "COMPLETED: KoadOS Vault Phase 2 — Portable Identity Layer"
+++

Implemented KOAD_VAULT_URI resolution and 'koad vault' CLI. Vault identity is now decoupled from absolute filesystem paths. Added: whoami, env, get, ls, and secret subcommands. Updated koad-agent boot to export vault URI.
