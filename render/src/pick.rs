//! Picking: ray cast from screen to select entity.
//!
//! Uses mathlib for screen-to-view ray and view-to-world transform. Perspective projection only in v1.
//! Uses BSP for ray traversal (candidates) and [`ray_aabb`](crate::cull::ray_aabb) for early AABB reject.
//! Triangle/quad meshes use Möller–Trumbore ray-triangle; line and curve primitives use ray-segment
//! ([`Primitive3D::LineSegment`]) or sampled segments (Bézier, Hermite, B-spline).
//!
//! [`Bezier`]: crate::scene::Primitive3D::Bezier
//! [`Hermite`]: crate::scene::Primitive3D::Hermite
//! [`BSpline`]: crate::scene::Primitive3D::BSpline

use crate::backend::math_prep::world_matrix;
use crate::backend::primitive_mesh;
use crate::backend::Camera3d;
use crate::cull::{primitive_aabb, world_aabb};
use crate::scene::{EntityId, World};
use crate::spatial::BspTree;
use collision::{ray_segment, ray_triangle};
use mathlib::cg::{screen_to_view_ray, transform_point, vector3, vector4_from_point};
use mathlib::math3d::{matrix4f_inverse, Matrix4f};

/// Returns the world-space ray (origin and normalized direction) from screen coordinates.
/// Use for gizmo drag and other ray-based interaction. Returns `None` if the camera is orthographic.
#[must_use]
pub fn screen_ray_to_world(
    camera: &impl Camera3d,
    screen_x: f32,
    screen_y: f32,
) -> Option<([f32; 3], [f32; 3])> {
    let proj = camera.perspective_params()?;
    let view = camera.view_matrix();
    let view_inv = matrix4f_inverse(&view);

    let (origin_view, dir_view) = screen_to_view_ray(
        &proj,
        screen_x,
        screen_y,
        camera.viewport_width(),
        camera.viewport_height(),
    );

    let origin_view_4 =
        vector4_from_point(origin_view.get(0), origin_view.get(1), origin_view.get(2));
    let origin_world_4 = &view_inv * &origin_view_4;
    let end_view = vector3(
        origin_view.get(0) + dir_view.get(0),
        origin_view.get(1) + dir_view.get(1),
        origin_view.get(2) + dir_view.get(2),
    );
    let end_view_4 = vector4_from_point(end_view.get(0), end_view.get(1), end_view.get(2));
    let end_world_4 = &view_inv * &end_view_4;
    let origin_world = [
        origin_world_4.get(0) / origin_world_4.get(3).max(1e-9),
        origin_world_4.get(1) / origin_world_4.get(3).max(1e-9),
        origin_world_4.get(2) / origin_world_4.get(3).max(1e-9),
    ];
    let dir_world = vector3(
        end_world_4.get(0) / end_world_4.get(3).max(1e-9) - origin_world[0],
        end_world_4.get(1) / end_world_4.get(3).max(1e-9) - origin_world[1],
        end_world_4.get(2) / end_world_4.get(3).max(1e-9) - origin_world[2],
    );
    let dir_len =
        (dir_world.get(0).powi(2) + dir_world.get(1).powi(2) + dir_world.get(2).powi(2)).sqrt();
    if dir_len < 1e-9 {
        return None;
    }
    let dir_world = [
        dir_world.get(0) / dir_len,
        dir_world.get(1) / dir_len,
        dir_world.get(2) / dir_len,
    ];
    Some((origin_world, dir_world))
}

/// Cast a ray from screen coordinates and return the first hit entity, if any.
///
/// Uses perspective projection only; returns `None` if the camera is orthographic or if no entity is hit.
///
/// # Arguments
/// * `world` - Scene world (entities and primitives).
/// * `camera` - Camera implementing [`Camera3d`] (must have perspective projection for picking).
/// * `screen_x` - Cursor x in pixel coordinates.
/// * `screen_y` - Cursor y in pixel coordinates.
///
/// # Examples
///
/// ```ignore
/// if let Some(id) = render::pick_entity(engine.world(), engine.camera(), x, y) {
///     engine.set_selected_entity(Some(id));
/// }
/// ```
#[must_use]
pub fn pick_entity(
    world: &World,
    camera: &impl Camera3d,
    screen_x: f32,
    screen_y: f32,
) -> Option<EntityId> {
    let (origin_world, dir_world) = screen_ray_to_world(camera, screen_x, screen_y)?;
    let origin_world = vector3(origin_world[0], origin_world[1], origin_world[2]);
    let dir_world = vector3(dir_world[0], dir_world[1], dir_world[2]);

    let tree = world.tree();
    let n = tree.num_nodes();
    let mut world_matrices: Vec<Matrix4f> = Vec::with_capacity(n);
    world_matrices.resize(n, mathlib::cg::matrix4f_identity());
    for id in world.entities_dfs() {
        let parent_world = world
            .parent(id)
            .map(|p| world_matrices[p.0].clone())
            .unwrap_or_else(mathlib::cg::matrix4f_identity);
        world_matrices[id.0] = world_matrix(tree, id.0, &parent_world);
    }

    let entity_aabbs: Vec<(EntityId, crate::cull::Aabb)> = world
        .entities_dfs()
        .into_iter()
        .filter_map(|id| {
            let node = world.get(id)?;
            let prim = node.primitive?;
            let model_aabb = primitive_aabb(&prim);
            let world_mat = &world_matrices[id.0];
            Some((id, world_aabb(&model_aabb, world_mat)))
        })
        .collect();
    let bsp = BspTree::build(&entity_aabbs);
    let origin_arr = [
        origin_world.get(0),
        origin_world.get(1),
        origin_world.get(2),
    ];
    let dir_arr = [dir_world.get(0), dir_world.get(1), dir_world.get(2)];
    let candidates = bsp.intersect_ray(&origin_arr, &dir_arr);

    let mut best_t: Option<f32> = None;
    let mut best_entity: Option<EntityId> = None;

    for id in candidates {
        let node = match world.get(id) {
            Some(n) => n,
            None => continue,
        };
        let prim = match node.primitive {
            Some(p) => p,
            None => continue,
        };

        let world_mat = &world_matrices[id.0];
        let model_inv = matrix4f_inverse(world_mat);
        let origin_model = transform_point(&model_inv, &origin_world);
        let end_world = vector3(
            origin_world.get(0) + dir_world.get(0),
            origin_world.get(1) + dir_world.get(1),
            origin_world.get(2) + dir_world.get(2),
        );
        let end_model = transform_point(&model_inv, &end_world);
        let dir_model = vector3(
            end_model.get(0) - origin_model.get(0),
            end_model.get(1) - origin_model.get(1),
            end_model.get(2) - origin_model.get(2),
        );
        let d_len =
            (dir_model.get(0).powi(2) + dir_model.get(1).powi(2) + dir_model.get(2).powi(2)).sqrt();
        if d_len < 1e-9 {
            continue;
        }

        let origin_model_arr = [
            origin_model.get(0),
            origin_model.get(1),
            origin_model.get(2),
        ];
        let dir_model_arr = [
            dir_model.get(0) / d_len,
            dir_model.get(1) / d_len,
            dir_model.get(2) / d_len,
        ];

        let (vertices, indices) = primitive_mesh(&prim);
        if prim.is_line_list() {
            for chunk in indices.chunks(2) {
                if chunk.len() < 2 {
                    continue;
                }
                let i0 = chunk[0] as usize;
                let i1 = chunk[1] as usize;
                if i0 >= vertices.len() || i1 >= vertices.len() {
                    continue;
                }
                let a = [
                    vertices[i0].position[0],
                    vertices[i0].position[1],
                    vertices[i0].position[2],
                ];
                let b = [
                    vertices[i1].position[0],
                    vertices[i1].position[1],
                    vertices[i1].position[2],
                ];
                if let Some(t) = ray_segment(&origin_model_arr, &dir_model_arr, &a, &b) {
                    let is_better = best_t.map_or(true, |best| t < best);
                    if is_better {
                        best_t = Some(t);
                        best_entity = Some(id);
                    }
                }
            }
        } else {
            for chunk in indices.chunks(3) {
                if chunk.len() < 3 {
                    continue;
                }
                let i0 = chunk[0] as usize;
                let i1 = chunk[1] as usize;
                let i2 = chunk[2] as usize;
                if i0 >= vertices.len() || i1 >= vertices.len() || i2 >= vertices.len() {
                    continue;
                }
                let v0 = [
                    vertices[i0].position[0],
                    vertices[i0].position[1],
                    vertices[i0].position[2],
                ];
                let v1 = [
                    vertices[i1].position[0],
                    vertices[i1].position[1],
                    vertices[i1].position[2],
                ];
                let v2 = [
                    vertices[i2].position[0],
                    vertices[i2].position[1],
                    vertices[i2].position[2],
                ];
                if let Some(t) = ray_triangle(&origin_model_arr, &dir_model_arr, &v0, &v1, &v2) {
                    if t > 1e-5 {
                        let is_better = best_t.map_or(true, |best| t < best);
                        if is_better {
                            best_t = Some(t);
                            best_entity = Some(id);
                        }
                    }
                }
            }
        }
    }

    best_entity
}
