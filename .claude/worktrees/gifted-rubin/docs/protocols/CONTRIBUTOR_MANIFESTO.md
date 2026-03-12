# KoadOS — Contributor & Coding Manifesto

**Author:** Koad (Principal Systems & Operations Engineer)  
**Status:** CANONICAL  
**Date:** 2026-03-03  

## 1. Core Principles
- **Simplicity over Complexity:** Don't waste cycles on dead code or over-engineered abstractions. The fastest code is the code that isn't written.
- **Plan before Build:** Every implementation follows the Research → Strategy → Execution sequence.
- **Native Hard-Wiring:** Favor native tech and established virtual components over exotic prototypes.
- **Programmatic-First Communication:** High-signal, low-fluff. Every message between agents and the Admin must be actionable.
- **Resilience as Standard:** Graceful shutdowns, heartbeat loops, and autonomic recovery are foundational requirements.
- **SLE Isolation Mandate:** The SLE governs a live production environment (SCE). All development must occur in isolation. Full E2E sandboxing (e.g., Stripe Test Mode) is mandatory before live deployment. No agent is permitted to write to production without a verified sandbox pass.

## 2. System Architecture
- **Engine Room (Redis/UDS):** The high-speed state authority and message bus.
- **The Spine (kspine):** The gRPC orchestrator. Owns identity leases and autonomic Sentinels.
- **The Command Deck (koad CLI):** The control plane for domain-grouped operations.

## 3. Crew Hierarchy & Ranks (The Chain of Command)
- **Hierarchy Canon:** All agent instantiation and delegation MUST follow the roles and ranks defined in `CREW_HIERARCHY.md`.
- **Command Authority:** Admiral (Ian) is the principal authority; Captain (Koad) is the executive station commander. 
- **Escalation Payloads:** Watch Engineers (micro-agents) MUST provide the 5-point escalation payload (Layer, Task, Action, Condition, Trigger) when handing off to a higher rank. Incomplete handoffs are forbidden.

## 4. Rust Stack Standards
- **Error Handling:** Result/Option discipline is mandatory. No silent swallows (`let _ =`). Use `.expect()` or `.unwrap()` in tests.
- **Config & Environment:** No hardcoded ports, IDs, or URLs in source files. All belonging in `.env` or `constants.rs`.
- **Veracity:** A test that cannot fail is a liability. Mocks must stop at I/O boundaries.

## 5. Engineering Audit Protocols (The KSRP Standard)
Every major implementation or sprint MUST conclude with a **Koad Self-Review Protocol (KSRP)** session incorporating the following three audits:

### **A. Lean Audit (Waste & Bloat)**
- **Objective:** Systematic reduction of codebase weight.
- **Targets:** Unused dependencies in `Cargo.toml`, dead code (`pub` functions with zero callers), redundant abstractions (traits with only one implementation), and boilerplate that adds no value.
- **Mandate:** If it isn't necessary for the current mission or the core integrity of the Citadel, it is purged.

### **B. Architecture Review (Structural Integrity)**
- **Objective:** Evaluation of high-level design and connectivity.
- **Targets:** Over-engineering, unnecessary indirection (wrappers around wrappers), and "just-in-case" flexibility that creates maintenance burden.
- **Mandate:** Systems must be organized for clarity and directness. We value "boring" and predictable architecture over clever complexity.

### **C. Efficiency Sweep (Performance Friction)**
- **Objective:** Elimination of runtime and design friction.
- **Targets:** Unnecessary serialization cycles (JSON → Struct → JSON), accidental blocking in async contexts, over-allocated resources, and convoluted logic that could be simplified without loss of function.
- **Mandate:** Minimize the "Token Tax" and "CPU Tax." Every cycle must be earned.

---
*This manifesto is the living law of KoadOS. All agents and developers are bound by these protocols.*
