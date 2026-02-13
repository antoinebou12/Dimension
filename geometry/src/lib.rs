//! geometry — Mesh processing: voxelization, tetrahedralization, CSG, remeshing, smoothing.
//!
//! This crate provides:
//! - **[`TriMesh`](mesh::TriMesh)** — Indexed triangle mesh with conversions to/from parse and render.
//! - **Voxelization** — Surface voxelization, flood fill, marching cubes.
//! - **Tetrahedralization** — Grid and surface-based tet meshes.
//! - **CSG** — Boolean operations (union, intersection, difference) on watertight meshes.
//! - **Remeshing** — Isotropic remeshing (split/collapse/flip/smooth).
//! - **Smoothing** — Laplacian diffusion (uniform, cotangent, implicit).

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]

pub mod csg;
pub mod error;
pub mod half_edge;
pub mod mesh;
pub mod remesh;
pub mod smooth;
pub mod tet;
pub mod voxel;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use csg::{csg_difference, csg_intersection, csg_union, CsgOp};
pub use error::GeometryError;
pub use half_edge::{FaceId, HalfEdge, HalfEdgeId, HalfEdgeMesh, HeFace, HeVertex, VertexId};
pub use mesh::TriMesh;
pub use remesh::remesh_isotropic;
pub use smooth::{smooth_cotangent, smooth_implicit, smooth_uniform};
pub use tet::TetMesh;
pub use voxel::{flood_fill, marching_cubes, voxelize_mesh, VoxelGrid};

/// Prelude: commonly used types.
pub mod prelude {
    pub use crate::{GeometryError, HalfEdgeMesh, TriMesh};
}
