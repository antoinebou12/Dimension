//! Tetrahedral mesh: grid and surface-based tetrahedralization.

use crate::{GeometryError, TriMesh};
use collision::Aabb;
use std::f32::consts::PI;

/// Tetrahedral mesh: vertex positions and tet connectivity.
///
/// Optionally stores inverse rest-shape matrices and rest volumes for physics use.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TetMesh {
    /// Tetrahedra as 4-tuples of vertex indices `[p0, p1, p2, p3]`.
    pub tets: Vec<[usize; 4]>,
    /// Optional inverse rest-shape matrices Dm⁻¹ (one 3×3 row-major per tet).
    pub dm_inv: Option<Vec<[f32; 9]>>,
    /// Optional rest-pose volumes V₀ (one per tet).
    pub rest_volumes: Option<Vec<f32>>,
}

fn mat3_det(m: &[f32; 9]) -> f32 {
    m[0] * (m[4] * m[8] - m[5] * m[7]) - m[1] * (m[3] * m[8] - m[5] * m[6])
        + m[2] * (m[3] * m[7] - m[4] * m[6])
}

fn mat3_inverse(m: &[f32; 9]) -> Option<[f32; 9]> {
    let det = mat3_det(m);
    if det.abs() < 1e-12 {
        return None;
    }
    let inv_det = 1.0 / det;
    Some([
        (m[4] * m[8] - m[5] * m[7]) * inv_det,
        (m[2] * m[7] - m[1] * m[8]) * inv_det,
        (m[1] * m[5] - m[2] * m[4]) * inv_det,
        (m[5] * m[6] - m[3] * m[8]) * inv_det,
        (m[0] * m[8] - m[2] * m[6]) * inv_det,
        (m[2] * m[3] - m[0] * m[5]) * inv_det,
        (m[3] * m[7] - m[4] * m[6]) * inv_det,
        (m[1] * m[6] - m[0] * m[7]) * inv_det,
        (m[0] * m[4] - m[1] * m[3]) * inv_det,
    ])
}

fn edges_to_mat3(p0: &[f32; 3], p1: &[f32; 3], p2: &[f32; 3], p3: &[f32; 3]) -> [f32; 9] {
    [
        p1[0] - p0[0],
        p2[0] - p0[0],
        p3[0] - p0[0],
        p1[1] - p0[1],
        p2[1] - p0[1],
        p3[1] - p0[1],
        p1[2] - p0[2],
        p2[2] - p0[2],
        p3[2] - p0[2],
    ]
}

impl TetMesh {
    /// Builds a [`TetMesh`] from vertex positions and tet indices.
    ///
    /// Optionally computes Dm⁻¹ and rest volumes. Degenerate tets get identity Dm⁻¹ and zero volume.
    #[must_use]
    pub fn from_vertices_and_tets(
        positions: &[[f32; 3]],
        tets: Vec<[usize; 4]>,
        compute_rest: bool,
    ) -> Self {
        let (dm_inv, rest_volumes) = if compute_rest {
            let mut dm_inv = Vec::with_capacity(tets.len());
            let mut rest_volumes = Vec::with_capacity(tets.len());
            for tet in &tets {
                let p0 = positions[tet[0]];
                let p1 = positions[tet[1]];
                let p2 = positions[tet[2]];
                let p3 = positions[tet[3]];
                let dm = edges_to_mat3(&p0, &p1, &p2, &p3);
                let det = mat3_det(&dm);
                let vol = det.abs() / 6.0;
                if let Some(inv) = mat3_inverse(&dm) {
                    dm_inv.push(inv);
                } else {
                    dm_inv.push([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
                }
                rest_volumes.push(vol);
            }
            (Some(dm_inv), Some(rest_volumes))
        } else {
            (None, None)
        };
        Self {
            tets,
            dm_inv,
            rest_volumes,
        }
    }

    /// Generates a regular tetrahedral grid (Dompierre 6-tet per hex cell).
    ///
    /// Returns `(positions, TetMesh)`.
    #[must_use]
    pub fn tetrahedralize_grid(
        nx: usize,
        ny: usize,
        nz: usize,
        spacing: f32,
        compute_rest: bool,
    ) -> (Vec<[f32; 3]>, Self) {
        let vx = nx + 1;
        let vy = ny + 1;
        let vz = nz + 1;
        let num_verts = vx * vy * vz;
        let mut positions = Vec::with_capacity(num_verts);
        for iz in 0..vz {
            for it in 0..vy {
                for ix in 0..vx {
                    positions.push([
                        ix as f32 * spacing,
                        it as f32 * spacing,
                        iz as f32 * spacing,
                    ]);
                }
            }
        }
        let idx = |ix: usize, it: usize, iz: usize| -> usize { iz * vy * vx + it * vx + ix };
        let mut tets = Vec::with_capacity(nx * ny * nz * 6);
        for iz in 0..nz {
            for it in 0..ny {
                for ix in 0..nx {
                    let v0 = idx(ix, it, iz);
                    let v1 = idx(ix + 1, it, iz);
                    let v2 = idx(ix + 1, it + 1, iz);
                    let v3 = idx(ix, it + 1, iz);
                    let v4 = idx(ix, it, iz + 1);
                    let v5 = idx(ix + 1, it, iz + 1);
                    let v6 = idx(ix + 1, it + 1, iz + 1);
                    let v7 = idx(ix, it + 1, iz + 1);
                    tets.push([v0, v5, v1, v6]);
                    tets.push([v0, v1, v2, v6]);
                    tets.push([v0, v2, v3, v6]);
                    tets.push([v0, v3, v7, v6]);
                    tets.push([v0, v7, v4, v6]);
                    tets.push([v0, v4, v5, v6]);
                }
            }
        }
        let mesh = Self::from_vertices_and_tets(&positions, tets, compute_rest);
        (positions, mesh)
    }

    /// Tetrahedralizes the interior of a closed triangle mesh by covering its AABB with a grid and keeping tets whose centroid is inside the mesh.
    ///
    /// Uses a ray-cast point-in-mesh test (parity of ray-triangle intersections). Assumes watertight mesh.
    ///
    /// # Errors
    /// Returns `GeometryError::EmptyInput` if the mesh has no triangles.
    pub fn tetrahedralize_surface(
        mesh: &TriMesh,
        resolution: [usize; 3],
        padding: f32,
    ) -> Result<(Vec<[f32; 3]>, Self), GeometryError> {
        if mesh.indices.is_empty() {
            return Err(GeometryError::EmptyInput);
        }
        let aabb = mesh_aabb(mesh);
        let min = [
            aabb.min[0] - padding,
            aabb.min[1] - padding,
            aabb.min[2] - padding,
        ];
        let size = [
            (aabb.max[0] - aabb.min[0] + 2.0 * padding).max(1e-6),
            (aabb.max[1] - aabb.min[1] + 2.0 * padding).max(1e-6),
            (aabb.max[2] - aabb.min[2] + 2.0 * padding).max(1e-6),
        ];
        let cell_size = (size[0] / resolution[0] as f32)
            .max(size[1] / resolution[1] as f32)
            .max(size[2] / resolution[2] as f32);
        let (positions, full_mesh) = Self::tetrahedralize_grid(
            resolution[0],
            resolution[1],
            resolution[2],
            cell_size,
            false,
        );
        let origin = min;
        let positions_shifted: Vec<[f32; 3]> = positions
            .iter()
            .map(|p| [origin[0] + p[0], origin[1] + p[1], origin[2] + p[2]])
            .collect();
        let mut inside = vec![false; full_mesh.tets.len()];
        for (idx, tet) in full_mesh.tets.iter().enumerate() {
            let c = [
                (positions_shifted[tet[0]][0]
                    + positions_shifted[tet[1]][0]
                    + positions_shifted[tet[2]][0]
                    + positions_shifted[tet[3]][0])
                    * 0.25,
                (positions_shifted[tet[0]][1]
                    + positions_shifted[tet[1]][1]
                    + positions_shifted[tet[2]][1]
                    + positions_shifted[tet[3]][1])
                    * 0.25,
                (positions_shifted[tet[0]][2]
                    + positions_shifted[tet[1]][2]
                    + positions_shifted[tet[2]][2]
                    + positions_shifted[tet[3]][2])
                    * 0.25,
            ];
            if point_in_mesh(c, mesh) {
                inside[idx] = true;
            }
        }
        let mut new_tets = Vec::new();
        for (idx, &tet) in full_mesh.tets.iter().enumerate() {
            if inside[idx] {
                new_tets.push(tet);
            }
        }
        let mesh_out = Self::from_vertices_and_tets(&positions_shifted, new_tets, true);
        Ok((positions_shifted, mesh_out))
    }

    /// Number of tetrahedra.
    #[must_use]
    pub fn num_tets(&self) -> usize {
        self.tets.len()
    }

    /// Radius ratio (inscribed radius / circumscribed radius * 3) for tet at index `i`; higher is better.
    #[must_use]
    pub fn radius_ratio(&self, positions: &[[f32; 3]], i: usize) -> f32 {
        let tet = &self.tets[i];
        let p0 = positions[tet[0]];
        let p1 = positions[tet[1]];
        let p2 = positions[tet[2]];
        let p3 = positions[tet[3]];
        let vol = tet_volume(p0, p1, p2, p3);
        if vol <= 0.0 {
            return 0.0;
        }
        let a = edge_len_sq(p0, p1);
        let b = edge_len_sq(p0, p2);
        let c = edge_len_sq(p0, p3);
        let d = edge_len_sq(p1, p2);
        let e = edge_len_sq(p1, p3);
        let f = edge_len_sq(p2, p3);
        let circum_r_sq =
            (a * (b + c + e + f - a) + d * (b + c + e + f - d) + 2.0 * (b * f + c * e))
                / (16.0 * vol * vol);
        if circum_r_sq < 1e-20 {
            return 0.0;
        }
        let in_r = 3.0 * vol
            / (face_area(p0, p1, p2)
                + face_area(p0, p1, p3)
                + face_area(p0, p2, p3)
                + face_area(p1, p2, p3));
        3.0 * in_r / (circum_r_sq.sqrt())
    }

    /// Minimum dihedral angle (radians) in the tet at index `i`.
    #[must_use]
    pub fn min_dihedral_angle(&self, positions: &[[f32; 3]], i: usize) -> f32 {
        let tet = &self.tets[i];
        let p0 = positions[tet[0]];
        let p1 = positions[tet[1]];
        let p2 = positions[tet[2]];
        let p3 = positions[tet[3]];
        let n012 = face_normal(p0, p1, p2);
        let n013 = face_normal(p0, p1, p3);
        let n023 = face_normal(p0, p2, p3);
        let n123 = face_normal(p1, p2, p3);
        let mut min_angle = PI;
        for (na, nb) in [
            (&n012, &n013),
            (&n012, &n023),
            (&n012, &n123),
            (&n013, &n023),
            (&n013, &n123),
            (&n023, &n123),
        ] {
            let cos_a = (na[0] * nb[0] + na[1] * nb[1] + na[2] * nb[2]).abs();
            let angle = (1.0_f32 - cos_a).min(1.0).max(-1.0).acos();
            min_angle = min_angle.min(angle);
        }
        min_angle
    }
}

fn mesh_aabb(mesh: &TriMesh) -> Aabb {
    let mut min = [f32::INFINITY; 3];
    let mut max = [f32::NEG_INFINITY; 3];
    for p in &mesh.positions {
        for (i, &v) in p.iter().enumerate() {
            min[i] = min[i].min(v);
            max[i] = max[i].max(v);
        }
    }
    Aabb::new(min, max)
}

fn tet_volume(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3]) -> f32 {
    let dm = edges_to_mat3(&p0, &p1, &p2, &p3);
    mat3_det(&dm).abs() / 6.0
}

fn edge_len_sq(a: [f32; 3], b: [f32; 3]) -> f32 {
    let dx = b[0] - a[0];
    let dy = b[1] - a[1];
    let dz = b[2] - a[2];
    dx * dx + dy * dy + dz * dz
}

fn face_normal(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> [f32; 3] {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];
    let len = (nx * nx + ny * ny + nz * nz).sqrt().max(1e-10);
    [nx / len, ny / len, nz / len]
}

fn face_area(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> f32 {
    let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
    let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
    let cross = [
        e1[1] * e2[2] - e1[2] * e2[1],
        e1[2] * e2[0] - e1[0] * e2[2],
        e1[0] * e2[1] - e1[1] * e2[0],
    ];
    (cross[0] * cross[0] + cross[1] * cross[1] + cross[2] * cross[2]).sqrt() * 0.5
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
