//! CPU math preparation using mathlib (MVP, world matrix chain).

use mathlib::cg::{matrix4f_identity, matrix4f_to_array, model_view_projection};
use mathlib::graph::Tree;
use mathlib::math3d::Matrix4f;

use crate::scene::NodeData;

/// Identity matrix as array (column-major) for uniform buffer init.
#[must_use]
pub fn identity_matrix() -> [f32; 16] {
    matrix4f_to_array(&matrix4f_identity())
}

/// Build world matrix for a node (parent_world * local_model).
#[must_use]
pub fn world_matrix(tree: &Tree<NodeData>, node_id: usize, parent_world: &Matrix4f) -> Matrix4f {
    let node = &tree.nodes[node_id];
    let local = node.data.transform.to_model_matrix();
    parent_world * &local
}

/// Build MVP matrix for a draw call.
#[must_use]
pub fn mvp_matrix(model: &Matrix4f, view: &Matrix4f, projection: &Matrix4f) -> Matrix4f {
    model_view_projection(model, view, projection)
}

/// Build model-view matrix (view * model) for transforming positions to view space.
#[must_use]
pub fn model_view_matrix(model: &Matrix4f, view: &Matrix4f) -> Matrix4f {
    view * model
}

/// Frame uniform size (WGSL std140): view (64) + proj (64) + light_dir (16) + colormap_mode/exposure/gamma/ambient (16) +
/// num_lights (4) + pad to 16B (12) + _pad vec3 (12) + pad to 16B (4) + light2_dir (16) + selection_time (4) +
/// implicit pad to 16B (12) + _pad_selection vec3 (12) + struct trailing pad (4) = 240.
pub(crate) const SCENE_FRAME_UNIFORM_SIZE: usize = 240;

/// Object uniform size: mvp (64) + model_view (64) + material_mode+pad (16) + entity_color (16) + selected (4) + _pad_selection (12) = 176.
/// Used for legacy uniform layout; scene pass uses instanced vertex buffer instead.
pub(crate) const SCENE_OBJECT_UNIFORM_SIZE: usize = 176;

/// Legacy single-block size (kept for reference; use frame + object instead).
pub(crate) const SCENE_UNIFORM_SIZE: usize = 208;

/// Default direction toward light in view space (normalized). Top-right-front.
fn default_light_dir_view(strength: f32) -> [f32; 4] {
    let x = 0.577_350_27_f32; // 1/sqrt(3)
    [x, x, x, strength]
}

/// Build light direction vec4 (xyz normalized, w = strength). If `dir` is zero, uses default.
#[must_use]
fn light_dir_view(dir: [f32; 3], strength: f32) -> [f32; 4] {
    let len_sq = dir[0] * dir[0] + dir[1] * dir[1] + dir[2] * dir[2];
    let (x, y, z) = if len_sq > 1e-10_f32 {
        let len = len_sq.sqrt();
        (dir[0] / len, dir[1] / len, dir[2] / len)
    } else {
        let x = 0.577_350_27_f32;
        (x, x, x)
    };
    [x, y, z, strength]
}

/// Build frame uniform bytes for GPU upload (written once per frame).
/// Layout (WGSL std140): view (64) + proj (64) + light_dir (16) + colormap_mode (4) + exposure (4) + gamma (4) + ambient (4) +
/// num_lights (4) + pad to 16B (12) + _pad vec3 (12) + pad to 16B (4) + light2_dir (16) + selection_time (4) +
/// implicit pad to 16B (12) + _pad_selection vec3 (12) + struct trailing pad (4) = 240.
///
/// `light_direction` is in view space (xyz); if zero length, a default top-right-front direction is used.
/// `light2` is optional second light (xyz direction in view space, w = strength); `None` means num_lights = 1.
/// `selection_time` is time in seconds for selection pulse (blinking highlight).
#[must_use]
pub fn scene_frame_uniform_bytes(
    view: &Matrix4f,
    proj: &Matrix4f,
    light_direction: [f32; 3],
    lighting_strength: f32,
    light2: Option<[f32; 4]>,
    colormap_mode: u32,
    exposure: f32,
    gamma: f32,
    ambient_intensity: f32,
    selection_time: f32,
) -> Vec<u8> {
    let view_arr = matrix4f_to_array(view);
    let proj_arr = matrix4f_to_array(proj);
    let light = light_dir_view(light_direction, lighting_strength);
    let num_lights = if light2.is_some() { 2u32 } else { 1u32 };
    let light2_dir = light2.unwrap_or([0.0, 0.0, 0.0, 0.0]);
    let mut bytes = Vec::with_capacity(SCENE_FRAME_UNIFORM_SIZE);
    bytes.extend_from_slice(bytemuck::cast_slice(&view_arr));
    bytes.extend_from_slice(bytemuck::cast_slice(&proj_arr));
    bytes.extend_from_slice(bytemuck::cast_slice(&light));
    bytes.extend_from_slice(bytemuck::bytes_of(&colormap_mode));
    bytes.extend_from_slice(bytemuck::bytes_of(&exposure));
    bytes.extend_from_slice(bytemuck::bytes_of(&gamma));
    bytes.extend_from_slice(bytemuck::bytes_of(&ambient_intensity));
    bytes.extend_from_slice(bytemuck::bytes_of(&num_lights));
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 3])); // pad so _pad vec3 is 16-byte aligned
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 3])); // _pad vec3<u32>
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 1])); // pad so light2_dir vec4 is 16-byte aligned
    bytes.extend_from_slice(bytemuck::cast_slice(&light2_dir));
    bytes.extend_from_slice(bytemuck::bytes_of(&selection_time));
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 3])); // implicit pad to 16B (vec3 alignment)
    bytes.extend_from_slice(bytemuck::bytes_of(&[0.0f32; 3])); // _pad_selection vec3<f32>
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 1])); // struct trailing pad
    bytes
}

/// Build per-object uniform bytes for GPU upload (dynamic offset per draw).
/// Layout (std140): mvp (64) + model_view (64) + material_mode (4) + pad (12) + entity_color (16) + selected (4) + _pad_selection (12) = 176.
/// Caller may need to pad to device min_uniform_buffer_offset_alignment when using dynamic offsets.
#[must_use]
pub fn scene_object_uniform_bytes(
    mvp: &Matrix4f,
    model_view: &Matrix4f,
    material_mode: u32,
    entity_color: [f32; 4],
    selected: u32,
) -> Vec<u8> {
    let mvp_arr = matrix4f_to_array(mvp);
    let mv_arr = matrix4f_to_array(model_view);
    let mut bytes = Vec::with_capacity(SCENE_OBJECT_UNIFORM_SIZE);
    bytes.extend_from_slice(bytemuck::cast_slice(&mvp_arr));
    bytes.extend_from_slice(bytemuck::cast_slice(&mv_arr));
    bytes.extend_from_slice(bytemuck::bytes_of(&material_mode));
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 3])); // pad to vec4
    bytes.extend_from_slice(bytemuck::bytes_of(&entity_color));
    bytes.extend_from_slice(bytemuck::bytes_of(&selected));
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 3])); // _pad_selection vec3<u32>
    bytes
}

/// Build scene uniform bytes for GPU upload (legacy single-block; used for init only).
/// Layout (std140): mvp (64) + model_view (64) + light_dir (16) + colormap_mode/exposure/gamma/pad (16) +
/// material_mode (4) + pad (12) + entity_color (16) + _pad2 (8) + struct padding (8) = 208.
#[must_use]
pub fn scene_uniform_bytes(
    mvp: &Matrix4f,
    model_view: &Matrix4f,
    lighting_strength: f32,
    colormap_mode: u32,
    exposure: f32,
    gamma: f32,
    material_mode: u32,
    entity_color: [f32; 4],
) -> Vec<u8> {
    let mvp_arr = matrix4f_to_array(mvp);
    let mv_arr = matrix4f_to_array(model_view);
    let light = default_light_dir_view(lighting_strength);
    let mut bytes = Vec::with_capacity(SCENE_UNIFORM_SIZE);
    bytes.extend_from_slice(bytemuck::cast_slice(&mvp_arr));
    bytes.extend_from_slice(bytemuck::cast_slice(&mv_arr));
    bytes.extend_from_slice(bytemuck::cast_slice(&light));
    bytes.extend_from_slice(bytemuck::bytes_of(&colormap_mode));
    bytes.extend_from_slice(bytemuck::bytes_of(&exposure));
    bytes.extend_from_slice(bytemuck::bytes_of(&gamma));
    bytes.extend_from_slice(bytemuck::bytes_of(&0u32)); // padding
    bytes.extend_from_slice(bytemuck::bytes_of(&material_mode));
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 3])); // pad 12 bytes for vec4 alignment
    bytes.extend_from_slice(bytemuck::bytes_of(&entity_color));
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 2])); // _pad2
    bytes.extend_from_slice(bytemuck::bytes_of(&[0u32; 2])); // struct padding to 208
    bytes
}
