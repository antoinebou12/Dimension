//! WASM demo: XPBD rigid body physics.
//!
//! Build with: cargo build -p physics-demo --target wasm32-unknown-unknown --example physics_wasm
//! Then use wasm-bindgen-cli to generate pkg and serve wasm-demo/.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Build for wasm32: just build-physics-demo-wasm");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    physics_demo::run_wasm().map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Return the bodies tree as a newline-separated string for the HTML tree panel. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_bodies_tree() -> String {
    physics_demo::get_bodies_tree()
}

/// Return total momentum magnitude as a string.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_total_momentum() -> String {
    physics_demo::get_total_momentum()
}

/// Enable or disable verbose debug logging. Use `?debug=1` in URL to auto-enable.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_debug_physics(enabled: bool) {
    physics_demo::set_debug_physics(enabled);
}

/// Return whether debug logging is enabled.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_debug_physics() -> bool {
    physics_demo::get_debug_physics()
}

/// Spawn a sphere from HTML toolbar. Applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn spawn_sphere() {
    physics_demo::spawn_sphere_js();
}

/// Spawn a box from HTML toolbar. Applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn spawn_box_body() {
    physics_demo::spawn_box_js();
}

/// Spawn a jelly (soft body) from HTML toolbar. Applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn spawn_jelly() {
    physics_demo::spawn_jelly_js();
}

/// Reset the scene from HTML toolbar. Applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn reset_scene() {
    physics_demo::reset_scene_js();
}

/// Return current body count (rigid + soft, excluding ground) for HTML display.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_body_count() -> u32 {
    physics_demo::get_body_count()
}
