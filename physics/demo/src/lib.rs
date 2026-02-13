//! physics-demo — 3D demo of PBD physics (winit + WASM).
//!
//! **Controls:** Left-drag = orbit camera; Ctrl+drag = pan; wheel = zoom.
//! Use the "Drop object" UI button to spawn bodies that fall onto the ground plane.
//!
//! Run native: `cargo run -p physics-demo --example physics_native`
//! Run WASM: build with `just build-physics-demo-wasm`, then serve and open the physics demo page.

mod scene;

pub use scene::{build_physics_scene, step_physics, sync_bodies_to_world, PhysicsScene};

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;

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
