//! glTF / GLB parser using the gltf crate.
//!
//! Extracts meshes with POSITION, NORMAL, TEXCOORD_0, indices.
//! TRIANGLES mode only; non-indexed meshes get sequential indices.

use crate::error::ParseError;
use crate::mesh::{Mesh, Vertex};
use gltf::mesh::util::ReadIndices;
use mathlib::{Point3, Vector3f};
use std::collections::HashMap;

/// glTF parse result: meshes keyed by name.
pub type GltfData = HashMap<String, Mesh>;

/// Parses glTF or GLB from bytes.
///
/// TRIANGLES primitives only. Non-indexed meshes generate sequential indices.
/// Missing normals default to (0,1,0). Missing UVs default to (0,0).
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid glTF.
pub fn parse(data: &[u8]) -> Result<GltfData, ParseError> {
    let (document, buffers, _images) =
        gltf::import_slice(data).map_err(|e| ParseError::Io(e.to_string()))?;

    let mut result = GltfData::new();
    let _default_normal = {
        let mut v = Vector3f::with_capacity(3);
        v.set(0, 0.0);
        v.set(1, 1.0);
        v.set(2, 0.0);
        v
    };

    for mesh in document.meshes() {
        let mesh_name = mesh.name().unwrap_or("").to_string();
        let mut vertices = Vec::new();

        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                continue;
            }

            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .map(|it| it.collect())
                .unwrap_or_default();
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .map(|it| it.collect())
                .unwrap_or_default();
            let uvs: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .map(|iter| iter.into_f32().collect())
                .unwrap_or_default();

            let indices: Vec<u32> = match reader.read_indices() {
                Some(ReadIndices::U8(it)) => it.map(u32::from).collect(),
                Some(ReadIndices::U16(it)) => it.map(u32::from).collect(),
                Some(ReadIndices::U32(it)) => it.collect(),
                None => (0..positions.len() as u32).collect(),
            };

            for tri in indices.chunks_exact(3) {
                let [i0, i1, i2] = [tri[0] as usize, tri[1] as usize, tri[2] as usize];
                for &i in &[i0, i1, i2] {
                    let pos = positions.get(i).copied().unwrap_or([0.0, 0.0, 0.0]);
                    let n = normals.get(i).copied().unwrap_or([0.0, 1.0, 0.0]);
                    let uv = uvs.get(i).copied().unwrap_or([0.0, 0.0]);

                    let mut normal_vec = Vector3f::with_capacity(3);
                    normal_vec.set(0, n[0]);
                    normal_vec.set(1, n[1]);
                    normal_vec.set(2, n[2]);

                    vertices.push(Vertex {
                        position: Point3::new(pos[0], pos[1], pos[2]),
                        normal: normal_vec,
                        uv: (uv[0], uv[1]),
                        tangent: None,
                    });
                }
            }
        }

        if !vertices.is_empty() {
            let name = if mesh_name.is_empty() {
                format!("mesh_{}", result.len())
            } else {
                mesh_name
            };
            result.insert(
                name.clone(),
                Mesh {
                    name,
                    vertices,
                    material: None,
                },
            );
        }
    }

    Ok(result)
}
