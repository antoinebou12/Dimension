//! geometry-demo — Native and WASM demos for the geometry crate.
//!
//! Run native: `cargo run -p geometry-demo --example geometry_native`
//! Run WASM: build with `just build-geometry-demo-wasm`, then serve and open.

mod scene;

pub use scene::build_geometry_scene;

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
