//! Citadel State Management
//!
//! Manages persistent agent state: bay provisioning, docking lifecycle,
//! and the SQLite/Redis storage bridge.

pub mod bay_store;
pub mod docking;
pub mod storage_bridge;
