//! hello-plugin — minimal KoadOS WASM guest component.
//!
//! Demonstrates the WIT component model round-trip:
//!   host calls `on-signal` → guest calls host `log` import → host logs the message.

// Generate guest-side bindings from the shared WIT world.
// The path is resolved relative to this crate's CARGO_MANIFEST_DIR.
wit_bindgen::generate!({
    path: "../../wit/hooks.wit",
    world: "citadel-hooks",
});

struct HelloPlugin;

impl Guest for HelloPlugin {
    /// Called by the host via the `on-signal` export.
    fn on_signal(topic: String, payload: String) {
        // Call back the host's `log` import.
        log(&format!(
            "[hello-plugin] received signal: topic='{}', payload='{}'",
            topic, payload
        ));
    }
}

// Register `HelloPlugin` as the implementation of the `citadel-hooks` world.
export!(HelloPlugin);
