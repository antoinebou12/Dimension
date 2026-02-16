//! physics-demo — 3D demo of XPBD rigid body physics (winit + WASM).
//!
//! **Controls:** Left-drag = orbit camera; Ctrl+drag = pan; wheel = zoom.
//! Use the "Drop Sphere" / "Drop Box" UI buttons to spawn bodies.
//! Press S = sphere, B = box, R = reset (native only).
//!
//! Run native: `cargo run -p physics-demo --example physics_native`
//! Run WASM: build with `just build-physics-demo-wasm`, then serve and open the physics demo page.

mod scene;
mod wasm_debug;

pub use scene::{
    build_bodies_tree_panel, build_physics_scene, format_bodies_tree, step_physics,
    sync_bodies_to_world, PhysicsScene, BODIES_TREE_WINDOW_ID,
};

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;

/// Return the bodies tree as a newline-separated string for the HTML tree panel. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_bodies_tree() -> String {
    crate::wasm::get_bodies_tree()
}

/// Return total momentum magnitude as a string.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_total_momentum() -> String {
    crate::wasm::get_total_momentum()
}

/// Enable or disable verbose debug logging. Use `?debug=1` in URL to auto-enable.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn set_debug_physics(enabled: bool) {
    wasm_debug::set_debug_physics(enabled);
}

/// Return whether debug logging is enabled.
#[must_use]
pub fn get_debug_physics() -> bool {
    wasm_debug::get_debug_physics()
}

/// Run on native (winit).
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    native::run()
}

/// Run on WASM (canvas, requestAnimationFrame).
#[cfg(target_arch = "wasm32")]
pub fn run_wasm() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    wasm::run()
}

/// Spawn sphere from JS (HTML toolbar). WASM only.
#[cfg(target_arch = "wasm32")]
pub fn spawn_sphere_js() {
    wasm::spawn_sphere_js();
}

/// Spawn box from JS (HTML toolbar). WASM only.
#[cfg(target_arch = "wasm32")]
pub fn spawn_box_js() {
    wasm::spawn_box_js();
}

/// Spawn jelly from JS (HTML toolbar). WASM only.
#[cfg(target_arch = "wasm32")]
pub fn spawn_jelly_js() {
    wasm::spawn_jelly_js();
}

/// Reset scene from JS (HTML toolbar). WASM only.
#[cfg(target_arch = "wasm32")]
pub fn reset_scene_js() {
    wasm::reset_scene_js();
}

/// Body count (rigid + soft, excluding ground). WASM only.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_body_count() -> u32 {
    wasm::get_body_count()
}
