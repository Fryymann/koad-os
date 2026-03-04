//! Koad-Core: The Hull of the Spaceship
//! Shared traits, types, and constants for the KoadOS workspace.

pub mod config;
pub mod constants;
pub mod identity;
pub mod intent;
pub mod logging;
pub mod session;
pub mod storage;
pub mod utils;
pub mod types;

/// The basic trait for any system component that can be started and stopped.
#[async_trait::async_trait]
pub trait Component: Send + Sync {
    fn name(&self) -> &str;
    async fn start(&self) -> anyhow::Result<()>;
    async fn stop(&self) -> anyhow::Result<()>;
}
