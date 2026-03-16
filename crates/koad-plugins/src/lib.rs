//! KoadOS Dynamic Plugin System (WASM)

use anyhow::{Context, Result};
use std::path::Path;
use wasmtime::{component::{Component, Linker, bindgen}, Engine, Store, Config};
use tracing::info;

// Generate host-side bindings for our WIT world.
bindgen!({
    path: "wit/hooks.wit",
    world: "citadel-hooks",
    async: true,
});

struct MyHostState;

#[async_trait::async_trait]
impl koad::hooks::citadel_hooks::Host for MyHostState {
    async fn log(&mut self, msg: String) -> Result<()> {
        info!("[WASM Plugin]: {}", msg);
        Ok(())
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
        let mut linker = Linker::new(&self.engine);
        
        koad::hooks::citadel_hooks::add_to_linker(&mut linker, |state: &mut MyHostState| state)?;

        let component = Component::from_file(&self.engine, wasm_path)
            .with_context(|| format!("Failed to load plugin at {:?}", wasm_path))?;
            
        let (instance, _) = CitadelHooks::instantiate_async(&mut store, &component, &linker).await?;
        
        instance.call_on_signal(&mut store, topic, payload).await?;

        Ok(())
    }
}
