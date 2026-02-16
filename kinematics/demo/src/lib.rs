//! kinematics-demo — 3D demo of kinematics: 3-joint arm with IK (winit + WASM).
//!
//! Run native: `cargo run --example kinematics_native`
//! Run WASM: build with `just build-kinematics-demo-wasm`, then serve and open `wasm-demo/index.html`.

mod scene;

#[cfg(feature = "bvh")]
mod bvh_import;

pub use scene::{
    apply_kinematics_action, build_armature_controls_panel, build_armature_tree_panel,
    build_kinematics_scene, build_scene_entity_panel, camera_view_forward, format_armature_tree,
    format_armature_tree_for_chain, randomize_ik_target_for_chain, screen_to_plane_at_point,
    screen_to_plane_y, screen_to_plane_y0, step_ik, sync_armature_to_world, ArmPreset, ChainIndex,
    ChainState, IkSolverType, KinematicsAction, KinematicsScene, ARMATURE_CONTROLS_WINDOW_ID,
    ARMATURE_TREE_WINDOW_ID, SCENE_ENTITY_WINDOW_ID,
};

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;

mod wasm_debug;

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

/// Set the IK solver from JS. Call with `"fabrik"`, `"jacobian"`, or `"ccd"`; applied on next frame to active chain.
#[cfg(target_arch = "wasm32")]
pub fn set_ik_solver(name: &str) {
    wasm::set_ik_solver(name);
}

/// Set the active chain from JS. Call with `"a"` or `"b"`; applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_active_chain(name: &str) {
    wasm::set_active_chain(name);
}

/// Set the arm preset for the active chain from JS. Call with `"spherical"`, `"revolute"`, `"mixed"`, or `"snake"`; applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_arm_preset(name: &str) {
    wasm::set_arm_preset(name);
}

/// Return the current IK solver. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_ik_solver() -> String {
    wasm::get_ik_solver()
}

/// Return comma-separated list of solver names for the UI (e.g. "fabrik,jacobian,ccd").
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_available_solvers() -> String {
    wasm::get_available_solvers()
}

/// Return the armature tree as a newline-separated string for the HTML tree panel. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_armature_tree() -> String {
    wasm::get_armature_tree()
}

/// Return the end-effector position as `"x,y,z"`. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_ee_position() -> String {
    wasm::get_ee_position()
}

/// Return the current end-effector node index. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_end_effector_index() -> u32 {
    wasm::get_end_effector_index()
}

/// Return the number of nodes in the active chain's armature. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_armature_node_count() -> u32 {
    wasm::get_armature_node_count()
}

/// Return the IK target position as `"x,y,z"`. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_target_position() -> String {
    wasm::get_target_position()
}

/// Set the end-effector node index. Applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_end_effector(idx: u32) {
    wasm::set_end_effector(idx);
}

/// Trigger randomize IK target. Applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn randomize_target() {
    wasm::randomize_target();
}

/// Set the IK target to the current end-effector position for the active chain. Applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_target_to_ee() {
    wasm::set_target_to_ee();
}

/// Reset the scene to initial state. Applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn reset_scene() {
    wasm::reset_scene();
}

/// Load BVH from bytes and replace the active chain on next frame. Returns true if parse/conversion succeeded. Requires `bvh` feature.
#[cfg(all(target_arch = "wasm32", feature = "bvh"))]
#[must_use]
pub fn load_bvh_from_bytes(bytes: &[u8]) -> bool {
    wasm::load_bvh_from_bytes(bytes)
}

/// Enable or disable verbose debug logging. Use `?debug=1` in URL to auto-enable.
#[cfg(target_arch = "wasm32")]
pub fn set_debug_kinematics(enabled: bool) {
    wasm_debug::set_debug_kinematics(enabled);
}

/// Return whether debug logging is enabled.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_debug_kinematics() -> bool {
    wasm_debug::get_debug_kinematics()
}

/// Return the last Hessian snapshot when solver is Hessian: `{ hessian, size, error }` or null.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_hessian_snapshot() -> wasm_bindgen::JsValue {
    wasm::get_hessian_snapshot()
}
