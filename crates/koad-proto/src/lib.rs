//! KoadOS gRPC Protocol Definitions
//!
//! This crate contains the auto-generated Rust bindings for the KoadOS gRPC services
//! (Citadel, CASS, and Skill). Do not edit the generated files directly.

#![cfg_attr(not(test), warn(missing_docs))]

/// Skill service definitions.
pub mod skill {
    tonic::include_proto!("koad.skill");
}

/// Citadel (Control Plane) service definitions.
pub mod citadel {
    /// Version 5 of the Citadel protocol.
    pub mod v5 {
        tonic::include_proto!("koad.citadel.v5");
    }
}

/// CASS (Cognition/Memory) service definitions.
pub mod cass {
    /// Version 1 of the CASS protocol.
    pub mod v1 {
        tonic::include_proto!("koad.cass.v1");
    }
}
