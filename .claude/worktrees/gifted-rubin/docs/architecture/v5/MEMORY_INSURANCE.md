# v5.0 Memory Insurance Protocol (The "Gold" Standard)

> [!CAUTION]
> **Primary Mandate:** Knowledge is the station's most valuable asset. Its loss is a Category 1 System Failure. v5.0 implements **Triple-Redundancy** and **Sovereign Protection** for all agent learnings.

---

## 1. Memory Sovereignty (Anti-Deletion)
The system is designed with **Strict Prevention of Deletion**. Knowledge is considered "Station History" and cannot be nuked by standard agent operations.

### **A. Soft-Delete by Default**
- The `knowledge` table in SQLite implements a `is_deleted` flag. 
- The `DELETE` SQL command is programmatically blocked at the `KoadDB` layer for all Agent-rank entities.
- Any attempt to "prune" results in a soft-delete, preserving the raw data in the `audit_trail` for recovery.

### **B. WORM Ledger (Write Once, Read Many)**
- The **Ledger Mirror (`ledger.jsonl`)** is an append-only file. 
- The station process has no "Overwrite" or "Truncate" permissions on this file. It can only `APPEND`.
- This ensures that even a corrupted Spine cannot "erase" history; it can only add new records.

### **C. Admiral Override**
- Only the **Admiral (Ian)** using a specific tactical command (`koad dood purge --force`) can physically remove records from the durable store. This requires an interactive "Confirmation Sequence" in the TUI.

---

## 2. Triple-Redundancy Architecture
Every time an agent or Admiral uses `koad intel remember`, the data is broadcast to three independent sinks.

| Sink | Type | Purpose |
| :--- | :--- | :--- |
| **1. SQLite (`koad.db`)** | Binary | Fast, structured retrieval for agent context hydration. |
| **2. Ledger Mirror (`ledger.jsonl`)** | Text (JSONL) | An append-only, human-readable safety net. If the DB is lost, we rebuild from here. |
| **3. Cloud Sink (Notion)** | Cloud API | Remote off-site backup. Total protection against local hardware or environment failure. |

## 2. The "Brain Drain" Verification
The v5.0 **Teardown State** cannot proceed until the Spine receives an `ACK` from all three sinks.
- If Notion is offline, the learning is queued in a Redis "Sync-Later" list.
- If SQLite is locked, the agent process hangs until the lock clears or it escalates a **Distress Signal** to the Admiral.

## 3. Automatic Snapshotting (The "Vault")
The **Signal Corps** manages the Vault task:
- **Interval:** Every 10 new commits or every 4 hours of station uptime.
- **Action:** `VACUUM INTO` a timestamped backup file in `.koad-os/backups/vault/`.
- **Retention:** Keep the last 30 snapshots.

## 4. The Reconstruction Tool: `koad system restore`
A new professional-level recovery tool.
- **Mode A (Rebuild from Ledger):** Reads the JSONL file and recreates the SQLite `knowledge` table.
- **Mode B (Pull from Cloud):** Fetches all records from the Notion Knowledge Base and hydrates the local station.
- **Mode C (Snapshot Rollback):** Swaps the current `koad.db` with a known-good vault snapshot.

## 5. Summary: Data Integrity Gates
- **Phase 1 (Engine Room):** Includes the JSONL Ledger implementation.
- **Phase 2 (Spine):** Includes the `restore` tool and `VACUUM` loop.
- **Phase 3 (Signal Corps):** Includes the Notion mirroring task.

---
*The Gold is Secured. Command by Captain Tyr.*
