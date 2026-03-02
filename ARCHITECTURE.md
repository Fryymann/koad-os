# KoadOS Architecture: The Agentic Operating System

## 1. System Vision
KoadOS is not a static CLI or a standard web application; it is a **Distributed Agentic Operating System**. It is designed to host, orchestrate, and monitor autonomous AI agents (PMs, Developers, Reviewers) while providing the human Admin ("Dood") with absolute monitoring and override capabilities via unified Command Decks.

## 2. Core Architectural Diagram

This chart represents the current target architecture of KoadOS. It is maintained here and must be updated as the system evolves.

```mermaid
graph TD
    %% Oversight & Governance
    DECK[GitHub Command Deck: Project #2] --- ADMIN((Dood))
    DECK --- KCM[Koad Compliance Manager: v3.1 IMPLEMENTED]
    KCM --- OV[Compliance Agent: Overseer]
    
    %% Workflow Orchestration
    DECK --- WF[Spine Workflow Engine: v3.1 IMPLEMENTED]
    WF --- ACTIONS[GitHub Actions: CI/Hygiene]
    WF --- KERNEL[Koad Kernel: v3.1 BUILDER REFACTOR]

    OV --- KERNEL

    %% Core Memory (Hydrated & Durable)
    Redis[(Redis Hot Path)] --- SB[Storage Bridge: IMPLEMENTED]
    SQLite[(SQLite Cold Path)] --- SB
    SB --- KERNEL[Koad Kernel: v3.1 BUILDER REFACTOR]

    %% Agent Layer (Process Managed)
    KERNEL --- ASM[Agent Session Manager: IMPLEMENTED]
    ASM --- DEV((Developer))
    ASM --- PM((PM))
    
    %% Execution Layer (Fused & Secure)
    KERNEL --- CM[Command Manager + Sandbox: IMPLEMENTED]
    
    %% Isolated I/O (The Air Gap)
    KERNEL ---|UDS ONLY| EDGE[Isolated Edge Gateway: v3.1 IMPLEMENTED]
    EDGE -->|TCP 0.0.0.0| Web(Chrome)
    EDGE -->|TCP 0.0.0.0| TUI(Windows TUI)
    
    %% Diagnostics
    SPINE --- DOC[Koad Doctor]
```

## 3. Core Component Definitions

### A. The Spine (Event Bus)
The central nervous system. Rather than direct, blocking RPC calls between agents and managers, KoadOS uses an **Event-Driven Architecture** backed by Redis Streams. Agents publish *Intents*, and Managers consume them asynchronously, allowing massive concurrency without deadlocks.

### B. Storage Bridge
Abstracts the duality of KoadOS state:
- **Hot Path (Redis)**: Live telemetry, active task streaming, volatile context.
- **Cold Path (SQLite)**: Long-term memory, execution history, system configuration.

### C. Agent Session Manager (ASM)
The lifecycle controller for AI agents. It handles agent instantiation, identity verification, and context scoping. It ensures that a PM agent and a Developer agent operating in the same project share a unified view of reality.

### D. Command Manager + Sandbox
The execution engine for the OS. It translates agent Intents into literal shell commands or API calls.
- **Sandbox Policy**: Enforces strict security bounds based on the requesting Agent's role. For example, Developer agents are restricted from modifying KoadOS kernel files or using `sudo`, while the Koad Admin retains root privileges.

### E. Edge Gateway
The consolidated I/O edge of KoadOS. Instead of scattered servers causing port collisions, the Edge Gateway acts as a unified reverse-proxy and protocol upgrade layer (HTTP, WebSocket, gRPC) ensuring clean cross-environment connectivity (e.g., WSL to Windows 11).

## 4. KoadOS Development Canon
The management of KoadOS is strictly governed by the following protocols to ensure system integrity:
- **Ticket-First Workflow**: ALL work begins with a local `Ticket` object (Problem/Solution/Implementation) which mirrors to a GitHub Issue.
- **Anti-Overengineering Protocol (AOP)**: Every ticket MUST undergo a four-pillar evaluation before implementation:
    1. **Relevance Evaluation**: Does this feature align with the "Agentic OS" vision, or is it a legacy human tool?
    2. **Value Validation**: Does the security/utility gain outweigh the friction introduced to development and testing?
    3. **Utility Audit**: Is there a simpler, native way to achieve the goal without adding new dependencies or host-level state?
    4. **YAGNI Review**: "You Aren't Gonna Need It." Eliminate "just-in-case" architecture before the first line of code is written.
- **Tight Git Coupling**: Incremental development is enforced via surgical, issue-linked commits.
- **GitHub Orchestration**: All planning and status tracking is handled via GitHub Projects and Milestones.
- **Automated Compliance**: The `repo-clean` tool enforces zero-drift repository hygiene.
- **Self-Documenting Charts**: Architecture maps must be updated in real-time.

## 5. Security & Isolation (Strategic Direction)

KoadOS prioritizes **Agility and Scalability** for AI Micro-Agents. While Unix-user isolation was explored, the system has reverted to a **Consolidated User-Space Architecture** at `~/.koad-os/` to avoid host-level friction.

### 🛡 Sandbox Policy
- **Software-Level Isolation**: The `Sandbox` engine enforces command blacklisting (e.g., preventing `sudo` or access to sensitive Admin paths).
- **Process Isolation**: Future micro-agents will utilize lightweight user-space sandboxing (like `bwrap` or WASM) to ensure process integrity without requiring root-level Unix users.
