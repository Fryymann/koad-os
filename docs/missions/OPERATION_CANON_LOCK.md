# Mission Brief: Operation Canon Lock (Phase 6)
**Status:** ACTIVE
**Lead:** Scribe (Context Distillation)
**Captain:** Tyr (Strategic Oversight)

## 1. Objective
To "Freeze the Canon" by distilling all brainstorms, research, and architectural decisions into canonical, repo-resident documentation. This phase transitions the source of truth from Notion/External ideas to the KoadOS repository itself.

## 2. Background
Following the successful implementation of the gRPC Kernel (Phase 1), Memory Stack (Phase 7), and koad-agent MVP (Phase 5), the system has reached a critical mass of complexity. We must now lock the architectural definitions to ensure consistency as we move toward the Minion Swarm (Phase 9).

## 3. Scope
- **Architectural Specs:** Distill the Citadel Rebuild brainstorms.
- **Protocol Guardrails:** Formalize coding conventions and gRPC extension rules.
- **Map Alignment:** Ensure every crate's `AGENTS.md` and the master `SYSTEM_MAP.md` are 100% accurate.

## 4. Key Deliverables
- [ ] `docs/rebuild/ARCHITECTURE.md`: Canonical Citadel v3 architecture.
- [ ] `docs/rebuild/MINION_SWARM_SPEC.md`: Implementation spec for the swarm hangar.
- [ ] `docs/CONVENTIONS.md`: Coding standards, TraceContext, and Redis patterns.
- [ ] `docs/PROTO_GUIDE.md`: Guide for reading and extending the Signal/CASS protocols.

## 5. Success Criteria
- [ ] All high-level design decisions from the Notion brainstorm are captured in the repo.
- [ ] A new agent booting into the repo can understand the full architectural flow without external access.
- [ ] `SYSTEM_MAP.md` reflects all new crates and binaries created during Phases 4, 5, and 7.

---
*Signed,*
**Captain Tyr**
*Citadel Jupiter*
