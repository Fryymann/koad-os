//! Koad-Core: The Hull of the Spaceship
//! Shared traits, types, and constants for the KoadOS workspace.

pub mod config;
pub mod constants;
pub mod health;
pub mod hierarchy;
pub mod identity;
pub mod intelligence;
pub mod intent;
pub mod logging;
pub mod session;
pub mod signal;
pub mod skills;
pub mod storage;
pub mod types;
pub mod utils;

/// The basic trait for any system component that can be started and stopped.
///
/// Implementors are typically long-running services (like gRPC servers or drain loops)
/// that need orchestrated lifecycle management by the Citadel Kernel.
#[async_trait::async_trait]
pub trait Component: Send + Sync {
    /// Returns the human-readable name of the component.
    fn name(&self) -> &str;

    /// Initializes and starts the component's background tasks.
    async fn start(&self) -> anyhow::Result<()>;

    /// Signals the component to stop and waits for a graceful teardown.
    async fn stop(&self) -> anyhow::Result<()>;
}
