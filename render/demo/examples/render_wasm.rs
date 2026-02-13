//! WASM demo: cube, tetrahedron, cylinder with gizmo and picking.
//!
//! Build: just wasm-render-build
//! HTML must include <canvas id="canvas"></canvas>.
//!
//! Controls: Use the Scene panel to select entities; hold **Shift** and click on object to pick;
//! drag to orbit; hold Ctrl and drag to pan (camera-only, gizmo hidden). JS can call
//! set_demo("default"|"curves") to switch world and get_primitive_tree() for the tree panel.

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    eprintln!("Build for wasm32: just wasm-render-build");
}

#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    render_demo::run_wasm().map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Switch the active scene. Call with "default" or "curves"; applied on next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_demo(name: &str) {
    render_demo::set_demo(name);
}

/// Return the current world's primitive tree as a string (for the tree panel).
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_primitive_tree() -> String {
    render_demo::get_primitive_tree()
}

/// Sentinel value for "no entity selected". Compare get_selected_entity() against this.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_no_entity() -> u32 {
    render_demo::NO_ENTITY
}

/// Return the currently selected entity id, or get_no_entity() if none.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_selected_entity() -> u32 {
    render_demo::get_selected_entity()
}

/// Set the selected entity. Pass get_no_entity() to clear selection.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_selected_entity(id: u32) {
    render_demo::set_selected_entity(id);
}

/// Return material names (newline-separated) for the HTML material dropdown.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_material_names() -> String {
    render_demo::get_material_names()
}

/// Set the material for an entity. No-op if id is invalid.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_entity_material(entity_id: u32, material_name: &str) {
    render_demo::set_entity_material(entity_id, material_name);
}

/// Set the primitive for an entity. Name: "cube", "sphere", "line", "bezier", etc.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_entity_primitive(entity_id: u32, primitive_name: &str) {
    render_demo::set_entity_primitive(entity_id, primitive_name);
}

/// Remove the currently selected entity. Root cannot be removed.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn remove_selected_entity() {
    render_demo::remove_selected_entity();
}

/// Return the current gizmo mode: "translate", "rotate", or "scale". Gizmo is shown only when an entity is picked.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_gizmo_mode() -> String {
    render_demo::get_gizmo_mode()
}

/// Set the gizmo mode: "translate", "rotate", or "scale". Changes which transform handles are shown for the selected entity.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_gizmo_mode(mode: &str) {
    render_demo::set_gizmo_mode(mode);
}

/// Add a new entity as child of the root with the given primitive (e.g. "cube", "sphere"). Applied next frame; the new entity becomes selected.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn add_entity(primitive_name: &str) {
    render_demo::add_entity(primitive_name);
}

/// Local position of the selected entity as "x,y,z". Empty if none selected.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_selected_entity_local_position() -> String {
    render_demo::get_selected_entity_local_position()
}

/// World position of the selected entity as "x,y,z". Empty if none selected.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_selected_entity_world_position() -> String {
    render_demo::get_selected_entity_world_position()
}

/// Set local position of an entity. Applied next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_entity_local_position(entity_id: u32, x: f32, y: f32, z: f32) {
    render_demo::set_entity_local_position(entity_id, x, y, z);
}

/// Set world position of an entity. Applied next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_entity_world_position(entity_id: u32, x: f32, y: f32, z: f32) {
    render_demo::set_entity_world_position(entity_id, x, y, z);
}

/// Local rotation (roll, pitch, yaw in radians) of the selected entity as "r,p,y". Empty if none selected.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_selected_entity_local_rotation() -> String {
    render_demo::get_selected_entity_local_rotation()
}

/// Local scale of the selected entity as "x,y,z". Empty if none selected.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn get_selected_entity_local_scale() -> String {
    render_demo::get_selected_entity_local_scale()
}

/// Set local rotation of an entity. Pass roll, pitch, yaw in radians. Applied next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_entity_local_rotation(entity_id: u32, roll: f32, pitch: f32, yaw: f32) {
    render_demo::set_entity_local_rotation(entity_id, roll, pitch, yaw);
}

/// Set local scale of an entity. Applied next frame.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn set_entity_local_scale(entity_id: u32, x: f32, y: f32, z: f32) {
    render_demo::set_entity_local_scale(entity_id, x, y, z);
}
