# v5.0 Performance Metrics & Friction Analytics

> [!IMPORTANT]
> **Primary Mandate:** "Identify the Grind."
> We don't just track uptime; we track the efficiency of the neural grid. Any component or link that introduces latency is flagged as **System Friction**.

---

## 1. The Friction Map (E2E Latency)
We track the "Neural Round Trip" (NRT) for every Admiral or Agent intent. 

| Metric | Target | Friction Trigger |
| :--- | :--- | :--- |
| **Spine Hop** (CLI -> gRPC -> Spine) | < 5ms | > 20ms |
| **Engine Link** (Spine -> Redis -> Spine) | < 2ms | > 10ms |
| **Persistence Sink** (Spine -> SQLite) | < 50ms | > 200ms |
| **Agent Thought** (Chassis -> LLM -> Chassis) | Model Dependent | 2x Baseline |
| **Tool Execution** (Local Shell/IO) | Task Dependent | > 1.5x Historical Avg |

---

## 2. Friction Probes (Surgical Monitoring)

### **A. The Redis Lua Probe**
Since Redis is our authority, the Spine periodically executes a non-blocking `EVAL` script that measures the time taken to acquire and release an atomic lease.
- **Goal:** Identify contention between Tyr and Sky before it leads to a deadlock.

### **B. The "Token Bloat" Monitor**
Monitors the size of the `AgentContext` being transmitted via gRPC.
- **Metric:** `Context_Density = (Tokens / Character_Count)`.
- **Friction Alert:** If context density drops (too much metadata), the **Context Governor** triggers a "Compaction Required" signal.

### **C. The I/O Ghost Tracker**
Tracks file system latency within **Git Worktrees**.
- **Metric:** Time to create/delete a worktree.
- **Friction Alert:** If `git worktree add` takes > 2s, it indicates disk I/O congestion or a corrupt git index.

---

## 3. Visualizing Friction (The Heatmap)
The TUI and Web Deck will implement a **Station Friction Heatmap**.

- **🟢 Green Pulse:** All links within target latency.
- **🟡 Yellow Oscillation:** A specific component (e.g., SQLite) is exhibiting "Grind" (slow writes).
- **🔴 Red Static:** A critical bottleneck is detected (e.g., Spine connection pool is at 90% capacity).

---

## 4. Professional Inspector: `koad dood bottleneck`
A new tactical command for the Admiral to find the source of station lag.

```bash
# Example Output
$ koad dood bottleneck --full

[FRICTION REPORT: TRC-99-SKY]
1. CLI -> Spine: 4ms [PASS]
2. Spine -> Redis: 12ms [WARN: High Contention]
3. Spine -> SQLite: 340ms [FAIL: I/O WAIT]
4. Total Intent Latency: 356ms

Bottleneck Identified: SQLite Disk Write (WAL Lock).
Suggested Action: Run 'koad system vacuum' or check for ghost I/O processes.
```

---

## 5. Implementation Standard: "The Latency Budget"
Every new feature implemented in v5.0 MUST include a **Latency Budget** in its KSRP report.
- **Rule:** If a feature increases the NRT (Neural Round Trip) by >10% without a structural justification, it is rejected during the **Efficiency Sweep**.

---
*Next: Final Consolidation of the v5.0 Master Specification.*
