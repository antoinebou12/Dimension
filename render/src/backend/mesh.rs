//! Primitive meshes (2D and 3D). Uses mathlib for positions and trig; SIMD for batch color fill.

use super::vertex::Vertex;
use crate::scene::{Primitive, Primitive2D, Primitive3D};
use mathlib::math::curve::{bezier_curve, bspline_curve, hermite_curve, linear_curve};
use mathlib::trig::{cos_scalar, sin_scalar};

/// Fill a slice with 1.0 (white). Uses mathlib SIMD when the `simd` feature is enabled.
fn fill_slice_white(slice: &mut [f32]) {
    #[cfg(feature = "simd")]
    {
        mathlib::cpu::simd::set_zero_f32(slice);
        let ones = vec![1.0_f32; slice.len()];
        mathlib::cpu::simd::add_f32(&ones, slice, slice);
    }
    #[cfg(not(feature = "simd"))]
    {
        slice.fill(1.0);
    }
}

/// Default number of segments for circle, ellipse, and cylinder.
const DEFAULT_CIRCLE_SEGMENTS: u32 = 32;
/// Default number of segments for curve primitives (line/Bezier/Hermite/B-spline).
const DEFAULT_CURVE_SEGMENTS: u32 = 32;

/// Default ellipse radii (rx, ry) when building ellipse mesh.
const DEFAULT_ELLIPSE_RX: f32 = 0.5;
const DEFAULT_ELLIPSE_RY: f32 = 0.25;

/// Default cylinder radius and half-height.
const DEFAULT_CYLINDER_RADIUS: f32 = 0.5;
const DEFAULT_CYLINDER_HALF_HEIGHT: f32 = 0.5;

/// Default sphere segments (latitude and longitude).
const DEFAULT_SPHERE_SEGMENTS: u32 = 24;
/// Default cone/capsule circle segments.
const DEFAULT_CONE_SEGMENTS: u32 = 32;
/// Default capsule half-height (cylinder part only, excluding caps).
const DEFAULT_CAPSULE_HALF_HEIGHT: f32 = 0.4;

/// Build vertices and indices for a primitive.
///
/// Dispatches to 2D or 3D mesh generation. All 2D shapes use z = 0.
///
/// # Panics
/// Panics if the primitive is unknown (exhaustive match makes this unreachable).
#[must_use]
pub fn primitive_mesh(primitive: &Primitive) -> (Vec<Vertex>, Vec<u16>) {
    match primitive {
        Primitive::TwoD(d) => mesh_2d(d),
        Primitive::ThreeD(d) => mesh_3d(d),
    }
}

fn mesh_2d(p: &Primitive2D) -> (Vec<Vertex>, Vec<u16>) {
    match p {
        Primitive2D::Quad => fullscreen_quad(),
        Primitive2D::Square => square_2d(),
        Primitive2D::Circle => circle_2d(),
        Primitive2D::Ellipse => ellipse_2d(),
        Primitive2D::Triangle => triangle_2d(),
    }
}

fn mesh_3d(p: &Primitive3D) -> (Vec<Vertex>, Vec<u16>) {
    match p {
        Primitive3D::Quad => quad_3d(),
        Primitive3D::Triangle => triangle_3d(),
        Primitive3D::Cube => cube_3d(),
        Primitive3D::Tetrahedron => tetrahedron_3d(),
        Primitive3D::Cylinder => cylinder_3d(),
        Primitive3D::Sphere => sphere_3d(),
        Primitive3D::Cone => cone_3d(),
        Primitive3D::Capsule => capsule_3d(),
        Primitive3D::LineSegment { start, end } => {
            curve_line_list(1, |t| linear_curve(start.0, end.0, t))
        }
        Primitive3D::Bezier { control_points } => {
            let p0 = control_points[0].0;
            let p1 = control_points[1].0;
            let p2 = control_points[2].0;
            let p3 = control_points[3].0;
            curve_line_list(DEFAULT_CURVE_SEGMENTS, |t| bezier_curve(p0, p1, p2, p3, t))
        }
        Primitive3D::Hermite { p0, p1, m0, m1 } => curve_line_list(DEFAULT_CURVE_SEGMENTS, |t| {
            hermite_curve(p0.0, p1.0, m0.0, m1.0, t)
        }),
        Primitive3D::BSpline { control_points } => {
            let p0 = control_points[0].0;
            let p1 = control_points[1].0;
            let p2 = control_points[2].0;
            let p3 = control_points[3].0;
            curve_line_list(DEFAULT_CURVE_SEGMENTS, |t| bspline_curve(p0, p1, p2, p3, t))
        }
    }
}

/// Build line-list mesh from a curve: sample at (segments+1) points, output segments as line pairs.
fn curve_line_list<F>(segments: u32, sample: F) -> (Vec<Vertex>, Vec<u16>)
where
    F: Fn(f32) -> [f32; 3],
{
    let n = segments.max(1);
    let num_verts = (n + 1) as usize;
    let mut vertices = Vec::with_capacity(num_verts);
    for i in 0..=n {
        let t = (i as f32) / (n as f32);
        let pos = sample(t);
        vertices.push(vertex(pos[0], pos[1], pos[2], 0.0, 0.0, CURVE_LINE_COLOR));
    }
    let mut indices = Vec::with_capacity(2 * (n as usize));
    for i in 0..n {
        indices.push(i as u16);
        indices.push(i as u16 + 1);
    }
    (vertices, indices)
}

#[inline]
fn vertex(x: f32, y: f32, z: f32, u: f32, v: f32, color: [f32; 4]) -> Vertex {
    Vertex {
        position: [x, y, z],
        uv: [u, v],
        color,
        rect_min: [0.0, 0.0],
        rect_max: [0.0, 0.0],
        corner_radius: 0.0,
    }
}

#[inline]
#[allow(dead_code)]
fn vertex_from_vec3(v: &mathlib::math3d::Vector3f, u: f32, vv: f32, color: [f32; 4]) -> Vertex {
    vertex(v.get(0), v.get(1), v.get(2), u, vv, color)
}

const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

/// Bright color for line/curve primitives so they stand out on dark backgrounds.
const CURVE_LINE_COLOR: [f32; 4] = [0.2, 0.9, 1.0, 1.0];

// ---------- 2D meshes (z = 0) ----------

/// Fullscreen quad: (-1,-1) to (1,1), white.
#[must_use]
pub fn fullscreen_quad() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        vertex(-1.0, -1.0, 0.0, 0.0, 0.0, WHITE),
        vertex(1.0, -1.0, 0.0, 1.0, 0.0, WHITE),
        vertex(1.0, 1.0, 0.0, 1.0, 1.0, WHITE),
        vertex(-1.0, 1.0, 0.0, 0.0, 1.0, WHITE),
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    (vertices, indices)
}

fn square_2d() -> (Vec<Vertex>, Vec<u16>) {
    let h = 0.5;
    let vertices = vec![
        vertex(-h, -h, 0.0, 0.0, 0.0, WHITE),
        vertex(h, -h, 0.0, 1.0, 0.0, WHITE),
        vertex(h, h, 0.0, 1.0, 1.0, WHITE),
        vertex(-h, h, 0.0, 0.0, 1.0, WHITE),
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    (vertices, indices)
}

fn circle_2d() -> (Vec<Vertex>, Vec<u16>) {
    circle_ellipse_2d(DEFAULT_CIRCLE_SEGMENTS, 1.0, 1.0)
}

fn ellipse_2d() -> (Vec<Vertex>, Vec<u16>) {
    circle_ellipse_2d(
        DEFAULT_CIRCLE_SEGMENTS,
        DEFAULT_ELLIPSE_RX,
        DEFAULT_ELLIPSE_RY,
    )
}

fn circle_ellipse_2d(segments: u32, rx: f32, ry: f32) -> (Vec<Vertex>, Vec<u16>) {
    let n = segments.max(3);
    let num_verts = (n + 2) as usize; // center + (n+1) ring
    let mut color_buf = vec![0.0_f32; num_verts * 4];
    fill_slice_white(&mut color_buf);

    let mut vertices = Vec::with_capacity(num_verts);
    let mut indices = Vec::with_capacity(3 * (n as usize));

    let color0: [f32; 4] = color_buf[0..4].try_into().unwrap();
    vertices.push(vertex(0.0, 0.0, 0.0, 0.5, 0.5, color0));
    let two_pi = core::f32::consts::TAU;
    for i in 0..=n {
        let t = (i as f32 / n as f32) * two_pi;
        let x = rx * cos_scalar(t);
        let y = ry * sin_scalar(t);
        let u = 0.5 + 0.5 * (x / rx);
        let v = 0.5 - 0.5 * (y / ry);
        let j = (i as usize) + 1;
        let color: [f32; 4] = color_buf[j * 4..j * 4 + 4].try_into().unwrap();
        vertices.push(vertex(x, y, 0.0, u, v, color));
    }
    for i in 1..=n {
        indices.push(0);
        indices.push(i as u16);
        indices.push(if i < n { i as u16 + 1 } else { 1 });
    }
    (vertices, indices)
}

fn triangle_2d() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        vertex(0.0, 0.5, 0.0, 0.5, 1.0, [1.0, 0.0, 0.0, 1.0]),
        vertex(-0.5, -0.5, 0.0, 0.0, 0.0, [0.0, 1.0, 0.0, 1.0]),
        vertex(0.5, -0.5, 0.0, 1.0, 0.0, [0.0, 0.0, 1.0, 1.0]),
    ];
    let indices = vec![0, 1, 2];
    (vertices, indices)
}

// ---------- 3D meshes ----------

fn quad_3d() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        vertex(-0.5, -0.5, 0.0, 0.0, 0.0, WHITE),
        vertex(0.5, -0.5, 0.0, 1.0, 0.0, WHITE),
        vertex(0.5, 0.5, 0.0, 1.0, 1.0, WHITE),
        vertex(-0.5, 0.5, 0.0, 0.0, 1.0, WHITE),
    ];
    let indices = vec![0, 1, 2, 0, 2, 3];
    (vertices, indices)
}

fn triangle_3d() -> (Vec<Vertex>, Vec<u16>) {
    let vertices = vec![
        vertex(0.0, 0.5, 0.0, 0.5, 1.0, WHITE),
        vertex(-0.5, -0.5, 0.0, 0.0, 0.0, WHITE),
        vertex(0.5, -0.5, 0.0, 1.0, 0.0, WHITE),
    ];
    let indices = vec![0, 1, 2];
    (vertices, indices)
}

fn cube_3d() -> (Vec<Vertex>, Vec<u16>) {
    let h = 0.5;
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    let positions: [[f32; 3]; 8] = [
        [-h, -h, -h],
        [h, -h, -h],
        [h, h, -h],
        [-h, h, -h],
        [-h, -h, h],
        [h, -h, h],
        [h, h, h],
        [-h, h, h],
    ];
    let faces: [(u16, u16, u16, u16); 6] = [
        (0, 1, 2, 3), // -z: x,y
        (5, 4, 7, 6), // +z: -x,y
        (4, 0, 3, 7), // -x: z,y
        (1, 5, 6, 2), // +x: z,y
        (3, 2, 6, 7), // +y: x,z
        (4, 5, 1, 0), // -y: x,-z
    ];
    // Per-face UV from position: map the two varying axes to [0,1]. Normal is the constant axis.
    let to_uv = |face_ix: usize, p: [f32; 3]| -> [f32; 2] {
        let u = (p[0] + h) / (2.0 * h);
        let v = (p[1] + h) / (2.0 * h);
        let w = (p[2] + h) / (2.0 * h);
        match face_ix {
            0 => [u, v],       // -z: u=x, v=y
            1 => [1.0 - u, v], // +z: u=-x, v=y
            2 => [w, v],       // -x: u=z, v=y
            3 => [w, v],       // +x: u=z, v=y
            4 => [u, w],       // +y: u=x, v=z
            5 => [u, 1.0 - w], // -y: u=x, v=-z
            _ => [u, v],
        }
    };
    for (face_ix, &(a, b, c, d)) in faces.iter().enumerate() {
        let base = vertices.len() as u16;
        for &vi in &[a, b, c, d] {
            let p = positions[vi as usize];
            let [u, v] = to_uv(face_ix, p);
            vertices.push(vertex(p[0], p[1], p[2], u, v, WHITE));
        }
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
        indices.push(base);
        indices.push(base + 2);
        indices.push(base + 3);
    }
    (vertices, indices)
}

fn tetrahedron_3d() -> (Vec<Vertex>, Vec<u16>) {
    let s = 0.5_f32;
    let t = 1.0_f32 / (2.0_f32).sqrt();
    let v0 = [0.0_f32, s, 0.0];
    let v1 = [-s * t, -s, 0.0];
    let v2 = [s * t, -s, 0.0];
    let v3 = [0.0, 0.0, s * (2.0_f32).sqrt()];
    let positions = [v0, v1, v2, v3];
    // Four faces; each face gets three vertices with barycentric UVs (u,v = two of three coords).
    let faces: [(usize, usize, usize); 4] = [(0, 1, 2), (0, 2, 3), (0, 3, 1), (1, 3, 2)];
    let bary_uvs: [[f32; 2]; 3] = [[1.0, 0.0], [0.0, 1.0], [0.0, 0.0]]; // (1,0,0), (0,1,0), (0,0,1)
    let mut vertices = Vec::with_capacity(12);
    let mut indices = Vec::with_capacity(12);
    for (i, j, k) in faces {
        let base = vertices.len() as u16;
        for (idx, &vi) in [i, j, k].iter().enumerate() {
            let p = positions[vi];
            let [u, v] = bary_uvs[idx];
            vertices.push(vertex(p[0], p[1], p[2], u, v, WHITE));
        }
        indices.push(base);
        indices.push(base + 1);
        indices.push(base + 2);
    }
    (vertices, indices)
}

fn cylinder_3d() -> (Vec<Vertex>, Vec<u16>) {
    let n = DEFAULT_CIRCLE_SEGMENTS.max(3);
    let r = DEFAULT_CYLINDER_RADIUS;
    let h = DEFAULT_CYLINDER_HALF_HEIGHT;
    let two_pi = core::f32::consts::TAU;
    let num_ring = (n as usize + 1) * 2;
    let num_verts = num_ring + 2; // two caps
    let mut color_buf = vec![0.0_f32; num_verts * 4];
    fill_slice_white(&mut color_buf);

    let mut vertices = Vec::with_capacity(num_verts);
    let mut indices = Vec::new();

    for i in 0..=n {
        let t = (i as f32 / n as f32) * two_pi;
        let x = r * cos_scalar(t);
        let y = r * sin_scalar(t);
        let u = i as f32 / n as f32;
        let j = (i as usize) * 2;
        let c0: [f32; 4] = color_buf[j * 4..j * 4 + 4].try_into().unwrap();
        let c1: [f32; 4] = color_buf[(j + 1) * 4..(j + 1) * 4 + 4].try_into().unwrap();
        vertices.push(vertex(x, y, -h, u, 0.0, c0));
        vertices.push(vertex(x, y, h, u, 1.0, c1));
    }
    for i in 0..(n as usize) {
        let a = (i * 2) as u16;
        let b = (i * 2 + 1) as u16;
        let c = (i * 2 + 2) as u16;
        let d = (i * 2 + 3) as u16;
        indices.push(a);
        indices.push(b);
        indices.push(c);
        indices.push(c);
        indices.push(b);
        indices.push(d);
    }
    let bottom_center = num_ring as u16;
    let top_center = bottom_center + 1;
    let c_bot: [f32; 4] = color_buf[num_ring * 4..num_ring * 4 + 4]
        .try_into()
        .unwrap();
    let c_top: [f32; 4] = color_buf[(num_ring + 1) * 4..(num_ring + 1) * 4 + 4]
        .try_into()
        .unwrap();
    vertices.push(vertex(0.0, 0.0, -h, 0.5, 0.5, c_bot));
    vertices.push(vertex(0.0, 0.0, h, 0.5, 0.5, c_top));
    for i in 0..n {
        let next = if i + 1 < n { (i as u16 + 1) * 2 } else { 0 };
        let curr = (i as u16) * 2;
        indices.push(bottom_center);
        indices.push(curr);
        indices.push(next);
        indices.push(top_center);
        indices.push(next + 1);
        indices.push(curr + 1);
    }
    (vertices, indices)
}

fn sphere_3d() -> (Vec<Vertex>, Vec<u16>) {
    let n_lat = DEFAULT_SPHERE_SEGMENTS.max(3);
    let n_long = (DEFAULT_SPHERE_SEGMENTS * 2).max(4);
    let r = 0.5;
    let verts_per_ring = (n_long + 2) as usize;
    let mut vertices = Vec::with_capacity((n_lat as usize + 1) * verts_per_ring);
    let mut indices = Vec::with_capacity(6 * n_lat as usize * n_long as usize);

    for lat in 0..=n_lat {
        let v = lat as f32 / n_lat as f32;
        let phi = core::f32::consts::FRAC_PI_2 - v * core::f32::consts::PI;
        let y = r * sin_scalar(phi);
        let ring_r = r * cos_scalar(phi);
        for long in 0..=n_long {
            let u = long as f32 / n_long as f32;
            let theta = u * core::f32::consts::TAU;
            let x = ring_r * cos_scalar(theta);
            let z = ring_r * sin_scalar(theta);
            let uv_u = u;
            let uv_v = 1.0 - v;
            vertices.push(vertex(x, y, z, uv_u, uv_v, WHITE));
        }
        // Duplicate first meridian with u=1 to avoid texture seam at wrap boundary
        let theta0 = 0.0_f32;
        let x0 = ring_r * cos_scalar(theta0);
        let z0 = ring_r * sin_scalar(theta0);
        let uv_v = 1.0 - v;
        vertices.push(vertex(x0, y, z0, 1.0, uv_v, WHITE));
    }
    let verts_per_ring_u = verts_per_ring as u32;
    for lat in 0..n_lat {
        for long in 0..n_long {
            let a = (lat * verts_per_ring_u + long) as u16;
            let b = a + 1;
            let c = a + (verts_per_ring_u as u16);
            let d = c + 1;
            indices.extend([a, c, b, b, c, d]);
        }
    }
    (vertices, indices)
}

fn cone_3d() -> (Vec<Vertex>, Vec<u16>) {
    let n = DEFAULT_CONE_SEGMENTS.max(3);
    let r = DEFAULT_CYLINDER_RADIUS;
    let h = DEFAULT_CYLINDER_HALF_HEIGHT;
    let two_pi = core::f32::consts::TAU;
    let tip_idx = 0u16;
    let base_center_idx = 1u16;
    let mut vertices = Vec::with_capacity((3 + n as usize) as usize);
    vertices.push(vertex(0.0, 0.0, h, 0.5, 0.0, WHITE));
    vertices.push(vertex(0.0, 0.0, -h, 0.5, 1.0, WHITE));
    for i in 0..=n {
        let t = (i as f32 / n as f32) * two_pi;
        let x = r * cos_scalar(t);
        let y = r * sin_scalar(t);
        let u = i as f32 / n as f32;
        vertices.push(vertex(x, y, -h, u, 1.0, WHITE));
    }
    let mut indices = Vec::new();
    let n_usize = n as usize;
    for i in 0..n_usize {
        let base_v = 2 + i;
        let next = 2 + (i + 1) % n_usize;
        indices.extend([tip_idx, base_v as u16, next as u16]);
        indices.extend([base_center_idx, next as u16, base_v as u16]);
    }
    (vertices, indices)
}

fn capsule_3d() -> (Vec<Vertex>, Vec<u16>) {
    let n = DEFAULT_CONE_SEGMENTS.max(3);
    let r = DEFAULT_CYLINDER_RADIUS;
    let half_h = DEFAULT_CAPSULE_HALF_HEIGHT;
    let two_pi = core::f32::consts::TAU;
    let n_lat = (n / 2).max(2);
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let z_bottom = -half_h - r;
    let z_top = half_h + r;

    let ring_len = (n + 1) as usize;
    let mut base = 0u16;

    for i in 0..=n {
        let t = (i as f32 / n as f32) * two_pi;
        let x = r * cos_scalar(t);
        let y = r * sin_scalar(t);
        let u = i as f32 / n as f32;
        vertices.push(vertex(x, y, -half_h, u, 0.0, WHITE));
        vertices.push(vertex(x, y, half_h, u, 1.0, WHITE));
    }
    for i in 0..(n as usize) {
        let a = base + (i * 2) as u16;
        let b = a + 1;
        let c = base + (i * 2 + 2) as u16;
        let d = c + 1;
        indices.extend([a, b, c, c, b, d]);
    }
    base += (ring_len * 2) as u16;

    let bottom_center = base;
    vertices.push(vertex(0.0, 0.0, z_bottom, 0.5, 0.5, WHITE));
    base += 1;
    for lat in 1..n_lat {
        let v = lat as f32 / n_lat as f32;
        let phi = core::f32::consts::FRAC_PI_2 * (1.0 - v);
        let dz = r * cos_scalar(phi);
        let ring_r = r * sin_scalar(phi);
        for long in 0..=n {
            let u = long as f32 / n as f32;
            let theta = u * two_pi;
            let x = ring_r * cos_scalar(theta);
            let y = ring_r * sin_scalar(theta);
            vertices.push(vertex(x, y, z_bottom + dz, u, v, WHITE));
        }
    }
    let bottom_first_ring = base;
    let n_lat_usize = (n_lat - 1) as usize;
    for lat in 0..n_lat_usize {
        for long in 0..(n as usize) {
            let a = base + (lat * ring_len + long) as u16;
            let b = a + 1;
            let c = a + ring_len as u16;
            let d = c + 1;
            indices.extend([a, c, b, b, c, d]);
        }
    }
    for i in 0..(n as usize) {
        let next = (i + 1) % ring_len;
        indices.extend([
            bottom_center,
            bottom_first_ring + next as u16,
            bottom_first_ring + i as u16,
        ]);
    }
    base += (n_lat_usize * ring_len) as u16;

    let top_center = base;
    vertices.push(vertex(0.0, 0.0, z_top, 0.5, 0.5, WHITE));
    base += 1;
    for lat in 1..n_lat {
        let v = lat as f32 / n_lat as f32;
        let phi = -core::f32::consts::FRAC_PI_2 * v;
        let dz = r * cos_scalar(phi);
        let ring_r = r * sin_scalar(phi);
        for long in 0..=n {
            let u = long as f32 / n as f32;
            let theta = u * two_pi;
            let x = ring_r * cos_scalar(theta);
            let y = ring_r * sin_scalar(theta);
            vertices.push(vertex(x, y, z_top + dz, u, v, WHITE));
        }
    }
    let top_first_ring = base;
    for lat in 0..n_lat_usize {
        for long in 0..(n as usize) {
            let a = base + (lat * ring_len + long) as u16;
            let b = a + 1;
            let c = a + ring_len as u16;
            let d = c + 1;
            indices.extend([a, c, b, b, c, d]);
        }
    }
    for i in 0..(n as usize) {
        let next = (i + 1) % ring_len;
        indices.extend([
            top_center,
            top_first_ring + i as u16,
            top_first_ring + next as u16,
        ]);
    }

    (vertices, indices)
}
