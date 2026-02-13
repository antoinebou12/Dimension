//! Tetrahedral mesh topology: vertices, tetrahedra, rest-pose data.
//!
//! [`TetMesh`] stores the connectivity and precomputed inverse rest-shape
//! matrices (Dm⁻¹) and rest volumes used by Neo-Hookean constraints.

use mathlib::math3d_raw::{mat3_det, mat3_inverse};

/// Tetrahedral mesh: connectivity and rest-pose data.
///
/// Each tetrahedron references four particle indices. The precomputed
/// `dm_inv` and `rest_volumes` arrays are parallel to `tets`.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TetMesh {
    /// Tetrahedra as 4-tuples of particle indices `[p0, p1, p2, p3]`.
    pub tets: Vec<[usize; 4]>,
    /// Inverse rest-shape matrices Dm⁻¹ (one 3×3 row-major per tet).
    pub dm_inv: Vec<[f32; 9]>,
    /// Rest-pose volumes V₀ (one per tet, always positive).
    pub rest_volumes: Vec<f32>,
}

impl TetMesh {
    /// Build a [`TetMesh`] from vertex positions and tet indices.
    ///
    /// Computes Dm⁻¹ and V₀ for each tetrahedron. Tetrahedra whose rest-shape
    /// matrix is singular (degenerate) are assigned identity Dm⁻¹ and zero volume.
    ///
    /// # Arguments
    ///
    /// * `positions` — vertex positions (x, y, z) for all particles.
    /// * `tets` — tetrahedral indices (each references 4 positions).
    ///
    /// # Examples
    ///
    /// ```
    /// # use physics::tet::TetMesh;
    /// let verts = vec![
    ///     [0.0, 0.0, 0.0],
    ///     [1.0, 0.0, 0.0],
    ///     [0.0, 1.0, 0.0],
    ///     [0.0, 0.0, 1.0],
    /// ];
    /// let mesh = TetMesh::from_vertices_and_tets(&verts, vec![[0, 1, 2, 3]]);
    /// assert_eq!(mesh.tets.len(), 1);
    /// assert!(mesh.rest_volumes[0] > 0.0);
    /// ```
    #[must_use]
    pub fn from_vertices_and_tets(positions: &[[f32; 3]], tets: Vec<[usize; 4]>) -> Self {
        let mut dm_inv = Vec::with_capacity(tets.len());
        let mut rest_volumes = Vec::with_capacity(tets.len());

        for tet in &tets {
            let p0 = positions[tet[0]];
            let p1 = positions[tet[1]];
            let p2 = positions[tet[2]];
            let p3 = positions[tet[3]];

            // Dm = [p1-p0 | p2-p0 | p3-p0] as columns → row-major storage
            let dm = edges_to_mat3(&p0, &p1, &p2, &p3);
            let det = mat3_det(&dm);
            let vol = det.abs() / 6.0;

            if let Some(inv) = mat3_inverse(&dm) {
                dm_inv.push(inv);
            } else {
                // Degenerate tet — use identity as fallback
                dm_inv.push([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0]);
            }
            rest_volumes.push(vol);
        }

        Self {
            tets,
            dm_inv,
            rest_volumes,
        }
    }

    /// Generate a regular tetrahedral mesh from a cube grid.
    ///
    /// Creates `nx × ny × nz` hex cells, each subdivided into 6 tetrahedra,
    /// with the given `spacing` between adjacent vertices.
    ///
    /// Returns `(positions, TetMesh)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use physics::tet::TetMesh;
    /// let (verts, mesh) = TetMesh::cube(2, 2, 2, 1.0);
    /// assert_eq!(verts.len(), 27); // (2+1)^3
    /// assert_eq!(mesh.tets.len(), 2 * 2 * 2 * 6); // 6 tets per cell
    /// ```
    #[must_use]
    pub fn cube(nx: usize, ny: usize, nz: usize, spacing: f32) -> (Vec<[f32; 3]>, Self) {
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
                    // 8 corners of the hex cell
                    let v0 = idx(ix, it, iz);
                    let v1 = idx(ix + 1, it, iz);
                    let v2 = idx(ix + 1, it + 1, iz);
                    let v3 = idx(ix, it + 1, iz);
                    let v4 = idx(ix, it, iz + 1);
                    let v5 = idx(ix + 1, it, iz + 1);
                    let v6 = idx(ix + 1, it + 1, iz + 1);
                    let v7 = idx(ix, it + 1, iz + 1);

                    // 6-tet decomposition of a hex cell (consistent diagonal)
                    tets.push([v0, v1, v3, v4]);
                    tets.push([v1, v2, v3, v6]);
                    tets.push([v1, v4, v5, v6]);
                    tets.push([v3, v4, v6, v7]);
                    tets.push([v1, v3, v4, v6]);
                    tets.push([v0, v1, v3, v4]); // duplicate-safe: overridden below
                }
            }
        }

        // Re-do the 6-tet decomposition properly (the last entry above was a copy).
        // Standard 5-tet or 6-tet subdivisions exist. Use the Dompierre 6-tet:
        tets.clear();
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

                    // 6 tetrahedra per hex (BCC diagonal v0-v6)
                    tets.push([v0, v5, v1, v6]);
                    tets.push([v0, v1, v2, v6]);
                    tets.push([v0, v2, v3, v6]);
                    tets.push([v0, v3, v7, v6]);
                    tets.push([v0, v7, v4, v6]);
                    tets.push([v0, v4, v5, v6]);
                }
            }
        }

        let mesh = Self::from_vertices_and_tets(&positions, tets);
        (positions, mesh)
    }

    /// Number of tetrahedra.
    #[must_use]
    pub fn num_tets(&self) -> usize {
        self.tets.len()
    }
}

/// Build the edge matrix Dm from four vertex positions.
///
/// Dm columns = (p1-p0, p2-p0, p3-p0), stored in row-major.
///
/// Row-major layout with columns as edge vectors:
/// ```text
/// [ (p1-p0).x  (p2-p0).x  (p3-p0).x ]
/// [ (p1-p0).y  (p2-p0).y  (p3-p0).y ]
/// [ (p1-p0).z  (p2-p0).z  (p3-p0).z ]
/// ```
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
