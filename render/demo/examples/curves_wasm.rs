//! Curves WASM demo: line segment, Bézier, Hermite, B-spline.
//!
//! Build: cargo build -p render-demo --target wasm32-unknown-unknown --example curves_wasm --release

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Build for wasm32: cargo build -p render-demo --target wasm32-unknown-unknown --example curves_wasm");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    render_demo::run_wasm_curves().map_err(|e| JsValue::from_str(&e.to_string()))
}
