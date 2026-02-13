//! Indexed triangle mesh type and conversions.

use crate::GeometryError;

/// Lightweight indexed triangle mesh used throughout the geometry crate.
///
/// Positions and optional normals are stored per vertex; each face is a triangle
/// given by three vertex indices.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TriMesh {
    /// Vertex positions (x, y, z).
    pub positions: Vec<[f32; 3]>,
    /// Triangle indices: each element is three vertex indices.
    pub indices: Vec<[u32; 3]>,
    /// Optional vertex normals (same length as `positions` when present).
    pub normals: Option<Vec<[f32; 3]>>,
}

impl TriMesh {
    /// Creates an empty mesh.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a mesh from positions and triangle indices.
    ///
    /// # Examples
    ///
    /// ```
    /// # use geometry::TriMesh;
    /// let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
    /// let indices = vec![[0u32, 1, 2]];
    /// let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
    /// assert_eq!(mesh.num_vertices(), 3);
    /// assert_eq!(mesh.num_triangles(), 1);
    /// ```
    ///
    /// # Errors
    /// Returns `GeometryError::EmptyInput` if `positions` or `indices` is empty.
    /// Returns `GeometryError::InvalidTopology` if any index is out of bounds.
    pub fn from_positions_and_indices(
        positions: Vec<[f32; 3]>,
        indices: Vec<[u32; 3]>,
    ) -> Result<Self, GeometryError> {
        if positions.is_empty() {
            return Err(GeometryError::EmptyInput);
        }
        if indices.is_empty() {
            return Err(GeometryError::EmptyInput);
        }
        let n = positions.len() as u32;
        for tri in &indices {
            if tri[0] >= n || tri[1] >= n || tri[2] >= n {
                return Err(GeometryError::InvalidTopology(format!(
                    "index out of bounds: {:?} (max index {})",
                    tri,
                    n - 1
                )));
            }
        }
        Ok(Self {
            positions,
            indices,
            normals: None,
        })
    }

    /// Number of vertices.
    #[must_use]
    pub fn num_vertices(&self) -> usize {
        self.positions.len()
    }

    /// Number of triangles.
    #[must_use]
    pub fn num_triangles(&self) -> usize {
        self.indices.len()
    }

    /// Computes vertex normals by averaging face normals.
    ///
    /// Overwrites `normals` if already present. Does not normalize (caller may normalize).
    pub fn compute_normals(&mut self) {
        let n_verts = self.positions.len();
        let mut normals = vec![[0.0_f32; 3]; n_verts];
        for tri in &self.indices {
            let i0 = tri[0] as usize;
            let i1 = tri[1] as usize;
            let i2 = tri[2] as usize;
            let p0 = self.positions[i0];
            let p1 = self.positions[i1];
            let p2 = self.positions[i2];
            let e1 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
            let e2 = [p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]];
            let nx = e1[1] * e2[2] - e1[2] * e2[1];
            let ny = e1[2] * e2[0] - e1[0] * e2[2];
            let nz = e1[0] * e2[1] - e1[1] * e2[0];
            normals[i0][0] += nx;
            normals[i0][1] += ny;
            normals[i0][2] += nz;
            normals[i1][0] += nx;
            normals[i1][1] += ny;
            normals[i1][2] += nz;
            normals[i2][0] += nx;
            normals[i2][1] += ny;
            normals[i2][2] += nz;
        }
        for n in &mut normals {
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            if len > 1e-8 {
                n[0] /= len;
                n[1] /= len;
                n[2] /= len;
            }
        }
        self.normals = Some(normals);
    }

    /// Converts this mesh to render-friendly vertices and indices (position, uv, color).
    ///
    /// Expands to one vertex per triangle corner; each vertex has position, uv (0,0), and
    /// color (white). Use this in the demo crate to build render `Vertex` buffers.
    #[must_use]
    pub fn to_render_vertices(&self) -> (Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<[f32; 4]>, Vec<u32>) {
        let mut positions = Vec::with_capacity(self.indices.len() * 3);
        let mut uvs = Vec::with_capacity(self.indices.len() * 3);
        let mut colors = Vec::with_capacity(self.indices.len() * 3);
        let mut indices = Vec::with_capacity(self.indices.len() * 3);
        let _normals = self.normals.as_deref();
        let white = [1.0_f32, 1.0, 1.0, 1.0];
        for (idx, tri) in self.indices.iter().enumerate() {
            for k in 0..3 {
                let i = tri[k] as usize;
                positions.push(self.positions[i]);
                uvs.push([0.0, 0.0]);
                colors.push(white);
                indices.push((idx * 3 + k) as u32);
            }
        }
        (positions, uvs, colors, indices)
    }
}

#[cfg(feature = "parse")]
impl TryFrom<&parse::mesh::Mesh> for TriMesh {
    type Error = GeometryError;

    fn try_from(m: &parse::mesh::Mesh) -> Result<Self, Self::Error> {
        if m.vertices.is_empty() {
            return Err(GeometryError::EmptyInput);
        }
        if m.vertices.len() % 3 != 0 {
            return Err(GeometryError::InvalidTopology(
                "parse::Mesh vertex count not multiple of 3".to_string(),
            ));
        }
        let positions: Vec<[f32; 3]> = m
            .vertices
            .iter()
            .map(|v| {
                let c = &v.position.coords;
                [c.get(0), c.get(1), c.get(2)]
            })
            .collect();
        let normals: Vec<[f32; 3]> = m
            .vertices
            .iter()
            .map(|v| [v.normal.get(0), v.normal.get(1), v.normal.get(2)])
            .collect();
        let mut indices = Vec::with_capacity(positions.len() / 3);
        for i in (0..positions.len()).step_by(3) {
            indices.push([i as u32, (i + 1) as u32, (i + 2) as u32]);
        }
        Ok(Self {
            positions,
            indices,
            normals: Some(normals),
        })
    }
}

#[cfg(feature = "parse")]
impl TryFrom<TriMesh> for parse::mesh::Mesh {
    type Error = GeometryError;

    fn try_from(m: TriMesh) -> Result<Self, Self::Error> {
        use mathlib::{Point3, Vector3f};
        let mut vertices = Vec::with_capacity(m.indices.len() * 3);
        let normals = m.normals.as_deref();
        for tri in &m.indices {
            for k in 0..3 {
                let i = tri[k] as usize;
                let pos = m.positions[i];
                let normal = normals
                    .and_then(|n| n.get(i).copied())
                    .unwrap_or([0.0, 1.0, 0.0]);
                let normal_vec = Vector3f::from_slice(&[normal[0], normal[1], normal[2]]);
                vertices.push(parse::mesh::Vertex::new(
                    Point3::new(pos[0], pos[1], pos[2]),
                    normal_vec,
                    (0.0, 0.0),
                ));
            }
        }
        Ok(parse::mesh::Mesh {
            name: "geometry".to_string(),
            vertices,
            material: None,
        })
    }
}
