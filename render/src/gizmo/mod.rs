//! 3D transform gizmo: translate, rotate, scale handles drawn as an overlay.

mod pick;

pub use pick::{pick_gizmo_handle, GizmoAxis};

use crate::backend::{GizmoVertex, Vertex};

/// Gizmo interaction mode: which handles are shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GizmoMode {
    /// Translate: three axis arrows (X red, Y green, Z blue).
    #[default]
    Translate,
    /// Rotate: three axis arcs (same colors).
    Rotate,
    /// Scale: three axis lines with box handles (same colors).
    Scale,
}

/// Result of building a gizmo mesh: either arrows (translate/scale) or rings (rotate).
#[derive(Clone, Debug)]
pub enum GizmoMesh {
    /// Arrow handles; use scene pipeline with [`Vertex`].
    Arrows(Vec<Vertex>, Vec<u16>),
    /// Rotation rings; use gizmo-rotate pipeline with [`GizmoVertex`].
    Rings(Vec<GizmoVertex>, Vec<u16>),
}

/// Axis colors (X, Y, Z) for gizmo handles — bright red, green, blue so the gizmo is clearly visible.
pub const GIZMO_X_COLOR: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
pub const GIZMO_Y_COLOR: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
pub const GIZMO_Z_COLOR: [f32; 4] = [0.0, 0.0, 1.0, 1.0];

/// Default gizmo size (length of each axis).
pub const GIZMO_DEFAULT_SIZE: f32 = 0.5;

/// RGB axis colors for rotation ring vertex color (vec3) — bright red, green, blue.
fn gizmo_x_color_rgb() -> [f32; 3] {
    [1.0, 0.0, 0.0]
}
fn gizmo_y_color_rgb() -> [f32; 3] {
    [0.0, 1.0, 0.0]
}
fn gizmo_z_color_rgb() -> [f32; 3] {
    [0.0, 0.0, 1.0]
}

fn vertex(x: f32, y: f32, z: f32, color: [f32; 4]) -> Vertex {
    Vertex {
        position: [x, y, z],
        uv: [0.0, 0.0],
        color,
        rect_min: [0.0, 0.0],
        rect_max: [0.0, 0.0],
        corner_radius: 0.0,
    }
}

/// Build gizmo mesh (vertices and indices) for the given mode.
/// Returns [`GizmoMesh::Arrows`] for translate/scale (use scene pipeline) or [`GizmoMesh::Rings`] for rotate (use gizmo-rotate pipeline).
#[must_use]
pub fn gizmo_mesh(mode: GizmoMode, size: f32) -> GizmoMesh {
    match mode {
        GizmoMode::Translate => {
            let (v, i) = gizmo_mesh_translate(size);
            GizmoMesh::Arrows(v, i)
        }
        GizmoMode::Rotate => {
            let (v, i) = gizmo_mesh_rotate(size);
            GizmoMesh::Rings(v, i)
        }
        GizmoMode::Scale => {
            let (v, i) = gizmo_mesh_scale(size);
            GizmoMesh::Arrows(v, i)
        }
    }
}

/// Push a box (8 vertices, 12 indices) with CCW winding.
fn push_axis_box(
    vertices: &mut Vec<Vertex>,
    indices: &mut Vec<u16>,
    base: u16,
    corners: [[f32; 3]; 8],
    color: [f32; 4],
) {
    for p in &corners {
        vertices.push(vertex(p[0], p[1], p[2], color));
    }
    let mut quad = |a: u16, b: u16, c: u16, d: u16| {
        indices.push(base + a);
        indices.push(base + b);
        indices.push(base + c);
        indices.push(base + c);
        indices.push(base + d);
        indices.push(base + a);
    };
    quad(0, 1, 2, 3);
    quad(5, 4, 7, 6);
    quad(4, 0, 3, 7);
    quad(1, 5, 6, 2);
    quad(3, 2, 6, 7);
    quad(4, 5, 1, 0);
}

/// Translate: three thin boxes (shafts) along X, Y, Z.
fn gizmo_mesh_translate(size: f32) -> (Vec<Vertex>, Vec<u16>) {
    let t = 0.015_f32;
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    push_axis_box(
        &mut vertices,
        &mut indices,
        0,
        [
            [0.0, -t, -t],
            [size, -t, -t],
            [size, t, -t],
            [0.0, t, -t],
            [0.0, -t, t],
            [size, -t, t],
            [size, t, t],
            [0.0, t, t],
        ],
        GIZMO_X_COLOR,
    );
    push_axis_box(
        &mut vertices,
        &mut indices,
        8,
        [
            [-t, 0.0, -t],
            [t, 0.0, -t],
            [t, size, -t],
            [-t, size, -t],
            [-t, 0.0, t],
            [t, 0.0, t],
            [t, size, t],
            [-t, size, t],
        ],
        GIZMO_Y_COLOR,
    );
    push_axis_box(
        &mut vertices,
        &mut indices,
        16,
        [
            [-t, -t, 0.0],
            [t, -t, 0.0],
            [t, -t, size],
            [-t, -t, size],
            [-t, t, 0.0],
            [t, t, 0.0],
            [t, t, size],
            [-t, t, size],
        ],
        GIZMO_Z_COLOR,
    );

    (vertices, indices)
}

/// Tube radius for rotation rings (relative to size).
const RING_TUBE_RATIO: f32 = 0.06;

/// Rotate: three torus rings (one per axis) with normals and texcoord for ring alpha.
fn gizmo_mesh_rotate(size: f32) -> (Vec<GizmoVertex>, Vec<u16>) {
    const _N_RING: u32 = 32;
    const _N_TUBE: u32 = 16;
    let r = size * RING_TUBE_RATIO;

    let mut vertices: Vec<GizmoVertex> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    // X axis ring: torus in YZ plane, component (1,0,0)
    push_torus_ring(
        &mut vertices,
        &mut indices,
        size,
        r,
        |theta| (0.0, theta.cos(), theta.sin()),
        |theta, phi| {
            let ct = theta.cos();
            let st = theta.sin();
            let cp = phi.cos();
            let sp = phi.sin();
            (sp * r, (size + r * cp) * ct, (size + r * cp) * st)
        },
        |_theta, phi| (phi.cos(), phi.sin()),
        gizmo_x_color_rgb(),
        [1.0, 0.0, 0.0],
    );

    // Y axis ring: torus in XZ plane, component (0,1,0)
    push_torus_ring(
        &mut vertices,
        &mut indices,
        size,
        r,
        |theta| (theta.cos(), 0.0, theta.sin()),
        |theta, phi| {
            let ct = theta.cos();
            let st = theta.sin();
            let cp = phi.cos();
            let sp = phi.sin();
            ((size + r * cp) * ct, sp * r, (size + r * cp) * st)
        },
        |_theta, phi| (phi.cos(), phi.sin()),
        gizmo_y_color_rgb(),
        [0.0, 1.0, 0.0],
    );

    // Z axis ring: torus in XY plane, component (0,0,1)
    push_torus_ring(
        &mut vertices,
        &mut indices,
        size,
        r,
        |theta| (theta.cos(), theta.sin(), 0.0),
        |theta, phi| {
            let ct = theta.cos();
            let st = theta.sin();
            let cp = phi.cos();
            let sp = phi.sin();
            ((size + r * cp) * ct, (size + r * cp) * st, sp * r)
        },
        |_theta, phi| (phi.cos(), phi.sin()),
        gizmo_z_color_rgb(),
        [0.0, 0.0, 1.0],
    );

    (vertices, indices)
}

/// Push vertices and indices for one torus ring. Center curve given by center(theta), position by pos(theta, phi), texcoord by tc(theta, phi).
fn push_torus_ring<FC, FP, FT>(
    vertices: &mut Vec<GizmoVertex>,
    indices: &mut Vec<u16>,
    _size: f32,
    _r: f32,
    center: FC,
    position: FP,
    texcoord: FT,
    color: [f32; 3],
    component: [f32; 3],
) where
    FC: Fn(f32) -> (f32, f32, f32),
    FP: Fn(f32, f32) -> (f32, f32, f32),
    FT: Fn(f32, f32) -> (f32, f32),
{
    const N_RING: u32 = 32;
    const N_TUBE: u32 = 16;
    let base = vertices.len() as u16;
    for i in 0..=N_RING {
        let theta = (i as f32 / N_RING as f32) * 2.0 * std::f32::consts::PI;
        for j in 0..=N_TUBE {
            let phi = (j as f32 / N_TUBE as f32) * 2.0 * std::f32::consts::PI;
            let (px, py, pz) = position(theta, phi);
            let (tx, ty) = texcoord(theta, phi);
            let (cx, cy, cz) = center(theta);
            let dx = px - cx;
            let dy = py - cy;
            let dz = pz - cz;
            let len = (dx * dx + dy * dy + dz * dz).sqrt();
            let (nx, ny, nz) = if len > 1e-6 {
                (dx / len, dy / len, dz / len)
            } else {
                (1.0, 0.0, 0.0)
            };
            vertices.push(GizmoVertex {
                position: [px, py, pz],
                normal: [nx, ny, nz],
                color,
                component,
                texcoord: [tx, ty],
            });
        }
    }
    for i in 0..N_RING {
        for j in 0..N_TUBE {
            let stride = (N_TUBE + 1) as u16;
            let i = i as u16;
            let j = j as u16;
            let a = base + i * stride + j;
            let b = base + i * stride + (j + 1);
            let c = base + (i + 1) * stride + (j + 1);
            let d = base + (i + 1) * stride + j;
            indices.push(a);
            indices.push(b);
            indices.push(c);
            indices.push(c);
            indices.push(d);
            indices.push(a);
        }
    }
}

/// Scale: three axis lines with small box at end (same as translate for now).
fn gizmo_mesh_scale(size: f32) -> (Vec<Vertex>, Vec<u16>) {
    gizmo_mesh_translate(size)
}
