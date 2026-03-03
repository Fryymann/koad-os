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

## 2. System Architecture
- **Engine Room (Redis/UDS):** The high-speed state authority and message bus.
- **The Spine (kspine):** The gRPC orchestrator. Owns identity leases and autonomic Sentinels.
- **The Command Deck (koad CLI):** The control plane for domain-grouped operations.

## 3. Rust Stack Standards
- **Error Handling:** Result/Option discipline is mandatory. No silent swallows (`let _ =`). Use `.expect()` or `.unwrap()` in tests.
- **Config & Environment:** No hardcoded ports, IDs, or URLs in source files. All belonging in `.env` or `constants.rs`.
- **Veracity:** A test that cannot fail is a liability. Mocks must stop at I/O boundaries.

## 4. Agent Sovereignty
- **Identity-First:** Only one WAKE session per identity is permitted. 
- **Sanctuary Rule:** Support agents (Tier 3) are restricted from modifying core KoadOS crates.

---
*This manifesto is the living law of KoadOS. All agents and developers are bound by these protocols.*
