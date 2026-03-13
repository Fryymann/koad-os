# Data Migration Mapping (v5.0)
**Status:** DRAFT (Phase 1)
**Issue:** #154

## 1. Requirement
Define extraction for `koad.db` and `koad:*` Redis keys before archiving.

## 2. Mapping Table

| **Legacy Key/Table** | **v5.0 Target** | **Type** | **Notes** |
| --- | --- | --- | --- |
| `koados_knowledge` (SQLite) | CASS L3 (Qdrant) | Knowledge | Vectorized upon migration. |
| `episodic_memories` (SQLite) | CASS L2 (Episodic) | Memory | Preserves all timestamps. |
| `task_outcomes` (SQLite) | CASS L2 (Episodic) | Memory | Maps to new EoW schema. |
| `koad:state:* ` (Redis) | Citadel Shared State | State | Only active leases/locks migrated. |
| `koad:mailbox:* ` (Redis) | A2A-S (Personal Bay) | Mailbox | Migrated to new Redis stream format. |
| `koad:kai:* ` (Redis) | Identity Registry (SQLite) | Identity | Standardized to TOML-based registry. |

## 3. Tool Spec (`koad system migrate-v5`)
- **Source:** Local `~/.koad-os/koad.db` and local Redis instance.
- **Target:** v5-compatible SQLite WAL (`L2`) and Qdrant collections (`L3`).
- **Safety:** Hard-stop on secret/token detection in legacy data (`Zero-Trust`).
- **Continuity:** Must map old author tags to new v5 Trace ID format.