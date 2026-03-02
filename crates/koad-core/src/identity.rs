use serde::{Deserialize, Serialize};

/// Represents the session identity of a user or agent within the Spaceship.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Identity {
    /// The display name or unique ID of the entity.
    pub name: String,
    /// The hierarchical rank of the entity (affects permission mapping).
    pub rank: Rank,
    /// A list of explicit permission strings granted to this identity.
    pub permissions: Vec<String>,
}

/// The hierarchical rank structure of KoadOS.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Rank {
    /// The Fleet Admiral (Ian). Full system control.
    Admiral,
    /// Koad (Admin). Primary system orchestrator.
    Captain,
    /// Named sub-agents (e.g., PM, Lead Dev).
    Officer,
    /// General sub-agents (e.g., Coders, Researchers).
    Crew,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity_serialization() {
        let identity = Identity {
            name: "Dood".to_string(),
            rank: Rank::Admiral,
            permissions: vec!["all".to_string()],
        };
        let serialized = serde_json::to_string(&identity).unwrap();
        let deserialized: Identity = serde_json::from_str(&serialized).unwrap();
        assert_eq!(identity, deserialized);
    }
}
