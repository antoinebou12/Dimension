//! WASM demo: 3-joint arm with forward kinematics and FABRIK/Jacobian IK.
//!
//! Build: just build-kinematics-demo-wasm
//! HTML must include <canvas id="canvas"></canvas>.
//!
//! Controls: Left drag = orbit, Right drag = IK target, Scroll = zoom.
//! JS can call set_ik_solver("fabrik"|"jacobian"), get_armature_tree(), etc.

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

/// Set the IK solver. Call with "fabrik" or "jacobian"; applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_ik_solver(name: &str) {
    kinematics_demo::set_ik_solver(name);
}

/// Return the current IK solver. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_ik_solver() -> String {
    kinematics_demo::get_ik_solver()
}

/// Return comma-separated list of solver names for the UI.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_available_solvers() -> String {
    kinematics_demo::get_available_solvers()
}

/// Set the active chain. Call with "a" or "b"; applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_active_chain(name: &str) {
    kinematics_demo::set_active_chain(name);
}

/// Set the arm preset for the active chain. Call with "spherical", "revolute", "mixed", or "snake"; applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_arm_preset(name: &str) {
    kinematics_demo::set_arm_preset(name);
}

/// Return the armature tree as a newline-separated string for the HTML tree panel. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_armature_tree() -> String {
    kinematics_demo::get_armature_tree()
}

/// Return the end-effector position as "x,y,z". Updated each frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_ee_position() -> String {
    kinematics_demo::get_ee_position()
}

/// Return the current end-effector node index. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_end_effector_index() -> u32 {
    kinematics_demo::get_end_effector_index()
}

/// Return the number of nodes in the active chain's armature. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_armature_node_count() -> u32 {
    kinematics_demo::get_armature_node_count()
}

/// Return the IK target position as "x,y,z". Updated each frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_target_position() -> String {
    kinematics_demo::get_target_position()
}

/// Set the end-effector node index. Applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_end_effector(idx: u32) {
    kinematics_demo::set_end_effector(idx);
}

/// Trigger randomize IK target. Applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn randomize_target() {
    kinematics_demo::randomize_target();
}

/// Set the IK target to the current end-effector position for the active chain. Applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_target_to_ee() {
    kinematics_demo::set_target_to_ee();
}

/// Reset the scene to initial state. Applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn reset_scene() {
    kinematics_demo::reset_scene();
}

/// Load BVH from bytes and replace the active chain on next frame. Returns true if parse/conversion succeeded. Requires `bvh` feature.
#[cfg(all(target_arch = "wasm32", feature = "bvh"))]
#[wasm_bindgen]
pub fn load_bvh_from_bytes(bytes: &[u8]) -> bool {
    kinematics_demo::load_bvh_from_bytes(bytes)
}

/// Enable or disable verbose debug logging. Use `?debug=1` in URL to auto-enable.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_debug_kinematics(enabled: bool) {
    kinematics_demo::set_debug_kinematics(enabled);
}

/// Return whether debug logging is enabled.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_debug_kinematics() -> bool {
    kinematics_demo::get_debug_kinematics()
}

/// Return the last Hessian snapshot when solver is Hessian: object with hessian (Float64Array), size, error, or null.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_hessian_snapshot() -> JsValue {
    kinematics_demo::get_hessian_snapshot()
}
