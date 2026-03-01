# KoadOS Spine: System Kernel Design

The **Spine** is the central nervous system of KoadOS. It is responsible for process orchestration, state synchronization, and providing a unified IPC interface for all Koad components (CLI, TUI, Dashboard, and Skills).

## 1. Core Objectives
- **Reliable Orchestration**: Asynchronous task execution with persistent logging and retry logic.
- **State Consistency**: Unified view of system state across Redis (volatile/hot) and SQLite (persistent/cold).
- **Unified IPC**: A single gRPC/UDS gateway for all external component interactions.
- **Environment Integrity**: Strict path resolution and environment inheritance for child processes.

## 2. Architectural Layers

### A. The Engine (Core Logic)
- **Task Scheduler**: Manages the lifecycle of background tasks (Pending -> Running -> Success/Failure).
- **Resource Manager**: Tracks active projects, files, and system resources.
- **Environment Controller**: Injects `filesystem.mappings` and PATH variables into all spawned processes to prevent `os error 2`.

### B. The Memory Bridge (State Management)
- **Hot Path (Redis)**: Pub/Sub for real-time events, task heartbeats, and live telemetry.
- **Cold Path (SQLite)**: WAL-mode persistence for facts, learnings, and execution history.
- **Sync Service**: A "Write-Behind" worker that drains Redis state into SQLite to ensure long-term durability without blocking the hot path.

### D. The Service Gateway (Connectivity)
- **Binding Controller**: Manages TCP/UDS listeners. Enforces `0.0.0.0` for WSL-to-Windows visibility while providing a fallback to `127.0.0.1` for local-only security.
- **Port Manager**: Prevents "Address already in use" errors by tracking PIDs and performing "Force-Clear" operations on hung ports.
- **URL Resolver**: Broadcasts the active WSL-to-Windows bridge IP so Chrome on Windows 11 can always find the Dashboard.

## 3. Development Flow & Standards
...
### Phase 1: Schema Stabilization
- Define all internal messages in `proto/spine.proto`.
- Standardize the `Task` and `Event` JSON structures used in Redis.
- **Service Registration**: Define the `ServiceEntry` schema for the Gateway.


### Phase 2: Diagnostic Visibility
- Implement a "Trace Mode" where every internal state change is broadcast to a dedicated Redis channel.
- Build `koad dood spine-logs` to tail the kernel's raw internal heartbeat.

### Phase 3: Robust Execution
- Implement a "Sandbox" wrapper for shell commands that explicitly validates the existence of the working directory and binary before execution.
- Fix the `systemd` environment inheritance bug by loading a standard `.env` or `koad.json` context into the Spine's global state.

## 5. Redis Hot-Path Schema (The "Backbone" State)

To ensure consistency across WSL and Windows, the Spine uses a structured Redis key space.

### A. Task State (`koad:task:{id}`)
- **Type**: JSON Hash
- **Structure**:
  ```json
  {
    "task_id": "uuid",
    "command": "string",
    "args": ["string"],
    "working_dir": "string",
    "environment": "WSL | WINDOWS",
    "status": "PENDING | RUNNING | SUCCESS | FAILED",
    "exit_code": "int | null",
    "stdout": "string",
    "stderr": "string",
    "updated_at": "iso-8601"
  }
  ```

### B. Active Task Index (`koad:tasks:active`)
- **Type**: Set
- **Members**: `task_id` strings for all non-terminal tasks.

### C. System Event Stream (`koad:events:stream`)
- **Type**: Redis Stream (XADD)
- **Fields**:
  - `source`: string (e.g., "engine", "windows-cli")
  - `severity`: "INFO", "WARN", "ERROR"
  - `message`: string
  - `metadata`: JSON string
  - `timestamp`: unix-timestamp

### E. Service Inventory (`koad:services`)
- **Type**: Hash
- **Fields**: `name` (e.g., "web-deck")
- **Value**: JSON `ServiceEntry` (host, port, status, last_seen)
- **Purpose**: Unified lookup for cross-env connectivity.

## 6. Standardized Event Payloads (metadata_json)

To maintain a flexible but predictable event bus, all `SystemEvent` payloads must follow these sub-schemas within the `metadata_json` field.

### A. `FILE_WATCHER` (Source: "engine:discovery")
```json
{
  "path": "/absolute/path/to/file",
  "action": "CREATE | MODIFY | DELETE",
  "is_dir": false,
  "extension": "rs"
}
```

### B. `TASK_LIFECYCLE` (Source: "engine:scheduler")
```json
{
  "task_id": "uuid",
  "status": "RUNNING | SUCCESS | FAILED",
  "exit_code": 0,
  "elapsed_ms": 1250
}
```

### C. `RESOURCE_DISCOVERY` (Source: "engine:discovery")
```json
{
  "resource_type": "PROJECT | SKILL | AGENT",
  "name": "string",
  "path": "string",
  "action": "REGISTERED | RETIRED"
}
```

### D. `SYSTEM_HEARTBEAT` (Source: "engine")
```json
{
  "uptime_secs": 3600,
  "active_tasks": 2,
  "memory_usage_mb": 45,
  "environment_drift": false
}
```
