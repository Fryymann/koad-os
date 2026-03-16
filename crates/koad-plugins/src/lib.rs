//! KoadOS Dynamic Plugin System (WASM)
//!
//! # Modules
//! - Root: [`WasmPluginManager`] — low-level component loader.
//! - [`registry`]: [`PluginRegistry`] — named plugin registry with invocation metrics.

pub mod registry;

use anyhow::{Context, Result};
use std::path::Path;
use tracing::info;
use wasmtime::{
    component::{bindgen, Component, Linker},
    Config, Engine, Store,
};

// Generate host-side bindings for our WIT world.
//
// For a world with only bare (non-interface) function imports, wasmtime 22.x
// generates the following at the current module scope:
//
//   struct CitadelHooks                       — component handle (exports live here)
//   trait CitadelHooksImports                 — host must implement all bare imports
//   trait CitadelHooksImportsGetHost<T>       — helper for the host-getter closure
//   CitadelHooks::add_to_linker_imports_get_host(linker, getter) — registers the impl
//   CitadelHooks::instantiate_async(store, component, linker) → (Self, Instance)
//   CitadelHooks::call_on_signal(store, topic, payload)
//
// The WIT package namespace (`koad:hooks`) does NOT produce a Rust module hierarchy.
// The trait name follows the world name: `CitadelHooksImports`, NOT `Host`.
bindgen!({
    path: "wit/hooks.wit",
    world: "citadel-hooks",
    async: true,
});

struct MyHostState;

// `CitadelHooksImports` uses explicit `Pin<Box<dyn Future>>` return types
// (wasmtime 22.x stable async pattern).  We implement each method by pinning
// an async block directly — no `#[async_trait]` needed.
impl CitadelHooksImports for MyHostState {
    fn log<'life0, 'async_trait>(
        &'life0 mut self,
        msg: wasmtime::component::__internal::String,
    ) -> ::core::pin::Pin<
        Box<dyn ::core::future::Future<Output = ()> + ::core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            info!("[WASM Plugin]: {}", msg);
        })
    }
}

pub struct WasmPluginManager {
    engine: Engine,
}

impl WasmPluginManager {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);
        let engine = Engine::new(&config)?;
        Ok(Self { engine })
    }

    pub async fn run_plugin(&self, wasm_path: &Path, topic: &str, payload: &str) -> Result<()> {
        let mut store = Store::new(&self.engine, MyHostState);
        let mut linker: Linker<MyHostState> = Linker::new(&self.engine);

        // Register the host implementation with the linker.
        // The generated fn is an associated fn on `CitadelHooks`, not a free fn.
        // A named function (not a closure) is required so the borrow checker can
        // infer the `for<'a>` higher-ranked lifetime bound on the getter.
        fn get_host(state: &mut MyHostState) -> &mut MyHostState {
            state
        }
        CitadelHooks::add_to_linker_imports_get_host(&mut linker, get_host)?;

        let component = Component::from_file(&self.engine, wasm_path)
            .with_context(|| format!("Failed to load plugin at {:?}", wasm_path))?;

        // `instantiate_async` returns `(CitadelHooks, Instance)`, not `(Self, Store<T>)`.
        let (instance, _instance_handle) =
            CitadelHooks::instantiate_async(&mut store, &component, &linker).await?;

        instance.call_on_signal(&mut store, topic, payload).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    /// Smoke test: confirms the host engine initialises without error.
    #[tokio::test]
    async fn test_plugin_manager_new() {
        let manager = WasmPluginManager::new();
        assert!(manager.is_ok(), "WasmPluginManager::new() should succeed");
    }

    /// Integration test: loads the hello-plugin WASM component and drives the
    /// `on-signal` export.  The guest calls back the host `log` import.
    ///
    /// Requires the guest component to be built first:
    ///   1. cargo build \
    ///        --manifest-path crates/koad-plugins/examples/hello-plugin/Cargo.toml \
    ///        --target wasm32-unknown-unknown --release
    ///   2. wasm-tools component new \
    ///        crates/koad-plugins/examples/hello-plugin/target/wasm32-unknown-unknown/release/hello_plugin.wasm \
    ///        -o crates/koad-plugins/examples/hello-plugin/target/wasm32-unknown-unknown/release/hello_plugin.component.wasm
    ///
    /// No WASI adapter is needed because the guest uses no WASI syscalls —
    /// it only calls back the host `log` import.
    #[tokio::test]
    async fn test_plugin_manager_runs_wasm() {
        // `.component.wasm` is a pure WASM component produced by `wasm-tools component new`
        // from the `wasm32-unknown-unknown` build of hello-plugin.
        let component_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "examples/hello-plugin/target/wasm32-unknown-unknown/release/hello_plugin.component.wasm",
        );

        if !component_path.exists() {
            eprintln!(
                "SKIP: hello-plugin component not found at {:?}. \
                 See test doc-comment for build instructions.",
                component_path
            );
            return;
        }

        let manager = WasmPluginManager::new().expect("engine init");
        manager
            .run_plugin(&component_path, "test.topic", "{\"key\":\"value\"}")
            .await
            .expect("plugin should run without error");
    }
}
