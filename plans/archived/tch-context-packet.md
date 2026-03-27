# Plan: Temporal Context Hydration (TCH) — Context Packet Impl

## Objective
Reduce structural token waste during session boot by implementing a distilled "Ghost Summary" context packet. This fulfills the Phase 2 roadmap goal of high-impact cognitive offload.

## Key Files
- `crates/koad-cass/src/services/hydration.rs`: Enhance `hydrate` logic.
- `crates/koad-cli/src/handlers/boot.rs`: Call `Hydrate` RPC during boot.
- `crates/koad-cass/src/storage/mod.rs`: Add query for recent episodes.

## Implementation Steps

### 1. Enhance CASS Storage Queries
- Implement a method to fetch the last 3 `EpisodicMemory` entries for a specific agent.
- Implement a method to fetch high-significance `FactCard` entries for a specific agent.

### 2. Upgrade `CassHydrationService`
- Update `hydrate()` to:
    - Fetch the agent's recent session summaries (Episodes).
    - Fetch the agent's specific facts.
    - Format these into a "Memory" section in the markdown packet.
    - Ensure the total packet size respects the `token_budget`.

### 3. Integrate with `koad-agent boot`
- Update the `boot` handler in `koad-cli` to:
    - Make a gRPC call to `HydrationService::Hydrate`.
    - Accept a `--budget` flag (defaulting to 4000 tokens).
    - Write the returned `markdown_packet` to a temporary file (e.g., `~/.koad-os/current_context.md`).
    - Inform the agent about this file in the `eval` output.

## Verification
- Run `koad-agent boot` and verify that the output context file contains real facts and summaries instead of just placeholders.
- Verify token budgeting correctly truncates the packet.

## ROI Assessment
- **Current Burn:** ~5,000 tokens per boot (manual re-reading).
- **Target Burn:** ~500 tokens per boot (distilled packet).
- **Net Gain:** 4,500 tokens saved per agent session.
