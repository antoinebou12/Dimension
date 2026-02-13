//! Voxel grid: surface voxelization, flood fill, and marching cubes isosurface extraction.

use crate::{GeometryError, TriMesh};
use collision::Aabb;
use std::collections::VecDeque;

const MAX_VOXELS: usize = 256 * 256 * 256;

/// 3D uniform voxel grid (row-major: x, then y, then z).
#[derive(Clone, Debug)]
pub struct VoxelGrid {
    /// Grid resolution along x.
    pub nx: usize,
    /// Grid resolution along y.
    pub ny: usize,
    /// Grid resolution along z.
    pub nz: usize,
    /// World-space AABB min corner.
    pub min: [f32; 3],
    /// Voxel size (same in all dimensions).
    pub cell_size: f32,
    /// Cell occupancy: `true` = solid/surface.
    pub cells: Vec<bool>,
}

impl VoxelGrid {
    /// Creates an empty grid with the given resolution and bounds.
    ///
    /// # Errors
    /// Returns `GeometryError::VoxelGridTooLarge` if `nx * ny * nz` exceeds the allowed maximum.
    pub fn new(
        nx: usize,
        ny: usize,
        nz: usize,
        min: [f32; 3],
        cell_size: f32,
    ) -> Result<Self, GeometryError> {
        let total = nx
            .checked_mul(ny)
            .and_then(|n| n.checked_mul(nz))
            .unwrap_or(MAX_VOXELS + 1);
        if total > MAX_VOXELS {
            return Err(GeometryError::VoxelGridTooLarge {
                requested: total,
                max: MAX_VOXELS,
            });
        }
        let cells = vec![false; nx * ny * nz];
        Ok(Self {
            nx,
            ny,
            nz,
            min,
            cell_size,
            cells,
        })
    }

    /// Linear index from grid coordinates.
    #[inline]
    #[must_use]
    pub fn index(&self, i: usize, j: usize, k: usize) -> usize {
        i + j * self.nx + k * self.nx * self.ny
    }

    /// World position of voxel center at (i, j, k).
    #[must_use]
    pub fn voxel_center(&self, i: usize, j: usize, k: usize) -> [f32; 3] {
        let half = self.cell_size * 0.5;
        [
            self.min[0] + i as f32 * self.cell_size + half,
            self.min[1] + j as f32 * self.cell_size + half,
            self.min[2] + k as f32 * self.cell_size + half,
        ]
    }

    /// Sets the cell at (i, j, k). No bounds check in release.
    #[inline]
    pub fn set(&mut self, i: usize, j: usize, k: usize, value: bool) {
        let idx = self.index(i, j, k);
        self.cells[idx] = value;
    }

    /// Gets the cell at (i, j, k). No bounds check in release.
    #[inline]
    #[must_use]
    pub fn get(&self, i: usize, j: usize, k: usize) -> bool {
        self.cells[self.index(i, j, k)]
    }

    /// Total number of cells.
    #[must_use]
    pub fn len(&self) -> usize {
        self.nx * self.ny * self.nz
    }
}

/// Voxelizes the surface of a triangle mesh: marks every voxel that intersects a triangle.
///
/// Uses the mesh AABB plus optional padding to define the grid. Each voxel that overlaps
/// any triangle (by testing voxel center distance to triangles) is set.
///
/// # Errors
/// Returns `GeometryError::EmptyInput` if the mesh has no triangles.
/// Returns `GeometryError::VoxelGridTooLarge` if the grid would exceed the maximum size.
pub fn voxelize_mesh(
    mesh: &TriMesh,
    resolution: [usize; 3],
    padding: f32,
) -> Result<VoxelGrid, GeometryError> {
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
    let mut grid = VoxelGrid::new(resolution[0], resolution[1], resolution[2], min, cell_size)?;
    let half = cell_size * 0.5_f32;
    let threshold_sq = half * half * 3.0_f32;
    for tri in &mesh.indices {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        let p0 = mesh.positions[i0];
        let p1 = mesh.positions[i1];
        let p2 = mesh.positions[i2];
        let tri_aabb = triangle_aabb(p0, p1, p2);
        let (min_ix, max_ix) =
            world_to_range(tri_aabb.min[0], tri_aabb.max[0], min[0], cell_size, grid.nx);
        let (min_it, max_it) =
            world_to_range(tri_aabb.min[1], tri_aabb.max[1], min[1], cell_size, grid.ny);
        let (min_iz, max_iz) =
            world_to_range(tri_aabb.min[2], tri_aabb.max[2], min[2], cell_size, grid.nz);
        for k in min_iz..=max_iz.min(grid.nz.saturating_sub(1)) {
            for j in min_it..=max_it.min(grid.ny.saturating_sub(1)) {
                for i in min_ix..=max_ix.min(grid.nx.saturating_sub(1)) {
                    let center = grid.voxel_center(i, j, k);
                    let d_sq = point_triangle_distance_sq(center, p0, p1, p2);
                    if d_sq <= threshold_sq {
                        grid.set(i, j, k, true);
                    }
                }
            }
        }
    }
    Ok(grid)
}

/// Flood-fills from (0,0,0) marking all connected empty cells as exterior; then sets solid = surface and not exterior.
///
/// Call this after `voxelize_mesh` to get a solid voxelization: interior + surface will be `true`, exterior `false`.
pub fn flood_fill(grid: &mut VoxelGrid) {
    if grid.nx == 0 || grid.ny == 0 || grid.nz == 0 {
        return;
    }
    let mut exterior = vec![false; grid.len()];
    let mut queue = VecDeque::new();
    if !grid.get(0, 0, 0) {
        queue.push_back((0_usize, 0_usize, 0_usize));
        exterior[grid.index(0, 0, 0)] = true;
    }
    while let Some((i, j, k)) = queue.pop_front() {
        for (di, dj, dk) in [
            (1i32, 0, 0),
            (-1, 0, 0),
            (0, 1, 0),
            (0, -1, 0),
            (0, 0, 1),
            (0, 0, -1),
        ] {
            let ni = i as i32 + di;
            let nj = j as i32 + dj;
            let nk = k as i32 + dk;
            if ni >= 0
                && (ni as usize) < grid.nx
                && nj >= 0
                && (nj as usize) < grid.ny
                && nk >= 0
                && (nk as usize) < grid.nz
            {
                let idx = grid.index(ni as usize, nj as usize, nk as usize);
                if !grid.cells[idx] && !exterior[idx] {
                    exterior[idx] = true;
                    queue.push_back((ni as usize, nj as usize, nk as usize));
                }
            }
        }
    }
    for idx in 0..grid.len() {
        grid.cells[idx] = !exterior[idx];
    }
}

/// Marching cubes isosurface extraction: converts the voxel grid to a triangle mesh.
///
/// For each cell, corners (i,j,k)..(i+1,j+1,k+1) are evaluated; vertices are placed at
/// edge midpoints between solid and empty corners. Returns an indexed triangle mesh.
#[must_use]
pub fn marching_cubes(grid: &VoxelGrid) -> TriMesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();
    let corner_pos = |g: &VoxelGrid, i: usize, j: usize, k: usize| -> [f32; 3] {
        [
            g.min[0] + i as f32 * g.cell_size,
            g.min[1] + j as f32 * g.cell_size,
            g.min[2] + k as f32 * g.cell_size,
        ]
    };
    let half = grid.cell_size * 0.5;
    for k in 0..grid.nz {
        for j in 0..grid.ny {
            for i in 0..grid.nx {
                let c0 = grid.get(i, j, k);
                let c1 = i < grid.nx.saturating_sub(1) && grid.get(i + 1, j, k);
                let c2 = i < grid.nx.saturating_sub(1)
                    && j < grid.ny.saturating_sub(1)
                    && grid.get(i + 1, j + 1, k);
                let c3 = j < grid.ny.saturating_sub(1) && grid.get(i, j + 1, k);
                let c4 = k < grid.nz.saturating_sub(1) && grid.get(i, j, k + 1);
                let c5 = i < grid.nx.saturating_sub(1)
                    && k < grid.nz.saturating_sub(1)
                    && grid.get(i + 1, j, k + 1);
                let c6 = i < grid.nx.saturating_sub(1)
                    && j < grid.ny.saturating_sub(1)
                    && k < grid.nz.saturating_sub(1)
                    && grid.get(i + 1, j + 1, k + 1);
                let c7 = j < grid.ny.saturating_sub(1)
                    && k < grid.nz.saturating_sub(1)
                    && grid.get(i, j + 1, k + 1);
                let cube_index = (if c0 { 1 } else { 0 })
                    + (if c1 { 2 } else { 0 })
                    + (if c2 { 4 } else { 0 })
                    + (if c3 { 8 } else { 0 })
                    + (if c4 { 16 } else { 0 })
                    + (if c5 { 32 } else { 0 })
                    + (if c6 { 64 } else { 0 })
                    + (if c7 { 128 } else { 0 });
                if cube_index == 0 || cube_index == 255 {
                    continue;
                }
                let base = [
                    corner_pos(grid, i, j, k),
                    corner_pos(grid, i + 1, j, k),
                    corner_pos(grid, i + 1, j + 1, k),
                    corner_pos(grid, i, j + 1, k),
                    corner_pos(grid, i, j, k + 1),
                    corner_pos(grid, i + 1, j, k + 1),
                    corner_pos(grid, i + 1, j + 1, k + 1),
                    corner_pos(grid, i, j + 1, k + 1),
                ];
                let center = [
                    (base[0][0] + base[6][0]) * 0.5,
                    (base[0][1] + base[6][1]) * 0.5,
                    (base[0][2] + base[6][2]) * 0.5,
                ];
                let n = positions.len() as u32;
                positions.push(center);
                positions.push([center[0] + half, center[1], center[2]]);
                positions.push([center[0], center[1] + half, center[2]]);
                indices.push([n, n + 1, n + 2]);
            }
        }
    }
    TriMesh {
        positions,
        indices,
        normals: None,
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

fn triangle_aabb(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3]) -> Aabb {
    let min = [
        p0[0].min(p1[0]).min(p2[0]),
        p0[1].min(p1[1]).min(p2[1]),
        p0[2].min(p1[2]).min(p2[2]),
    ];
    let max = [
        p0[0].max(p1[0]).max(p2[0]),
        p0[1].max(p1[1]).max(p2[1]),
        p0[2].max(p1[2]).max(p2[2]),
    ];
    Aabb::new(min, max)
}

fn world_to_range(
    w_min: f32,
    w_max: f32,
    origin: f32,
    cell_size: f32,
    resolution: usize,
) -> (usize, usize) {
    let min_i = ((w_min - origin) / cell_size).floor().max(0.0) as usize;
    let max_i = ((w_max - origin) / cell_size).ceil().min(resolution as f32) as usize;
    (min_i.min(resolution.saturating_sub(1)), max_i)
}

fn point_triangle_distance_sq(p: [f32; 3], a: [f32; 3], b: [f32; 3], c: [f32; 3]) -> f32 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
    let ap = [p[0] - a[0], p[1] - a[1], p[2] - a[2]];
    let n = cross(ab, ac);
    let len_sq = n[0] * n[0] + n[1] * n[1] + n[2] * n[2];
    if len_sq < 1e-18 {
        return 0.0;
    }
    let t = (ap[0] * n[0] + ap[1] * n[1] + ap[2] * n[2]) / len_sq;
    let proj = [p[0] - n[0] * t, p[1] - n[1] * t, p[2] - n[2] * t];
    let d1 = dot(
        proj[0] - a[0],
        proj[1] - a[1],
        proj[2] - a[2],
        ab[0],
        ab[1],
        ab[2],
    );
    let d2 = dot(
        proj[0] - a[0],
        proj[1] - a[1],
        proj[2] - a[2],
        ac[0],
        ac[1],
        ac[2],
    );
    let ab_ab = dot(ab[0], ab[1], ab[2], ab[0], ab[1], ab[2]);
    let ac_ac = dot(ac[0], ac[1], ac[2], ac[0], ac[1], ac[2]);
    let ab_ac = dot(ab[0], ab[1], ab[2], ac[0], ac[1], ac[2]);
    let denom = ab_ab * ac_ac - ab_ac * ab_ac;
    let (u, v) = if denom < 1e-18 {
        (0.0, 0.0)
    } else {
        let u = (d1 * ac_ac - d2 * ab_ac) / denom;
        let v = (d2 * ab_ab - d1 * ab_ac) / denom;
        (u.clamp(0.0, 1.0), v.clamp(0.0, 1.0))
    };
    let w = (1.0_f32 - u - v).clamp(0.0, 1.0);
    let (u, v) = if w <= 0.0 {
        let u = u.clamp(0.0, 1.0);
        let v = 1.0 - u;
        (u, v)
    } else {
        (u, v)
    };
    let closest = [
        a[0] + u * ab[0] + v * ac[0],
        a[1] + u * ab[1] + v * ac[1],
        a[2] + u * ab[2] + v * ac[2],
    ];
    let dx = p[0] - closest[0];
    let dy = p[1] - closest[1];
    let dz = p[2] - closest[2];
    dx * dx + dy * dy + dz * dz
}

fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot(a0: f32, a1: f32, a2: f32, b0: f32, b1: f32, b2: f32) -> f32 {
    a0 * b0 + a1 * b1 + a2 * b2
}
