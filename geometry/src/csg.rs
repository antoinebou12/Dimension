//! Constructive solid geometry: boolean operations on watertight triangle meshes.
//!
//! Assumes inputs are closed (watertight) and use consistent winding. Uses
//! point-in-mesh tests to classify triangles for union, intersection, and difference.

use crate::{GeometryError, TriMesh};

/// CSG operation type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CsgOp {
    /// A ∪ B: all points in A or B.
    Union,
    /// A ∩ B: points in both A and B.
    Intersection,
    /// A \ B: points in A but not in B.
    Difference,
}

/// Computes the union of two watertight meshes.
///
/// Returns a mesh containing triangles from A and those triangles from B whose
/// centroid lies outside A. Assumes both meshes are closed and consistently wound.
///
/// # Errors
/// Returns `GeometryError::EmptyInput` if either mesh has no triangles.
pub fn csg_union(a: &TriMesh, b: &TriMesh) -> Result<TriMesh, GeometryError> {
    csg_boolean(a, b, CsgOp::Union)
}

/// Computes the intersection of two watertight meshes.
///
/// # Errors
/// Returns `GeometryError::EmptyInput` if either mesh has no triangles.
pub fn csg_intersection(a: &TriMesh, b: &TriMesh) -> Result<TriMesh, GeometryError> {
    csg_boolean(a, b, CsgOp::Intersection)
}

/// Computes the difference A \ B (points in A but not in B).
///
/// # Errors
/// Returns `GeometryError::EmptyInput` if either mesh has no triangles.
pub fn csg_difference(a: &TriMesh, b: &TriMesh) -> Result<TriMesh, GeometryError> {
    csg_boolean(a, b, CsgOp::Difference)
}

fn csg_boolean(a: &TriMesh, b: &TriMesh, op: CsgOp) -> Result<TriMesh, GeometryError> {
    if a.indices.is_empty() || b.indices.is_empty() {
        return Err(GeometryError::EmptyInput);
    }
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    match op {
        CsgOp::Union => {
            positions.extend_from_slice(&a.positions);
            for tri in &a.indices {
                indices.push(*tri);
            }
            for tri in &b.indices {
                let c = centroid(b, tri);
                if !point_in_mesh(c, a) {
                    let n = positions.len() as u32;
                    indices.push([n, n + 1, n + 2]);
                    positions.push(b.positions[tri[0] as usize]);
                    positions.push(b.positions[tri[1] as usize]);
                    positions.push(b.positions[tri[2] as usize]);
                }
            }
        }
        CsgOp::Intersection => {
            for tri in &a.indices {
                let c = centroid(a, tri);
                if point_in_mesh(c, b) {
                    indices.push([
                        positions.len() as u32,
                        positions.len() as u32 + 1,
                        positions.len() as u32 + 2,
                    ]);
                    positions.push(a.positions[tri[0] as usize]);
                    positions.push(a.positions[tri[1] as usize]);
                    positions.push(a.positions[tri[2] as usize]);
                }
            }
            for tri in &b.indices {
                let c = centroid(b, tri);
                if point_in_mesh(c, a) {
                    indices.push([
                        positions.len() as u32,
                        positions.len() as u32 + 1,
                        positions.len() as u32 + 2,
                    ]);
                    positions.push(b.positions[tri[0] as usize]);
                    positions.push(b.positions[tri[1] as usize]);
                    positions.push(b.positions[tri[2] as usize]);
                }
            }
        }
        CsgOp::Difference => {
            for tri in &a.indices {
                let c = centroid(a, tri);
                if !point_in_mesh(c, b) {
                    let n = positions.len() as u32;
                    indices.push([n, n + 1, n + 2]);
                    positions.push(a.positions[tri[0] as usize]);
                    positions.push(a.positions[tri[1] as usize]);
                    positions.push(a.positions[tri[2] as usize]);
                }
            }
            for tri in &b.indices {
                let c = centroid(b, tri);
                if point_in_mesh(c, a) {
                    let n = positions.len() as u32;
                    indices.push([n, n + 1, n + 2]);
                    positions.push(b.positions[tri[0] as usize]);
                    positions.push(b.positions[tri[2] as usize]);
                    positions.push(b.positions[tri[1] as usize]);
                }
            }
        }
    }
    Ok(TriMesh {
        positions,
        indices,
        normals: None,
    })
}

fn centroid(mesh: &TriMesh, tri: &[u32; 3]) -> [f32; 3] {
    let p0 = mesh.positions[tri[0] as usize];
    let p1 = mesh.positions[tri[1] as usize];
    let p2 = mesh.positions[tri[2] as usize];
    [
        (p0[0] + p1[0] + p2[0]) / 3.0,
        (p0[1] + p1[1] + p2[1]) / 3.0,
        (p0[2] + p1[2] + p2[2]) / 3.0,
    ]
}

fn point_in_mesh(p: [f32; 3], mesh: &TriMesh) -> bool {
    let ray_dir = [1.0, 0.0, 0.0];
    let mut hits = 0_u32;
    for tri in &mesh.indices {
        let a = mesh.positions[tri[0] as usize];
        let b = mesh.positions[tri[1] as usize];
        let c = mesh.positions[tri[2] as usize];
        if ray_triangle_intersect(p, ray_dir, a, b, c) {
            hits += 1;
        }
    }
    hits % 2 == 1
}

fn ray_triangle_intersect(
    orig: [f32; 3],
    dir: [f32; 3],
    v0: [f32; 3],
    v1: [f32; 3],
    v2: [f32; 3],
) -> bool {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];
    let h = [
        dir[1] * e2[2] - dir[2] * e2[1],
        dir[2] * e2[0] - dir[0] * e2[2],
        dir[0] * e2[1] - dir[1] * e2[0],
    ];
    let a = e1[0] * h[0] + e1[1] * h[1] + e1[2] * h[2];
    if a.abs() < 1e-8 {
        return false;
    }
    let f = 1.0 / a;
    let s = [orig[0] - v0[0], orig[1] - v0[1], orig[2] - v0[2]];
    let u = f * (s[0] * h[0] + s[1] * h[1] + s[2] * h[2]);
    if u < 0.0 || u > 1.0 {
        return false;
    }
    let q = [
        s[1] * e1[2] - s[2] * e1[1],
        s[2] * e1[0] - s[0] * e1[2],
        s[0] * e1[1] - s[1] * e1[0],
    ];
    let v = f * (dir[0] * q[0] + dir[1] * q[1] + dir[2] * q[2]);
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let t = f * (e2[0] * q[0] + e2[1] * q[1] + e2[2] * q[2]);
    t > 1e-6
}
