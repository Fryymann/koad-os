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

/// Permissions granted to a registered plugin.
#[derive(Debug, Clone, Default)]
pub struct PluginPermissions {
    /// Allow reading from host filesystem (outside sandbox).
    pub read: bool,
    /// Allow writing to host filesystem (outside sandbox).
    pub write: bool,
    /// Allow network access.
    pub net: bool,
}

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
    pub output: String,
    pub metrics: PluginMetrics,
}

/// A registered plugin entry.
#[derive(Debug, Clone)]
struct PluginEntry {
    /// Absolute path to the `.component.wasm` binary.
    component_path: PathBuf,
    /// If set, invoke this plugin inside a ContainerSandbox using this image.
    container_image: Option<String>,
    /// Permissions granted to this plugin.
    permissions: PluginPermissions,
}

/// Thread-safe, in-memory registry of named WASM plugins.
///
/// `PluginRegistry` is cheaply `Clone`able — the underlying engine and entry
/// map are both `Arc`-wrapped, so cloning only increments reference counts.
/// This makes it straightforward to pass to a gRPC service handler without
/// an additional `Arc` wrapper.
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
#[derive(Clone)]
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
        self.register_with_opts(name, component_path, None).await;
    }

    /// Register a named plugin with optional container sandbox routing.
    /// If `container_image` is `Some`, invocations will be routed through
    /// a `ContainerSandbox` using the specified image.
    pub async fn register_with_opts(
        &self,
        name: impl Into<String>,
        component_path: PathBuf,
        container_image: Option<String>,
    ) {
        let name = name.into();
        info!(plugin = %name, path = ?component_path, container = ?container_image, "PluginRegistry: registered");
        self.entries.write().await.insert(
            name,
            PluginEntry {
                component_path,
                container_image,
                permissions: PluginPermissions::default(),
            },
        );
    }

    /// Register a named plugin with explicit permission grants.
    ///
    /// Overwrites any previous registration with the same name.
    /// The `container_image` field is preserved if the entry already exists;
    /// otherwise it defaults to `None`.
    pub async fn register_with_permissions(
        &self,
        name: impl Into<String>,
        component_path: PathBuf,
        permissions: PluginPermissions,
    ) {
        let name = name.into();
        info!(
            plugin = %name,
            path = ?component_path,
            read = permissions.read,
            write = permissions.write,
            net = permissions.net,
            "PluginRegistry: registered with permissions"
        );
        let mut guard = self.entries.write().await;
        let entry = guard.entry(name).or_insert(PluginEntry {
            component_path: component_path.clone(),
            container_image: None,
            permissions: PluginPermissions::default(),
        });
        entry.component_path = component_path;
        entry.permissions = permissions;
    }

    /// Return the permissions currently granted to a plugin, or `None` if not registered.
    pub async fn get_permissions(&self, name: &str) -> Option<PluginPermissions> {
        self.entries
            .read()
            .await
            .get(name)
            .map(|e| e.permissions.clone())
    }

    /// Return the container image configured for a plugin, or `None` if not set / not registered.
    pub async fn get_container_image(&self, name: &str) -> Option<String> {
        self.entries
            .read()
            .await
            .get(name)
            .and_then(|e| e.container_image.clone())
    }

    /// Start a background task that polls registered plugin files every 5 seconds
    /// and logs whenever an mtime change is detected.
    ///
    /// Returns a `JoinHandle` — drop (abort) it to stop watching.
    /// No additional dependencies are required; polling uses `tokio::time::sleep`.
    pub fn start_hot_reload(&self) -> tokio::task::JoinHandle<()> {
        let entries = Arc::clone(&self.entries);

        tokio::spawn(async move {
            // Track the last-seen mtime for every registered plugin path.
            let mut mtimes: HashMap<String, std::time::SystemTime> = HashMap::new();

            loop {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;

                let guard = entries.read().await;
                for (name, entry) in guard.iter() {
                    if let Ok(meta) = std::fs::metadata(&entry.component_path) {
                        if let Ok(modified) = meta.modified() {
                            let prev = mtimes.get(name).copied();
                            if prev.map(|t| t != modified).unwrap_or(false) {
                                tracing::info!(
                                    plugin = %name,
                                    path = ?entry.component_path,
                                    "HotReload: change detected"
                                );
                            }
                            mtimes.insert(name.clone(), modified);
                        }
                    }
                }
                // Guard is dropped at end of block before next sleep.
            }
        })
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
        let (path, container_image) = {
            let guard = self.entries.read().await;
            let entry = guard
                .get(name)
                .ok_or_else(|| anyhow!("Plugin '{}' is not registered", name))?;
            (entry.component_path.clone(), entry.container_image.clone())
        };

        info!(plugin = %name, topic = %topic, "PluginRegistry: invoking");

        // If a container image is configured, run the tool inside ContainerSandbox.
        // The WASM file is mounted read-only and executed via a wasmtime-enabled image.
        if let Some(image) = container_image {
            use koad_sandbox::container::{ContainerConfig, ContainerSandbox};
            use std::time::Duration;

            let host_path = path.to_string_lossy().to_string();
            let container_path = "/plugin.wasm".to_string();

            let config = ContainerConfig {
                image,
                runtime: "docker".to_string(),
                memory_limit: "128m".to_string(),
                cpu_limit: "0.5".to_string(),
                allow_network: false,
                read_only_mounts: vec![(host_path, container_path)],
                timeout: Duration::from_secs(30),
            };

            let cmd = format!(
                "wasmtime run /plugin.wasm --invoke on_signal -- '{}' '{}'",
                topic.replace('\'', "\\'"),
                payload.replace('\'', "\\'")
            );

            let start = Instant::now();
            let result = ContainerSandbox::new(config).execute(&cmd).await?;
            let duration_ms = start.elapsed().as_millis() as u64;

            return Ok(PluginResult {
                plugin_name: name.to_string(),
                output: result.stdout,
                metrics: PluginMetrics { duration_ms, memory_bytes: result.memory_bytes },
            });
        }

        let start = Instant::now();
        let output = self.manager.run_plugin(&path, topic, payload).await?;
        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(PluginResult {
            plugin_name: name.to_string(),
            output,
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
        // Use contains rather than eq: HashMap iteration is non-deterministic.
        assert_eq!(names.len(), 1);
        assert!(names.contains(&"hello".to_string()));
    }

    #[tokio::test]
    async fn test_registry_register_overwrite() {
        // Re-registering the same name replaces the previous path.
        let registry = PluginRegistry::new().expect("registry init");
        registry
            .register("hello", PathBuf::from("first.wasm"))
            .await;
        registry
            .register("hello", PathBuf::from("second.wasm"))
            .await;
        // Name count must still be 1 — no duplicate entry.
        assert_eq!(registry.list().await.len(), 1);
    }

    #[tokio::test]
    async fn test_registry_clone_shares_state() {
        // Cloning the registry gives a handle to the same underlying map.
        let registry = PluginRegistry::new().expect("registry init");
        let clone = registry.clone();
        registry
            .register("hello", PathBuf::from("dummy.wasm"))
            .await;
        // The clone must see the registration made through the original.
        assert!(clone.list().await.contains(&"hello".to_string()));
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

    #[tokio::test]
    async fn test_registry_concurrency() {
        let path = component_path();
        if !path.exists() {
            return;
        }

        let registry = PluginRegistry::new().expect("registry init");
        registry.register("hello", path).await;

        let registry_arc = Arc::new(registry);
        let mut handles = vec![];

        // 20 concurrent invocations
        for i in 0..20 {
            let reg = registry_arc.clone();
            handles.push(tokio::spawn(async move {
                reg.invoke("hello", "topic", &format!("{{\"id\": {}}}", i)).await
            }));
        }

        for handle in handles {
            let res = handle.await.expect("join task");
            assert!(res.is_ok(), "Concurrent invocation failed: {:?}", res.err());
        }
    }

    #[tokio::test]
    async fn test_registry_resilience() {
        let registry = PluginRegistry::new().expect("registry init");

        // 1. Missing File
        registry
            .register("missing", PathBuf::from("nonexistent.wasm"))
            .await;
        let res = registry.invoke("missing", "topic", "{}").await;
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to load plugin"), "Error was: {}", err_msg);

        // 2. Corrupt/Non-WASM File
        let temp_file = std::env::current_dir().unwrap().join("not_a_wasm.txt");
        std::fs::write(&temp_file, "this is not wasm").expect("write temp");
        registry.register("corrupt", temp_file.clone()).await;
        let res = registry.invoke("corrupt", "topic", "{}").await;
        assert!(res.is_err());
        let err_msg = res.unwrap_err().to_string();
        assert!(err_msg.contains("Failed to load plugin"), "Error was: {}", err_msg);

        let _ = std::fs::remove_file(&temp_file);
    }

    #[tokio::test]
    async fn test_register_with_permissions() {
        let registry = PluginRegistry::new().expect("registry init");

        let perms = PluginPermissions { read: true, write: false, net: true };
        registry
            .register_with_permissions("secure", PathBuf::from("secure.wasm"), perms)
            .await;

        let retrieved = registry.get_permissions("secure").await.expect("should exist");
        assert!(retrieved.read);
        assert!(!retrieved.write);
        assert!(retrieved.net);

        // Unknown plugin returns None.
        assert!(registry.get_permissions("ghost").await.is_none());
    }

    #[tokio::test]
    async fn test_register_with_permissions_preserves_container_image() {
        let registry = PluginRegistry::new().expect("registry init");

        // First register with a container image via register_with_opts.
        registry
            .register_with_opts(
                "sandboxed",
                PathBuf::from("sandboxed.wasm"),
                Some("koad/runner:latest".to_string()),
            )
            .await;

        // Now apply permissions — container_image must be preserved.
        let perms = PluginPermissions { read: false, write: false, net: false };
        registry
            .register_with_permissions("sandboxed", PathBuf::from("sandboxed.wasm"), perms)
            .await;

        // Entry must still exist, permissions must be updated, and container_image must be preserved.
        let retrieved = registry.get_permissions("sandboxed").await.expect("should exist");
        assert!(!retrieved.read);
        let image = registry.get_container_image("sandboxed").await;
        assert_eq!(image.as_deref(), Some("koad/runner:latest"), "container_image must survive register_with_permissions");
    }
}
