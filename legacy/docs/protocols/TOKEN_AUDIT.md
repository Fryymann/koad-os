# KoadOS Protocol: tokenaudit (Token Efficiency)

## 1. Overview
The `tokenaudit` protocol is a mandatory 5-pass review process designed to minimize operational costs and maximize agent performance by eliminating low-value token spend.

## 2. The 5-Pass Framework

### Pass 1: Redundancy Scan (Context Management)
- **Check:** Are we injecting the same project info or system rules multiple times?
- **Metric:** Context overlap ratio.
- **Action:** Consolidate redundant knowledge entries and use hierarchical injection.

### Pass 2: Verbosity Scrub (Interface Hygiene)
- **Check:** Is the CLI returning human-centric "fluff" (preambles, redundant confirmations)?
- **Metric:** Signal-to-Noise bytes.
- **Action:** Implement `--compact` mode by default for agent sessions. Strip non-essential formatting.

### Pass 3: Tool-Call Efficiency (Operational Logic)
- **Check:** Is the agent calling discovery tools (`ls`, `grep`) repeatedly for the same data?
- **Metric:** Redundant tool calls per task.
- **Action:** Improve directive guidance and implement Spine-level discovery caching.

### Pass 4: Payload Trimming (Data Transport)
- **Check:** Are gRPC responses sending full objects when only a status or ID is needed?
- **Metric:** JSON payload size overhead.
- **Action:** Surgical field selection in proto definitions.

### Pass 5: Persona Compaction (Identity Density)
- **Check:** Are KAI Bios and Mission Briefings excessively wordy?
- **Metric:** Character count of agent hydration.
- **Action:** Convert long-form persona descriptions into "High-Density" specialized instructions.

## 3. Tooling
- `koad system tokenaudit`: Analyzes current configuration and telemetry for efficiency gaps.
- `koad intel snippet`: (Implemented #63) - Example of a Pass 3 optimization.

## 4. Compliance
All new features must pass a `tokenaudit` review before being moved to 'Done' on the Command Deck.
