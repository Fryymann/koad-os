# Mission Brief: Operation koad-agent MVP (Phase 5)
**Status:** ACTIVE
**Lead:** Clyde (Implementation Lead)
**Captain:** Tyr (Strategic Oversight)

## 1. Objective
To implement the `koad-agent` CLI as a standalone context generation engine and bootstrapper. This "Force Multiplier" phase focuses on reducing agent token burn by automating context preparation and shell hydration, working independently of the live Citadel gRPC kernel.

## 2. Background
Currently, agents spend 40-60% of their context window on "Discovery" (reading file lists, scanning crates, exploring schemas). `koad-agent context` will pre-compute this data into a surgical markdown packet, allowing the model to focus purely on reasoning from turn one.

## 3. Scope
- **Context Generation:** `koad-agent context <crate>` (Crate metadata, API maps, git history).
- **Identity Bootstrapping:** `koad-agent boot <identity>` (Env var hydration, runtime config generation).
- **Task Management:** `koad-agent task <manifest>` (Validation and worktree awareness).
- **Degraded Mode:** Ensure all MVP features work with zero running services (filesystem-only).

## 4. Key Deliverables
- [ ] `koad-agent context` command generating high-density context packets.
- [ ] `koad-agent boot` pattern for shell integration (eval $(...)).
- [ ] Support for identity-based runtime configuration (Claude/Gemini/Codex).

## 5. Success Criteria
- [ ] `koad-agent context koad-core` produces a packet under 5,000 tokens that covers the entire public API.
- [ ] An agent can boot into a clean worktree using only the generated context file.
- [ ] Zero dependency on a running `koad-citadel` or `koad-cass` for core MVP features.

---
*Signed,*
**Captain Tyr**
*Citadel Jupiter*
