# Architecture Component Template

## 1. High-Level Summary
- **Component Name:** [Name of Component]
- **Primary Role:** [What is its single responsibility?]
- **Plane:** [Control Plane (Spine) | Data Plane (Redis) | Interface (Gateway/CLI)]

## 2. Mermaid Visualization
[Insert Mermaid Diagram detailing the component's internal flow and its connections to the Dual-Bus]
```mermaid
graph TD
    %% Diagram goes here
```

## 3. Interfaces & Contracts
### 3.1. Inputs (Listens To)
- **gRPC/Redis:** [Channel / RPC Method]
- **Payload:** [Expected Data Structure]

### 3.2. Outputs (Broadcasts / Returns)
- **gRPC/Redis:** [Channel / RPC Method]
- **Payload:** [Expected Data Structure]

## 4. State Management
- **Stateless/Stateful:** [Is this component purely functional, or does it hold state?]
- **Storage:** [If stateful, where is it persisted? SQLite vs. Redis Hash]

## 5. Failure Modes & Recovery
- **Known Failure States:** [What happens if it loses connection? Panic? Stale data?]
- **Recovery Protocol:** [Watchdog restart? Autonomic Hydration?]
