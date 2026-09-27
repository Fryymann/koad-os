//! KoadOS Dynamic Plugin System (WASM)
//!
//! # Modules
//! - Root: [`WasmPluginManager`] — low-level component loader.
//! - [`registry`]: [`PluginRegistry`] — named plugin registry with invocation metrics.
//! - [`NativePluginManager`] — `.so`/`.dll` dynamic library loader (unsafe, trusted plugins only).

#![allow(unsafe_code)]

pub mod registry;

use anyhow::{Context, Result};
use std::path::Path;
use tracing::info;
use wasmtime::{
    component::{bindgen, Component, HasSelf, Linker},
    Config, Engine, Store,
};

// Generate host-side bindings for our WIT world.
//
// For a world with only bare (non-interface) function imports, wasmtime 49
// generates the following at the current module scope:
//
//   struct CitadelHooks                       — component handle (exports live here)
//   trait CitadelHooksImports                 — host must implement all bare imports
//   CitadelHooks::add_to_linker::<T, D>(linker, getter) — registers the impl
//   CitadelHooks::instantiate_async(store, component, linker) → Self
//   CitadelHooks::call_invoke(store, topic, payload)
//
// The WIT package namespace (`koad:hooks`) does NOT produce a Rust module hierarchy.
// The trait name follows the world name: `CitadelHooksImports`, NOT `Host`.
bindgen!({
    path: "wit/hooks.wit",
    world: "citadel-hooks",
    imports: { default: async },
    exports: { default: async },
});

struct MyHostState;

impl CitadelHooksImports for MyHostState {
    async fn log(&mut self, msg: String) {
        info!("[WASM Plugin]: {}", msg);
    }
}

pub struct WasmPluginManager {
    engine: Engine,
}

impl WasmPluginManager {
    pub fn new() -> Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        Ok(Self { engine })
    }

    pub async fn run_plugin(&self, wasm_path: &Path, topic: &str, payload: &str) -> Result<String> {
        let mut store = Store::new(&self.engine, MyHostState);
        let mut linker: Linker<MyHostState> = Linker::new(&self.engine);

        // `HasSelf` tells the bindings the store data itself implements the imports.
        CitadelHooks::add_to_linker::<_, HasSelf<_>>(&mut linker, |state| state)?;

        let component = Component::from_file(&self.engine, wasm_path)
            .map_err(anyhow::Error::from)
            .with_context(|| format!("Failed to load plugin at {:?}", wasm_path))?;

        // `instantiate_async` returns `(CitadelHooks, Instance)`, not `(Self, Store<T>)`.
        let instance = CitadelHooks::instantiate_async(&mut store, &component, &linker).await?;

        let res = instance.call_invoke(&mut store, topic, payload).await?;

        Ok(res)
    }
}

/// Dynamic library plugin loader (`.so` / `.dll`).
///
/// # Safety
/// Dynamic library loading is inherently unsafe. The loaded library receives
/// host process memory access and must be trusted. Only load plugins from
/// verified, signed sources. Unloading is not supported at runtime due to
/// reference invalidation risks — registered libraries live for the process lifetime.
pub struct NativePluginManager {
    // Libraries are retained to prevent unloading (dropping a Library is UB if
    // any function pointers derived from it are still live).
    _libs: tokio::sync::RwLock<Vec<libloading::Library>>,
}

impl Default for NativePluginManager {
    fn default() -> Self {
        Self::new()
    }
}

impl NativePluginManager {
    pub fn new() -> Self {
        Self {
            _libs: tokio::sync::RwLock::new(vec![]),
        }
    }

    /// Load a `.so`/`.dll` plugin and invoke its `koad_invoke` export.
    ///
    /// The plugin must export:
    /// ```c
    /// extern "C" char* koad_invoke(const char* topic, const char* payload);
    /// ```
    /// The returned pointer must be a valid UTF-8 C string. The host does NOT
    /// free it — the plugin is responsible for static or leaked allocation.
    ///
    /// # Safety
    /// - `path` must point to a trusted, compiled shared library.
    /// - The `koad_invoke` symbol must have the expected signature.
    /// - The returned pointer must remain valid for the duration of this call.
    pub async fn invoke(
        &self,
        path: &std::path::Path,
        topic: &str,
        payload: &str,
    ) -> anyhow::Result<String> {
        use std::ffi::{CStr, CString};

        let topic_c = CString::new(topic)?;
        let payload_c = CString::new(payload)?;

        // SAFETY: Loading an untrusted library is unsafe. Callers must ensure
        // the library is from a trusted, verified source.
        let lib = unsafe { libloading::Library::new(path) }
            .map_err(|e| anyhow::anyhow!("Failed to load native plugin {:?}: {}", path, e))?;

        // SAFETY: The symbol must exist and match the expected extern "C" signature.
        let result_ptr = {
            let invoke_fn: libloading::Symbol<
                unsafe extern "C" fn(*const i8, *const i8) -> *const i8,
            > = unsafe { lib.get(b"koad_invoke\0") }
                .map_err(|e| anyhow::anyhow!("Symbol 'koad_invoke' not found: {}", e))?;

            // SAFETY: Calling the foreign function with valid, null-terminated C strings.
            unsafe { invoke_fn(topic_c.as_ptr(), payload_c.as_ptr()) }
        };

        if result_ptr.is_null() {
            anyhow::bail!("NativePlugin returned null pointer");
        }

        // SAFETY: The pointer must be a valid C string returned by the plugin for
        // the duration of this call. The library remains loaded (retained below)
        // so the pointer stays valid until we copy it into an owned String.
        let result = unsafe { CStr::from_ptr(result_ptr) }
            .to_string_lossy()
            .into_owned();

        // Retain the library so symbols remain valid for the process lifetime.
        self._libs.write().await.push(lib);

        Ok(result)
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

    /// Smoke test: confirms NativePluginManager constructs without error.
    #[tokio::test]
    async fn test_native_plugin_manager_new() {
        let _manager = NativePluginManager::new();
        // No panic — construction succeeds.
    }

    /// Smoke test: invoking a non-existent .so returns an error, not a panic.
    #[tokio::test]
    async fn test_native_plugin_manager_missing_lib_returns_error() {
        let manager = NativePluginManager::new();
        let result = manager
            .invoke(
                std::path::Path::new("/tmp/nonexistent_koad_plugin.so"),
                "topic",
                "{}",
            )
            .await;
        assert!(result.is_err(), "Expected error for missing library");
        let msg = result.unwrap_err().to_string();
        assert!(
            msg.contains("Failed to load native plugin"),
            "Unexpected error message: {}",
            msg
        );
    }

    /// Integration test: loads the hello-plugin WASM component and drives the
    /// `invoke` export.  The guest calls back the host `log` import.
    #[tokio::test]
    async fn test_plugin_manager_runs_wasm() {
        // Committed fixture built by `wasm-tools component new` from the
        // `wasm32-unknown-unknown` build of examples/hello-plugin. Fail rather
        // than skip when missing, so a broken runtime cannot pass silently.
        let component_path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("wit/hello-plugin.component.wasm");

        let manager = WasmPluginManager::new().expect("engine init");
        manager
            .run_plugin(&component_path, "test.topic", "{\"key\":\"value\"}")
            .await
            .expect("plugin should run without error");
    }
}
