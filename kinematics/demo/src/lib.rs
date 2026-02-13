//! kinematics-demo — 3D demo of kinematics: 3-joint arm with IK (winit + WASM).
//!
//! Run native: `cargo run --example kinematics_native`
//! Run WASM: build with `just build-kinematics-demo-wasm`, then serve and open `wasm-demo/index.html`.

mod scene;

pub use scene::{
    apply_kinematics_action, build_armature_controls_panel, build_armature_tree_panel,
    build_kinematics_scene, build_scene_entity_panel, camera_view_forward,
    screen_to_plane_at_point, screen_to_plane_y, screen_to_plane_y0, step_ik,
    sync_armature_to_world, IkSolverType, KinematicsAction, KinematicsScene,
    ARMATURE_CONTROLS_WINDOW_ID, ARMATURE_TREE_WINDOW_ID, SCENE_ENTITY_WINDOW_ID,
};

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
