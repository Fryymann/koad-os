//! Koad-Core: The Hull of the Spaceship
//! Shared traits, types, and constants for the KoadOS workspace.

pub mod config;
pub mod hierarchy;
pub mod constants;
pub mod health;
pub mod identity;
pub mod intelligence;
pub mod intent;
pub mod logging;
pub mod session;
pub mod signal;
pub mod utils;
pub mod storage;
pub mod types;

/// The basic trait for any system component that can be started and stopped.
#[async_trait::async_trait]
pub trait Component: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&self) -> anyhow::Result<()>;
    async fn stop(&self) -> anyhow::Result<()>;
}
