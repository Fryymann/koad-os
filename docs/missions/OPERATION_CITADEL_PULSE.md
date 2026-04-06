# Mission Brief: Operation Citadel Pulse (Phase 7.5)
**Status:** ACTIVE
**Lead:** Clyde (Implementation Lead)
**Captain:** Tyr (Strategic Oversight)

## 1. Objective
To bridge the awareness gap across the KoadOS agent fleet. This mission implements a "Pulse" system that hydrates all agents with global project status, role-specific news, and recent system updates during the boot sequence (TCH).

## 2. Background
Currently, agents rely on their own recent session history for context. This leads to a lack of awareness of work performed by other agents or system-wide changes (e.g., Phase completions). `koad updates` is passive and underutilized.

## 3. Scope & Impact
- **CASS Hydration:** Inject global/role-based pulses into the TCH packet.
- **Koad CLI:** New `koad pulse` command for broadcasts.
- **Living Docs:** Automated synchronization of `MISSION.md` and `AGENTS.md` with recent update history.

## 4. Proposed Solution: The "Citadel Pulse"
- **L1 Memory (Redis):** Store pulses with a 48-hour TTL.
- **TCH Packet:** New Section 0: "Global Pulses" containing high-priority system news.
- **Pulse API:** gRPC methods in CASS to register and query active pulses.

## 5. Phased Implementation
- **Phase A (Infra):** CASS Storage & gRPC extension.
- **Phase B (CLI):** `koad pulse` command and `updates` hook.
- **Phase C (Docs):** Scribe-driven manifest synchronization.

## 6. Success Criteria
- [ ] Agent boots and correctly identifies a pulse message in its system context.
- [ ] `koad pulse` successfully broadcasts a message to all active worktrees.
- [ ] `MISSION.md` reflects the "Active Phase" accurately after a `koad updates` commit.

## 7. Protocol Compliance
- **Canon:** All gRPC calls must accept `TraceContext`.
- **KSRP:** Mandatory structured review for CASS hydration changes.
- **PSRP:** Daily reflection on token usage and mission progress.

---
*Signed,*
**Captain Tyr**
*Citadel Jupiter*
