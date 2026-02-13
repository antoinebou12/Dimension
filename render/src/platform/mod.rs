//! Platform: SDL3 and RunDemo (native/wasm run loop moved to render-demo).

/// Shared input sensitivity constants for orbit, zoom, and pan. Used by native, SDL3, and WASM demos.
pub mod input_constants {
    /// Mouse sensitivity for orbit (yaw/pitch).
    pub const ORBIT_SENSITIVITY: f32 = 0.005;
    /// Mouse wheel sensitivity for zoom.
    pub const ZOOM_SENSITIVITY: f32 = 0.001;
    /// Mouse drag sensitivity for pan (when Ctrl held).
    pub const PAN_SENSITIVITY: f32 = 1.0;
    /// FPS exponential moving average smoothing factor.
    pub const FPS_EMA_ALPHA: f32 = 0.9;
}

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(all(not(target_arch = "wasm32"), feature = "sdl3"))]
mod sdl3;

/// Initial scene preset when starting the render loop.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RunDemo {
    /// Default: cube, tetrahedron, cylinder.
    #[default]
    Default,
    /// Curve primitives: line segment, Bézier, Hermite, B-spline.
    Curves,
    /// All 3D primitives: cube, tetrahedron, cylinder, sphere, cone, capsule, line, Bézier, Hermite, B-spline.
    AllShapes,
    /// 2D-only: two circles (Aabb2/Circle collision), AABB outlines; orthographic camera; per-frame update.
    Aabb2d,
}

/// Run the native render loop with default scene (winit + forte).
#[cfg(not(target_arch = "wasm32"))]
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    native::run()
}

/// Run the native render loop with the given scene preset.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_demo(_demo: RunDemo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    native::run_with_demo(_demo)
}

/// Stub on WASM: use the render-demo crate for the run loop.
#[cfg(target_arch = "wasm32")]
pub fn run() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("use the render-demo crate for WASM run loop".into())
}

/// Stub on WASM: use the render-demo crate for the run loop.
#[cfg(target_arch = "wasm32")]
pub fn run_demo(_: RunDemo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    Err("use the render-demo crate for WASM run loop".into())
}

/// Run the render loop with SDL3 (requires `sdl3` feature).
#[cfg(all(not(target_arch = "wasm32"), feature = "sdl3"))]
pub fn run_sdl3() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sdl3::run_sdl3()
}
