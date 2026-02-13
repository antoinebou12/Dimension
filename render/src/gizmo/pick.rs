//! Gizmo handle picking: ray-cast against gizmo geometry to determine which axis is under the cursor.

use crate::backend::Camera3d;
use crate::cull::{ray_aabb, Aabb};
use crate::gizmo::GizmoMode;
use collision::{ray_sphere, Sphere};
use mathlib::cg::{screen_to_view_ray, transform_point, vector3, vector4_from_point};
use mathlib::math3d::{matrix4f_inverse, Matrix4f};

/// Axis of the gizmo that can be picked (X, Y, or Z).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GizmoAxis {
    X,
    Y,
    Z,
}

/// Thin box half-extent for translate/scale handles (matches gizmo mesh).
const GIZMO_BOX_THICKNESS: f32 = 0.015;

/// Number of segments to sample per rotation ring for picking.
const RING_PICK_SEGMENTS: u32 = 32;

/// Tube radius for rotation rings (relative to size); matches gizmo mesh.
const RING_TUBE_RATIO: f32 = 0.06;

/// Returns the ray origin and normalized direction in world space, or `None` if perspective is unavailable.
fn screen_ray_world(
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

/// Transform ray from world to local space (origin and direction).
fn ray_world_to_local(
    origin_world: &[f32; 3],
    dir_world: &[f32; 3],
    world_to_local: &Matrix4f,
) -> ([f32; 3], [f32; 3]) {
    let origin_local = transform_point(
        world_to_local,
        &vector3(origin_world[0], origin_world[1], origin_world[2]),
    );
    let end_world = vector3(
        origin_world[0] + dir_world[0],
        origin_world[1] + dir_world[1],
        origin_world[2] + dir_world[2],
    );
    let end_local = transform_point(world_to_local, &end_world);
    let dx = end_local.get(0) - origin_local.get(0);
    let dy = end_local.get(1) - origin_local.get(1);
    let dz = end_local.get(2) - origin_local.get(2);
    let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-9);
    let origin_arr = [
        origin_local.get(0),
        origin_local.get(1),
        origin_local.get(2),
    ];
    let dir_arr = [dx / len, dy / len, dz / len];
    (origin_arr, dir_arr)
}

/// Local AABBs for the three translate/scale axis boxes (same geometry as gizmo mesh).
fn translate_scale_boxes(size: f32) -> [(GizmoAxis, Aabb); 3] {
    let t = GIZMO_BOX_THICKNESS;
    [
        (GizmoAxis::X, Aabb::new([0.0, -t, -t], [size, t, t])),
        (GizmoAxis::Y, Aabb::new([-t, 0.0, -t], [t, size, t])),
        (GizmoAxis::Z, Aabb::new([-t, -t, 0.0], [t, t, size])),
    ]
}

/// Pick translate or scale handles: ray vs three thin boxes in local space.
fn pick_arrows(
    origin_local: &[f32; 3],
    dir_local: &[f32; 3],
    size: f32,
) -> Option<(GizmoAxis, f32)> {
    let boxes = translate_scale_boxes(size);
    let mut best_t: Option<f32> = None;
    let mut best_axis: Option<GizmoAxis> = None;
    for (axis, aabb) in boxes {
        if let Some(t) = ray_aabb(origin_local, dir_local, &aabb) {
            if t > 1e-5 {
                let is_better = best_t.map_or(true, |b| t < b);
                if is_better {
                    best_t = Some(t);
                    best_axis = Some(axis);
                }
            }
        }
    }
    best_axis.map(|a| (a, best_t.unwrap()))
}

/// Ring center on the circle for axis X (YZ plane), Y (XZ), or Z (XY).
fn ring_center(axis: GizmoAxis, size: f32, theta: f32) -> [f32; 3] {
    let c = theta.cos() * size;
    let s = theta.sin() * size;
    match axis {
        GizmoAxis::X => [0.0, c, s],
        GizmoAxis::Y => [c, 0.0, s],
        GizmoAxis::Z => [c, s, 0.0],
    }
}

/// Pick rotate handles: sample each ring as spheres along the center circle.
fn pick_rings(
    origin_local: &[f32; 3],
    dir_local: &[f32; 3],
    size: f32,
) -> Option<(GizmoAxis, f32)> {
    let r = size * RING_TUBE_RATIO;
    let mut best_t: Option<f32> = None;
    let mut best_axis: Option<GizmoAxis> = None;

    let two_pi = 2.0 * std::f32::consts::PI;
    for axis in [GizmoAxis::X, GizmoAxis::Y, GizmoAxis::Z] {
        for i in 0..RING_PICK_SEGMENTS {
            let theta = (i as f32 / RING_PICK_SEGMENTS as f32) * two_pi;
            let center = ring_center(axis, size, theta);
            let sphere = Sphere { center, radius: r };
            if let Some(t) = ray_sphere(origin_local, dir_local, &sphere) {
                if t > 1e-5 {
                    let is_better = best_t.map_or(true, |b| t < b);
                    if is_better {
                        best_t = Some(t);
                        best_axis = Some(axis);
                    }
                }
            }
        }
    }
    best_axis.map(|a| (a, best_t.unwrap()))
}

/// Cast a ray from screen coordinates and return which gizmo axis (if any) is hit.
///
/// Uses the same screen-to-world ray as entity picking. The gizmo is assumed to be
/// at the given world matrix (e.g. selected entity's world matrix). Returns the axis
/// with the closest hit.
///
/// # Arguments
/// * `camera` - Camera with perspective (viewport and view matrix).
/// * `gizmo_world_matrix` - World transform of the gizmo (e.g. selected entity).
/// * `mode` - Current gizmo mode (Translate, Rotate, or Scale).
/// * `size` - Gizmo size (axis length); use [`GIZMO_DEFAULT_SIZE`] if unsure.
/// * `screen_x` - Cursor x in viewport pixels.
/// * `screen_y` - Cursor y in viewport pixels.
#[must_use]
pub fn pick_gizmo_handle(
    camera: &impl Camera3d,
    gizmo_world_matrix: &Matrix4f,
    mode: GizmoMode,
    size: f32,
    screen_x: f32,
    screen_y: f32,
) -> Option<GizmoAxis> {
    let (origin_world, dir_world) = screen_ray_world(camera, screen_x, screen_y)?;
    let world_to_local = matrix4f_inverse(gizmo_world_matrix);
    let (origin_local, dir_local) = ray_world_to_local(&origin_world, &dir_world, &world_to_local);

    let hit = match mode {
        GizmoMode::Translate | GizmoMode::Scale => pick_arrows(&origin_local, &dir_local, size),
        GizmoMode::Rotate => pick_rings(&origin_local, &dir_local, size),
    };
    hit.map(|(axis, _)| axis)
}
