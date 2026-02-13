//! PLY parser using ply-rs.
//!
//! Extracts vertex positions, normals, UVs and faces into Mesh/Vertex.

use crate::error::ParseError;
use crate::mesh::{Mesh, Vertex};
use mathlib::{Point3, Vector3f};
use ply_rs::parser::Parser;
use ply_rs::ply::{DefaultElement, Property};
use std::io::Cursor;

/// Parses PLY from bytes.
///
/// Extracts vertex element (x, y, z; nx, ny, nz; s, t) and face element (vertex_indices).
/// Triangulates faces (triangle fan from first vertex).
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid PLY.
pub fn parse(data: &[u8]) -> Result<Mesh, ParseError> {
    let mut cursor = Cursor::new(data);
    let parser = Parser::<DefaultElement>::new();
    let ply = parser
        .read_ply(&mut cursor)
        .map_err(|e| ParseError::Io(e.to_string()))?;

    let payload = &ply.payload;
    let vertices_raw = payload
        .get("vertex")
        .or_else(|| payload.get("Vertex"))
        .ok_or_else(|| ParseError::Syntax {
            filename: None,
            row: 0,
            col: 0,
            msg: "PLY has no vertex element".to_string(),
        })?;

    let mut positions: Vec<Point3> = Vec::new();
    let mut normals: Vec<Vector3f> = Vec::new();
    let mut uvs: Vec<(f32, f32)> = Vec::new();
    let default_normal = {
        let mut v = Vector3f::with_capacity(3);
        v.set(0, 0.0);
        v.set(1, 1.0);
        v.set(2, 0.0);
        v
    };

    for elem in vertices_raw {
        let x = get_f32(elem, "x").unwrap_or(0.0);
        let y = get_f32(elem, "y").unwrap_or(0.0);
        let z = get_f32(elem, "z").unwrap_or(0.0);
        positions.push(Point3::new(x, y, z));

        let nx = get_f32(elem, "nx").unwrap_or(0.0);
        let ny = get_f32(elem, "ny").unwrap_or(1.0);
        let nz = get_f32(elem, "nz").unwrap_or(0.0);
        let mut n = Vector3f::with_capacity(3);
        n.set(0, nx);
        n.set(1, ny);
        n.set(2, nz);
        normals.push(n);

        let s = get_f32(elem, "s")
            .or_else(|| get_f32(elem, "u"))
            .unwrap_or(0.0);
        let t = get_f32(elem, "t")
            .or_else(|| get_f32(elem, "v"))
            .unwrap_or(0.0);
        uvs.push((s, t));
    }

    let mut result_vertices: Vec<Vertex> = Vec::new();
    let faces_raw = payload.get("face").or_else(|| payload.get("Face"));

    if let Some(faces) = faces_raw {
        for elem in faces {
            let idx = get_vertex_indices(elem);
            for tri in triangulate_face(&idx) {
                for i in tri {
                    let pos = positions
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| Point3::new(0.0, 0.0, 0.0));
                    let n = normals
                        .get(i)
                        .cloned()
                        .unwrap_or_else(|| default_normal.clone());
                    let uv = uvs.get(i).copied().unwrap_or((0.0, 0.0));
                    result_vertices.push(Vertex {
                        position: pos,
                        normal: n,
                        uv,
                        tangent: None,
                    });
                }
            }
        }
    } else {
        for i in 0..positions.len() {
            let pos = positions
                .get(i)
                .cloned()
                .unwrap_or_else(|| Point3::new(0.0, 0.0, 0.0));
            let n = normals
                .get(i)
                .cloned()
                .unwrap_or_else(|| default_normal.clone());
            let uv = uvs.get(i).copied().unwrap_or((0.0, 0.0));
            result_vertices.push(Vertex {
                position: pos,
                normal: n,
                uv,
                tangent: None,
            });
        }
    }

    Ok(Mesh {
        name: "ply_mesh".to_string(),
        vertices: result_vertices,
        material: None,
    })
}

fn get_f32(elem: &DefaultElement, key: &str) -> Option<f32> {
    match elem.get(key)? {
        Property::Float(v) => Some(*v),
        Property::Double(v) => Some(*v as f32),
        _ => None,
    }
}

fn get_vertex_indices(elem: &DefaultElement) -> Vec<usize> {
    let list = match elem
        .get("vertex_indices")
        .or_else(|| elem.get("vertex_index"))
    {
        Some(Property::ListInt(v)) => v.iter().map(|&i| i as usize).collect(),
        Some(Property::ListUInt(v)) => v.iter().map(|&i| i as usize).collect(),
        _ => Vec::new(),
    };
    list
}

fn triangulate_face(indices: &[usize]) -> Vec<[usize; 3]> {
    let mut tri = Vec::new();
    for i in 1..indices.len().saturating_sub(1) {
        tri.push([indices[0], indices[i], indices[i + 1]]);
    }
    tri
}
