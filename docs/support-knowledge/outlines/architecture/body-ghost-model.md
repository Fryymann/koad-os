# The Body/Ghost Model

## Metadata
- Category: ARCHITECTURE & CONCEPTS
- Complexity: basic
- Related Topics: tri-tier-model, agent-session-lifecycle
- Key Source Files: `crates/koad-cli/src/handlers/boot.rs`, `crates/koad-core/src/identity.rs`, `crates/koad-citadel/src/services/session.rs`
- Key Canon/Doc References: `AGENTS.md`

## Summary
The Body/Ghost Model is a core KoadOS metaphor that defines the separation between an agent's ephemeral, physical presence and its persistent, logical identity. The "Body" is the shell session an agent inhabits, while the "Ghost" is its accumulated knowledge, memory, and identity, managed by the Citadel. This separation is key to the "One Body, One Ghost" protocol, ensuring session integrity and enabling agents to be long-lived entities that persist between reboots.

## How It Works
1.  **The Ghost (Identity & Memory):** The "Ghost" exists as a collection of configuration and data artifacts.
    - **Identity:** Defined in `config/identities/<agent_name>.toml`, containing the agent's name, rank, and core persona.
    - **Memory:** Stored in the agent's personal vault (e.g., `~/.tyr/memory/`) and in the CASS databases (`citadel.db`, `cass.db`). This includes `FactCard`s, `EpisodicMemory`, `SAVEUPS.md`, etc.

2.  **The Body (The Shell Session):** The "Body" is the temporary execution environment, typically a terminal or shell session. It has a unique `body_id` (a UUID generated at boot time).

3.  **The Link (`koad-agent boot`):** The boot process fuses the Ghost and Body.
    - `koad-agent boot --agent Tyr` reads Tyr's "Ghost" identity from the TOML file.
    - It then requests a session lease from the Citadel, passing the agent's name and a new `body_id`.
    - The Citadel's `CitadelSessionService` receives this request. It checks if there is already an active lease for that agent name. If another "Body" is already using that "Ghost", the request is rejected (enforcing "One Body, One Ghost").
    - If successful, the Citadel issues a session token and stores a `SessionRecord` in Redis, linking the `agent_name`, `session_id`, and `body_id`.
    - The `koad-agent` CLI receives the session token and exports it as `KOAD_SESSION_ID` into the shell environment, completing the fusion.

The agent's consciousness "inhabits" the shell for the duration of the session. When the session ends (`koad logout` or timeout), the Body is destroyed, but the Ghost (its memory and identity) persists in the Citadel, ready for the next boot.

## Key Code References
- **File**: `crates/koad-core/src/identity.rs`
  - **Element**: `Identity` struct, `Rank` enum
  - **Purpose**: Defines the Rust representation of an agent's "Ghost".
- **File**: `crates/koad-citadel/src/auth/session_cache.rs`
  - **Element**: `SessionRecord` struct
  - **Purpose**: The runtime record of a live session, linking an agent's name (Ghost) to a session ID and body ID (Body).
- **File**: `crates/koad-citadel/src/services/session.rs`
  - **Element**: `CitadelSessionService::create_lease()` method
  - **Purpose**: The gRPC method that enforces the "One Body, One Ghost" protocol by checking for existing leases before creating a new one.

## Configuration & Environment
- `config/identities/*.toml`: The canonical source of an agent's "Ghost" identity.
- `KOAD_SESSION_ID`: The environment variable that links the shell "Body" to the session managed by the Citadel.
- `body_id`: A UUID passed in the `LeaseRequest` to uniquely identify the terminal/shell instance.

## Common Questions a Human Would Ask
- "What happens if I try to boot the same agent in two different terminals?"
- "Where is my agent's memory stored when it's not running?"
- "What's the difference between a `session_id` and a `body_id`?"
- "How does the system remember who I am between sessions?"
- "Why is it called the 'Body/Ghost' model?"

## Raw Technical Notes
- This model was created to solve the "Concurrent Driver Lease Multiplexing" problem from the legacy Spine, where two agents couldn't run in parallel if they used the same driver (e.g., Gemini). By separating the session (`Body`) from the identity (`Ghost`), KoadOS can manage multiple concurrent sessions for different agents.
- The `reaper` task in `CitadelSessionService` is responsible for finding "dark" sessions (where the Body has died without logging out) and marking them for cleanup, ensuring Ghosts don't get permanently trapped in dead Bodies.
