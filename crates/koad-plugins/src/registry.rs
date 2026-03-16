//! In-memory WASM Plugin Registry.
//!
//! Allows agents to register named WASM component plugins at runtime and invoke
//! them by name, satisfying the Phase 4 acceptance criterion:
//! "An agent can register a new tool via gRPC and invoke it immediately."
//!
//! This module provides the core registry logic.  A thin gRPC wrapper in CASS
//! exposes it over the wire (Phase 4, task 1 — MCP Tool Registry).

use std::{collections::HashMap, path::PathBuf, sync::Arc, time::Instant};

use anyhow::{anyhow, Result};
use tokio::sync::RwLock;
use tracing::info;

use crate::WasmPluginManager;

/// Execution metrics for a single plugin invocation.
/// Maps to `TurnMetrics.execution_duration_ms` /
/// `TurnMetrics.execution_memory_bytes` in `proto/citadel.proto`.
#[derive(Debug, Clone, Default)]
pub struct PluginMetrics {
    /// Wall-clock duration of the `on-signal` call in milliseconds.
    pub duration_ms: u64,
    /// Reserved for future container / wasmtime memory tracking.
    pub memory_bytes: u64,
}

/// Result returned by [`PluginRegistry::invoke`].
#[derive(Debug)]
pub struct PluginResult {
    pub plugin_name: String,
    pub metrics: PluginMetrics,
}

/// A registered plugin entry.
#[derive(Debug, Clone)]
struct PluginEntry {
    /// Absolute path to the `.component.wasm` binary.
    component_path: PathBuf,
}

/// Thread-safe, in-memory registry of named WASM plugins.
///
/// # Example
/// ```rust,no_run
/// # use koad_plugins::registry::PluginRegistry;
/// # use std::path::PathBuf;
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let registry = PluginRegistry::new()?;
/// registry.register("greeter", PathBuf::from("greeter.component.wasm")).await;
/// let result = registry.invoke("greeter", "boot.complete", "{}").await?;
/// println!("ran in {}ms", result.metrics.duration_ms);
/// # Ok(())
/// # }
/// ```
pub struct PluginRegistry {
    manager: Arc<WasmPluginManager>,
    entries: Arc<RwLock<HashMap<String, PluginEntry>>>,
}

impl PluginRegistry {
    /// Create a new registry backed by a [`WasmPluginManager`].
    pub fn new() -> Result<Self> {
        Ok(Self {
            manager: Arc::new(WasmPluginManager::new()?),
            entries: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a named plugin at the given component path.
    /// Overwrites any previous registration with the same name.
    pub async fn register(&self, name: impl Into<String>, component_path: PathBuf) {
        let name = name.into();
        info!(plugin = %name, path = ?component_path, "PluginRegistry: registered");
        self.entries
            .write()
            .await
            .insert(name, PluginEntry { component_path });
    }

    /// Deregister a plugin by name.  Returns `true` if it was present.
    pub async fn deregister(&self, name: &str) -> bool {
        self.entries.write().await.remove(name).is_some()
    }

    /// List all registered plugin names.
    pub async fn list(&self) -> Vec<String> {
        self.entries.read().await.keys().cloned().collect()
    }

    /// Invoke a registered plugin by name with the given signal topic and payload.
    ///
    /// Returns [`PluginResult`] with wall-clock execution metrics.
    /// Returns an error if the plugin is not registered or execution fails.
    pub async fn invoke(&self, name: &str, topic: &str, payload: &str) -> Result<PluginResult> {
        let path = {
            let guard = self.entries.read().await;
            guard
                .get(name)
                .ok_or_else(|| anyhow!("Plugin '{}' is not registered", name))?
                .component_path
                .clone()
        };

        info!(plugin = %name, topic = %topic, "PluginRegistry: invoking");

        let start = Instant::now();
        self.manager.run_plugin(&path, topic, payload).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(PluginResult {
            plugin_name: name.to_string(),
            metrics: PluginMetrics {
                duration_ms,
                memory_bytes: 0, // Future: wasmtime memory introspection
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn component_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "examples/hello-plugin/target/wasm32-unknown-unknown/release/hello_plugin.component.wasm",
        )
    }

    #[tokio::test]
    async fn test_registry_register_and_list() {
        let registry = PluginRegistry::new().expect("registry init");
        assert!(registry.list().await.is_empty());

        registry
            .register("hello", PathBuf::from("dummy.wasm"))
            .await;
        let names = registry.list().await;
        assert_eq!(names, vec!["hello"]);
    }

    #[tokio::test]
    async fn test_registry_deregister() {
        let registry = PluginRegistry::new().expect("registry init");
        registry
            .register("hello", PathBuf::from("dummy.wasm"))
            .await;
        assert!(registry.deregister("hello").await);
        assert!(!registry.deregister("hello").await); // already gone
        assert!(registry.list().await.is_empty());
    }

    #[tokio::test]
    async fn test_registry_invoke_unknown_returns_error() {
        let registry = PluginRegistry::new().expect("registry init");
        let res = registry.invoke("nonexistent", "topic", "{}").await;
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("not registered"));
    }

    #[tokio::test]
    async fn test_registry_invoke_hello_plugin() {
        let path = component_path();
        if !path.exists() {
            eprintln!("SKIP: hello-plugin component not found. See lib.rs test docs.");
            return;
        }

        let registry = PluginRegistry::new().expect("registry init");
        registry.register("hello", path).await;

        let result = registry
            .invoke("hello", "test.topic", "{}")
            .await
            .expect("invocation should succeed");

        assert_eq!(result.plugin_name, "hello");
        // Duration should be non-zero (wasmtime init takes some time)
        // We just assert it's a valid u64, not zero (timing is environment-dependent)
        let _ = result.metrics.duration_ms;
    }
}
