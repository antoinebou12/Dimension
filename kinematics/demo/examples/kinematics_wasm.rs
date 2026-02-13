//! WASM demo: 3-joint arm with forward kinematics and Jacobian IK.
//!
//! Build: just build-kinematics-demo-wasm
//! HTML must include <canvas id="canvas"></canvas>.
//!
//! Controls: Left drag = orbit, Right drag = IK target, Scroll = zoom.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Build for wasm32: just build-kinematics-demo-wasm");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    kinematics_demo::run_wasm().map_err(|e| JsValue::from_str(&e.to_string()))
}
