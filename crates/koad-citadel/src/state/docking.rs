//! State machine for the agent docking lifecycle.
//!
//! Defines the seven-state [`DockingState`] enum and the [`DockingEvent`]
//! transitions that drive it from `Dormant` through to `Teardown`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// The 7-state lifecycle for agent docking.
/// DORMANT → DOCKING → HYDRATING → ACTIVE → WORKING → DARK → TEARDOWN
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DockingState {
    /// No active session — agent is offline.
    Dormant,
    /// CreateLease received, processing registration.
    Docking,
    /// Loading identity and context from SQLite into Redis.
    Hydrating,
    /// Fully online, heartbeat current.
    Active,
    /// Agent has an assigned worktree and is executing a task.
    Working,
    /// Heartbeat missed for >30s. Degraded but alive.
    Dark,
    /// Session closing or purge timeout exceeded.
    Teardown,
}

/// Events that trigger state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DockingEvent {
    /// A new session lease has been created for the agent.
    LeaseCreated,
    /// The hydration process (loading identity/context) has begun.
    HydrationStart,
    /// Hydration completed successfully; agent is fully initialised.
    HydrationDone,
    /// A heartbeat was received from the agent.
    HeartbeatReceived,
    /// A heartbeat was missed; agent may be degraded.
    HeartbeatMiss,
    /// A worktree has been assigned and the agent is executing a task.
    WorktreeAssigned,
    /// The agent's current task has completed.
    WorkComplete,
    /// The session is closing or a purge timeout has been exceeded.
    SessionClosed,
}

impl DockingState {
    /// Attempt a state transition. Returns the new state or an error
    /// describing the invalid transition.
    pub fn transition(self, event: DockingEvent) -> Result<DockingState, String> {
        use DockingEvent::*;
        use DockingState::*;

        match (self, event) {
            (Dormant, LeaseCreated) => Ok(Docking),
            (Docking, HydrationStart) => Ok(Hydrating),
            (Hydrating, HydrationDone) => Ok(Active),
            (Active, WorktreeAssigned) => Ok(Working),
            (Working, WorkComplete) => Ok(Active),
            (Active | Working, HeartbeatMiss) => Ok(Dark),
            (Dark, HeartbeatReceived) => Ok(Active),
            (_, SessionClosed) => Ok(Teardown),
            (state, event) => Err(format!("Invalid transition: {:?} + {:?}", state, event)),
        }
    }

    /// Returns `true` if the agent is in an active-equivalent state (Active, Working, or Dark).
    pub fn is_alive(self) -> bool {
        matches!(self, Self::Active | Self::Working | Self::Dark)
    }
}

impl fmt::Display for DockingState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Dormant => write!(f, "DORMANT"),
            Self::Docking => write!(f, "DOCKING"),
            Self::Hydrating => write!(f, "HYDRATING"),
            Self::Active => write!(f, "ACTIVE"),
            Self::Working => write!(f, "WORKING"),
            Self::Dark => write!(f, "DARK"),
            Self::Teardown => write!(f, "TEARDOWN"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_happy_path_lifecycle() {
        let state = DockingState::Dormant;
        let state = state.transition(DockingEvent::LeaseCreated).unwrap();
        assert_eq!(state, DockingState::Docking);
        let state = state.transition(DockingEvent::HydrationStart).unwrap();
        assert_eq!(state, DockingState::Hydrating);
        let state = state.transition(DockingEvent::HydrationDone).unwrap();
        assert_eq!(state, DockingState::Active);
        let state = state.transition(DockingEvent::WorktreeAssigned).unwrap();
        assert_eq!(state, DockingState::Working);
        let state = state.transition(DockingEvent::WorkComplete).unwrap();
        assert_eq!(state, DockingState::Active);
        let state = state.transition(DockingEvent::SessionClosed).unwrap();
        assert_eq!(state, DockingState::Teardown);
    }

    #[test]
    fn test_dark_mode_recovery() {
        let state = DockingState::Active;
        let state = state.transition(DockingEvent::HeartbeatMiss).unwrap();
        assert_eq!(state, DockingState::Dark);
        let state = state.transition(DockingEvent::HeartbeatReceived).unwrap();
        assert_eq!(state, DockingState::Active);
    }

    #[test]
    fn test_invalid_transition() {
        let result = DockingState::Dormant.transition(DockingEvent::HeartbeatReceived);
        assert!(result.is_err());
    }

    #[test]
    fn test_session_closed_from_any_state() {
        // SessionClosed should work from any state
        for state in [
            DockingState::Dormant,
            DockingState::Docking,
            DockingState::Active,
            DockingState::Working,
            DockingState::Dark,
        ] {
            let result = state.transition(DockingEvent::SessionClosed);
            assert_eq!(result.unwrap(), DockingState::Teardown);
        }
    }
}
