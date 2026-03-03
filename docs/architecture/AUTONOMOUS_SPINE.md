# KoadOS Autonomous Ecosystem (v4.3)

## 1. AI Crew Roster
The roster defines the permanent personnel of KoadOS. Each member has a type that determines the system's reaction to their state.

### 1.1 Roster Members
| Identity | Role | Type | Min Tier |
| :--- | :--- | :--- | :--- |
| **Koad** | Captain/Admin | Essential | 1 |
| **Vigil** | Auditor/PM | Support | 2 |
| **Pippin** | PM/Researcher | Support | 2 |
| **Gopher** | Discovery/Search | Support | 3 |

### 1.2 Health Response
- **Essential:** Spine emits `CONDITION_RED` alert if WAKE status is lost without a check-in.
- **Support:** Spine logs a `SERVICE_DEGRADED` warning.

## 2. Autonomic Watchdogs (Self-Healing)
The `ShipDiagnostics` loop is expanded to perform proactive recovery.

### 2.1 Neural Bus Sentinel
- **Detection:** Spine checks `redis` connectivity every 5s.
- **Healing:** If connection is lost and `koad.sock` exists, Spine assumes a Ghost Socket, purges it, and restarts the managed Redis instance.

### 2.2 Ghost Process Sentinel
- **Detection:** Spine scans for orphaned `kspine` or `kgateway` processes using `sysinfo`.
- **Healing:** Automatic `kill` signal sent to non-owned processes that interfere with system sockets.

## 3. Communication Templates (Issue #60)
Standardized payloads for system events.

### 3.1 Incident Report Template
```json
{
  "incident_id": "UUID",
  "source": "Sentinel:[Name]",
  "severity": "CRITICAL|WARN|INFO",
  "root_cause": "String",
  "recovery_attempted": true,
  "status": "RECOVERED|PENDING|FAILED"
}
```

### 3.2 Crew Update Template
```json
{
  "identity": "String",
  "event": "ARRIVED|DEPARTED|IDLE",
  "session_id": "String"
}
```
