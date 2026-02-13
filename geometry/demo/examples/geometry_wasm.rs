//! WASM geometry demo (browser).
//!
//! Build with: cargo build -p geometry-demo --target wasm32-unknown-unknown --example geometry_wasm
//! Then use wasm-bindgen-cli to generate pkg and serve.
//!
//! Requires a page with `<canvas id="canvas">` and optional `<div id="error">` with `<span id="error-message">`.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Build for wasm32: just build-geometry-demo-wasm");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    geometry_demo::run_wasm().map_err(|e| JsValue::from_str(&e.to_string()))
}
