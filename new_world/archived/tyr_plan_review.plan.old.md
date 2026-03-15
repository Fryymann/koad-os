# Tyr's Strategic Review: KoadOS Rebuild (Citadel-First Priority)
**Date:** 2026-03-12
**Status:** CONDITION GREEN (Approved with "Bootstrap" Course Correction)

## 1. Executive Summary
The revised `DEVELOPMENT_PLAN.md` (gifted-rubin) correctly pivots to a **Citadel-First** foundation. Prioritizing the "OS" layer (Citadel) before the "Brain" layer (CASS) is the correct engineering sequence to ensure agents have a stable "Body" to inhabit before they begin accumulating "Memory." The inclusion of the **Blockers** section (🔴/🟡) effectively addresses the architectural debt identified in my previous review.

## 2. Strategic Pivot Assessment
- **Citadel MVP (Phase 2):** Correctly scoped to session brokering and Personal Bays.
- **Agent Contribution Point:** The plan aims for agents (Tyr, Sky, Vigil) to help build CASS. This is a high-leverage move but introduces a "Bootstrap Gap" (see below).
- **Archival Strategy:** The "Archive, don't migrate" decision for the Spine is the only way to ensure a zero-trust foundation.

## 3. Identified Gaps & Friction Points (Revised)

### 🔴 The "Bootstrap Gap" (Phase 4 vs. Phase 2/3)
- **Gap:** The plan builds the **Citadel** (Phase 2) and **CASS** (Phase 3), but the tool used to *boot* agents into the Citadel (`koad-agent`) is deferred to **Phase 4**. 
- **Conflict:** If agents are supposed to contribute to building CASS (Phase 3), they need to be able to connect to the Citadel MVP and see the CASS blueprints. Without `koad-agent`, we are forced to manually "hand-wire" the agents into the new environment, which is prone to error and violates the "One Body, One Ghost" protocol.
- **Recommendation:** Split `koad-agent` into two parts. Move **Phase 4.1 (Bootstrap Flow)** and **Phase 4.3 (Identity TOML)** into **Phase 1.5**. A "Minimal `koad-agent`" must exist *before* Phase 2 completes so that agents can inhabit the new Citadel to help build CASS.

### 🔴 The "Admin Sovereignty" Role
- **Gap:** Blocker 8 asks if the Operator (Ian) has a special bay. Phase 2.3 mandates "No unauthenticated endpoints." 
- **Conflict:** In an emergency (e.g., Citadel state corruption), there must be a way to interact with the gRPC layer without a provisioned "Ghost" session.
- **Recommendation:** Define an **`ADMIN_OVERRIDE`** role in `citadel.proto` and `kernel.toml` that maps to a local Unix Domain Socket (UDS) with filesystem-level permissions. This allows Dood (Ian) or Tyr (as Captain) to perform maintenance even if the Auth/Lease system is stalled.

### 🟡 Blueprint Injection (Pre-CASS)
- **Gap:** Agents booting in Phase 2 (Citadel-only) will lack CASS memory.
- **Recommendation:** Standardize a `blueprints/` directory in the repo. `koad-agent` should be instructed to always inject `blueprints/CASS/*.md` into the Tier 2 context for any agent whose role includes "Developer" or "Architect" during Phase 2. This allows us to "fake" the memory hydration until the actual memory system is online.

### 🟡 Personal Bay Volatility
- **Gap:** Blocker 8 asks if Bay storage is Redis or SQLite.
- **Recommendation:** The **Health Record** and **Filesystem Map** MUST be SQLite-backed. If the Citadel crashes and Redis is wiped, an agent should be able to reconnect and have their "physical world" (assigned paths) immediately restored. Redis should be reserved for the **Transient Lease** and **Live Heartbeat** only.

## 4. Specific Plan Hardening (Applied to Blockers)
- **Blocker 1 (Dark Mode):** Recommendation: Use **TOML frontmatter** in `.md` files. It is easier for agents to generate correctly than JSON within a markdown block.
- **Blocker 5 (Path Discrepancy):** Approved: `config/identities/` is the canonical path. `personas/` should be officially retired in Phase 0.3.
- **Blocker 7 (CLI Surface):** I suggest adding `koad system doctor` to this list for automated verification of the new tri-tier environment.

## 5. Readiness Assessment
**Condition:** GREEN.
The plan is superior to the previous draft. By resolving the 🔴 **Bootstrap Gap** (moving `koad-agent` prep to Phase 1.5), we ensure the agents are "online" and "useful" during the most critical construction phases.

---
**Reviewer:** Tyr, Captain (KAI Officer)
**Signature:** `[TYR-SIG-2026-03-12-GIFTED-RUBIN]`
