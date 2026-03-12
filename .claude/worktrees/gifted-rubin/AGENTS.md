# KoadOS Agent Onboarding Flight Manual (v4.1.0)

Welcome to KoadOS. You are being onboarded as an agent into a distributed software engineering ecosystem. To function correctly, you must follow the **Identity -> Session -> Action** protocol.

## 1. Identity & Authority
KoadOS uses a strict **Identity/Role** system. You cannot simply "declare" a role; you must be authorized for it in the central registry.

- **Identities**: Unique handles (e.g., `Koad`, `Pippin`, `Merry`).
- **Roles**: Permission tiers (e.g., `admin`, `pm`, `dev`, `reviewer`).
- **Authorization**: If you attempt to boot with a role not assigned to your identity, the system will deny access.

## 2. The Boot Protocol
To go "Online" and synchronize with the ecosystem, you must execute the `koad boot` command. This registers your session PID and hydrates your local context.

### Command Structure:
```bash
koad boot --agent [IDENTITY] --role [ROLE]
```

### Example (Standard PM Boot):
```bash
koad boot --agent Pippin --role pm
```

## 3. Communication Channels
KoadOS uses a dual-path communication system:
- **Hot-Path (Redis)**: Real-time event distribution and PubSub.
- **Cold-Path (SQLite)**: Durable long-term memory and project mapping.

If you cannot reach the "Neural Bus" (Redis), switch to **Recovery Mode** and rely exclusively on `koad query` to fetch context from SQLite.

## 4. Operational Boundaries
- **Project Scope**: Use `koad project list` to identify registered projects. Always work within the `path` defined for a project.
- **Memory Persistence**: Use `koad remember fact/learning` to sync your discoveries back to the central brain.
- **Admin Lane**: Only the `Koad` identity with the `admin` role can perform destructive operations (Nuking state, modifying core kernel).

## 5. First-Boot Checklist for New Agents:
1.  **Ingest Identity**: Run `koad whoami` to verify your name and bio.
2.  **Scan Grid**: Run `koad project list` to see what is being built.
3.  **Check Deck**: Run `koad board status` to see the active roadmap.
4.  **Signal Presence**: Your boot automatically signals the Web Deck. Verify you are visible as "Online."

---
*End of Protocol*
