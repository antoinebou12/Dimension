//! render-demo — 2D/3D demo: cube, tetrahedron, cylinder, gizmo, curves (winit + WASM).
//!
//! Run native: `cargo run -p render-demo --example render_native`
//! Run WASM: build with `just wasm-render-build`, then serve and open `wasm-demo/index.html`.

mod aabb2d;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(target_arch = "wasm32")]
mod wasm;

/// Run on native (winit).
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    native::run(render::RunDemo::Default)
}

/// Run on native with curves scene preset.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native_curves() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    native::run(render::RunDemo::Curves)
}

/// Run on native with 2D AABB/Circle collision demo (orthographic, per-frame update).
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native_aabb2d() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    native::run(render::RunDemo::Aabb2d)
}

/// Run on WASM (canvas, requestAnimationFrame).
#[cfg(target_arch = "wasm32")]
pub fn run_wasm() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    wasm::run(render::RunDemo::Default)
}

/// Run on WASM with curves scene preset.
#[cfg(target_arch = "wasm32")]
pub fn run_wasm_curves() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    wasm::run(render::RunDemo::Curves)
}

/// Run on WASM with 2D AABB/Circle collision demo.
#[cfg(target_arch = "wasm32")]
pub fn run_wasm_aabb2d() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    wasm::run(render::RunDemo::Aabb2d)
}

/// Set the active scene from JS. Call with `"default"` or `"curves"`; applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_demo(name: &str) {
    wasm::set_demo(name);
}

/// Get the current world's primitive tree as a string for the tree panel.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_primitive_tree() -> String {
    wasm::get_primitive_tree()
}

/// Sentinel for "no entity selected" in JS-facing API (`u32::MAX`).
#[cfg(target_arch = "wasm32")]
pub const NO_ENTITY: u32 = wasm::NO_ENTITY;

/// Return the currently selected entity id, or [`NO_ENTITY`] if none. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_selected_entity() -> u32 {
    wasm::get_selected_entity()
}

/// Set the selected entity. Pass [`NO_ENTITY`] to clear selection. Applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_selected_entity(id: u32) {
    wasm::set_selected_entity(id);
}

/// Return material names (newline-separated) for the HTML material dropdown. Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_material_names() -> String {
    wasm::get_material_names()
}

/// Set the material for an entity. No-op if id is invalid. Applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_entity_material(entity_id: u32, material_name: &str) {
    wasm::set_entity_material(entity_id, material_name);
}

/// Set the primitive for an entity. Pass a name like "cube", "sphere", "line", "bezier".
/// No-op if id or name is invalid. Applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_entity_primitive(entity_id: u32, primitive_name: &str) {
    wasm::set_entity_primitive(entity_id, primitive_name);
}

/// Remove the currently selected entity (one at a time). Root cannot be removed. Applied on next frame.
#[cfg(target_arch = "wasm32")]
pub fn remove_selected_entity() {
    wasm::remove_selected_entity();
}

/// Return the current gizmo mode: "translate", "rotate", or "scale". Updated each frame.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_gizmo_mode() -> String {
    wasm::get_gizmo_mode()
}

/// Set the gizmo mode for the next frame. Pass "translate", "rotate", or "scale".
#[cfg(target_arch = "wasm32")]
pub fn set_gizmo_mode(mode: &str) {
    wasm::set_gizmo_mode(mode);
}

/// Add a new entity as child of the root with the given primitive (e.g. "cube", "sphere"). Applied next frame; the new entity becomes selected.
#[cfg(target_arch = "wasm32")]
pub fn add_entity(primitive_name: &str) {
    wasm::add_entity(primitive_name);
}

/// Local position of the selected entity as "x,y,z". Empty if none selected.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_selected_entity_local_position() -> String {
    wasm::get_selected_entity_local_position()
}

/// World position of the selected entity as "x,y,z". Empty if none selected.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_selected_entity_world_position() -> String {
    wasm::get_selected_entity_world_position()
}

/// Set local position of an entity. Applied next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_entity_local_position(entity_id: u32, x: f32, y: f32, z: f32) {
    wasm::set_entity_local_position(entity_id, x, y, z);
}

/// Set world position of an entity. Applied next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_entity_world_position(entity_id: u32, x: f32, y: f32, z: f32) {
    wasm::set_entity_world_position(entity_id, x, y, z);
}

/// Local rotation (roll, pitch, yaw in radians) of the selected entity as "r,p,y". Empty if none selected.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_selected_entity_local_rotation() -> String {
    wasm::get_selected_entity_local_rotation()
}

/// Local scale of the selected entity as "x,y,z". Empty if none selected.
#[cfg(target_arch = "wasm32")]
#[must_use]
pub fn get_selected_entity_local_scale() -> String {
    wasm::get_selected_entity_local_scale()
}

/// Set local rotation of an entity. Pass roll, pitch, yaw in radians. Applied next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_entity_local_rotation(entity_id: u32, roll: f32, pitch: f32, yaw: f32) {
    wasm::set_entity_local_rotation(entity_id, roll, pitch, yaw);
}

/// Set local scale of an entity. Applied next frame.
#[cfg(target_arch = "wasm32")]
pub fn set_entity_local_scale(entity_id: u32, x: f32, y: f32, z: f32) {
    wasm::set_entity_local_scale(entity_id, x, y, z);
}
