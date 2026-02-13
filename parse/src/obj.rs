//! Wavefront OBJ parser.
//!
//! Hand-roll: vertices, normals, UVs, faces (triangle fan), groups, mtllib, usemtl.

use crate::error::ParseError;
use crate::mesh::{Mesh, Vertex};
use crate::parser::Parser;
use mathlib::{Point3, Vector3f};
use std::path::Path;

/// OBJ parse result: meshes and optional MTL path.
#[derive(Clone, Debug)]
pub struct ObjData {
    /// Meshes.
    pub meshes: Vec<Mesh>,
    /// MTL library path (from mtllib).
    pub mtl_path: Option<String>,
}

/// Parses OBJ from bytes.
///
/// Vertices, normals, UVs are 1-indexed in the file. Face triangulation uses
/// triangle fan (first + (i-1, i) for i in 2..n).
///
/// # Errors
///
/// Returns [`ParseError`](crate::ParseError) on invalid OBJ.
pub fn parse(data: &[u8], base_path: Option<&Path>) -> Result<ObjData, ParseError> {
    let s = std::str::from_utf8(data).map_err(|e| ParseError::Io(e.to_string()))?;
    parse_str(s, base_path)
}

/// Parses OBJ from string.
pub fn parse_str(s: &str, base_path: Option<&Path>) -> Result<ObjData, ParseError> {
    let mut p = Parser::new(s.as_bytes(), None);

    let mut vertices: Vec<Point3> = vec![Point3::new(0.0, 0.0, 0.0)];
    let mut normals: Vec<Vector3f> = vec![{
        let mut v = Vector3f::with_capacity(3);
        v.set(0, 0.0);
        v.set(1, 1.0);
        v.set(2, 0.0);
        v
    }];
    let mut uvs: Vec<(f32, f32)> = vec![(0.0, 0.0)];

    let mut meshes: Vec<Mesh> = Vec::new();
    let mut current_name = "default".to_string();
    let mut current_material: Option<String> = None;
    let mut current_vertices: Vec<Vertex> = Vec::new();
    let mut mtl_path: Option<String> = None;

    fn flush_mesh(
        meshes: &mut Vec<Mesh>,
        name: &str,
        mat: Option<String>,
        verts: &mut Vec<Vertex>,
    ) {
        if !verts.is_empty() {
            meshes.push(Mesh {
                name: name.to_string(),
                vertices: std::mem::take(verts),
                material: mat.map(|m| crate::Material {
                    name: m,
                    ..Default::default()
                }),
            });
        }
    }

    while !p.is_eof() {
        p.parse_whitespace();
        if p.match_char(b'\n') {
            p.advance();
            continue;
        }
        if p.match_char(b'#') {
            while !p.is_eof() && !p.match_char(b'\n') {
                p.advance();
            }
            continue;
        }
        if p.is_eof() {
            break;
        }

        let c = p.peek();
        if c == b'v' {
            p.advance();
            if p.peek() == b' ' || p.peek() == b'\t' {
                p.parse_whitespace();
                let x = parse_f32(&mut p).ok_or_else(|| p.syntax_err("expected number"))?;
                p.parse_whitespace();
                let y = parse_f32(&mut p).ok_or_else(|| p.syntax_err("expected number"))?;
                p.parse_whitespace();
                let z = parse_f32(&mut p).ok_or_else(|| p.syntax_err("expected number"))?;
                vertices.push(Point3::new(x, y, z));
            } else if p.peek() == b'n' {
                p.advance();
                p.parse_whitespace();
                let x = parse_f32(&mut p).ok_or_else(|| p.syntax_err("expected number"))?;
                p.parse_whitespace();
                let y = parse_f32(&mut p).ok_or_else(|| p.syntax_err("expected number"))?;
                p.parse_whitespace();
                let z = parse_f32(&mut p).ok_or_else(|| p.syntax_err("expected number"))?;
                let mut v = Vector3f::with_capacity(3);
                v.set(0, x);
                v.set(1, y);
                v.set(2, z);
                normals.push(v);
            } else if p.peek() == b't' {
                p.advance();
                p.parse_whitespace();
                let u = parse_f32(&mut p).ok_or_else(|| p.syntax_err("expected number"))?;
                p.parse_whitespace();
                let v = parse_f32(&mut p).unwrap_or(0.0);
                uvs.push((u, v));
            } else {
                skip_rest_of_line(&mut p);
            }
        } else if c == b'f' {
            p.advance();
            p.parse_whitespace();
            let face = parse_face(&mut p, &vertices, &normals, &uvs)?;
            for v in face {
                current_vertices.push(v);
            }
        } else if c == b'g' || c == b'o' {
            let _g_or_o = p.peek();
            p.advance();
            p.parse_whitespace();
            let name = parse_rest_of_line(&mut p)?.trim().to_string();
            let name = if name.is_empty() {
                format!("{}_{}", current_name, meshes.len())
            } else {
                name
            };
            flush_mesh(
                &mut meshes,
                &current_name,
                current_material.clone(),
                &mut current_vertices,
            );
            current_name = name;
        } else if c == b'u' {
            if p.remaining().starts_with(b"usemtl ") {
                p.advance_n(7);
                p.parse_whitespace();
                let mat = parse_rest_of_line(&mut p)?.trim().to_string();
                if !mat.is_empty() {
                    flush_mesh(
                        &mut meshes,
                        &current_name,
                        current_material.clone(),
                        &mut current_vertices,
                    );
                    current_material = Some(mat);
                }
            } else {
                skip_rest_of_line(&mut p);
            }
        } else if c == b'm' {
            if p.remaining().starts_with(b"mtllib ") {
                p.advance_n(7);
                p.parse_whitespace();
                let path = parse_rest_of_line(&mut p)?.trim().to_string();
                if !path.is_empty() {
                    mtl_path = Some(if let Some(base) = base_path {
                        base.join(&path).to_string_lossy().to_string()
                    } else {
                        path
                    });
                }
            } else {
                skip_rest_of_line(&mut p);
            }
        } else {
            skip_rest_of_line(&mut p);
        }
    }

    flush_mesh(
        &mut meshes,
        &current_name,
        current_material,
        &mut current_vertices,
    );

    Ok(ObjData { meshes, mtl_path })
}

fn parse_face(
    p: &mut Parser<'_>,
    vertices: &[Point3],
    normals: &[Vector3f],
    uvs: &[(f32, f32)],
) -> Result<Vec<Vertex>, ParseError> {
    let mut indices: Vec<(usize, usize, usize)> = Vec::new();
    while !p.is_eof() && !p.match_char(b'\n') {
        p.parse_whitespace();
        if p.is_eof() || p.match_char(b'\n') {
            break;
        }
        let v_idx = parse_index(p, vertices.len())?;
        if p.peek() == b'/' {
            p.advance();
            if p.peek() == b'/' {
                p.advance();
                let vn = parse_index(p, normals.len())?;
                indices.push((v_idx, 0, vn));
                continue;
            }
            let vt = parse_index(p, uvs.len())?;
            let vn = if p.peek() == b'/' {
                p.advance();
                parse_index(p, normals.len())?
            } else {
                0
            };
            indices.push((v_idx, vt, vn));
        } else {
            indices.push((v_idx, 0, 0));
        }
    }

    if indices.len() < 3 {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();
    for i in 1..(indices.len() - 1) {
        let (a, b, c) = (indices[0], indices[i], indices[i + 1]);
        for &(vi, vti, vni) in &[a, b, c] {
            let pos = vertices[vi].clone();
            let n = normals[vni].clone();
            let uv = uvs[vti];
            result.push(Vertex {
                position: pos,
                normal: n,
                uv,
                tangent: None,
            });
        }
    }
    Ok(result)
}

fn parse_index(p: &mut Parser<'_>, max_len: usize) -> Result<usize, ParseError> {
    let neg = p.peek() == b'-';
    if neg {
        p.advance();
    }
    let mut v: usize = 0;
    while p.peek().is_ascii_digit() {
        v = v * 10 + (p.peek() - b'0') as usize;
        p.advance();
    }
    if v == 0 {
        return Err(p.syntax_err("invalid vertex index 0"));
    }
    let v = if neg { max_len.saturating_sub(v) } else { v };
    if v >= max_len {
        return Err(p.syntax_err(format!("vertex index {} out of range", v + 1)));
    }
    Ok(v)
}

fn parse_f32(p: &mut Parser<'_>) -> Option<f32> {
    let start = p.offset;
    if p.peek() == b'-' {
        p.advance();
    }
    while p.peek().is_ascii_digit() {
        p.advance();
    }
    if p.peek() == b'.' {
        p.advance();
        while p.peek().is_ascii_digit() {
            p.advance();
        }
    }
    if p.peek() == b'e' || p.peek() == b'E' {
        p.advance();
        if p.peek() == b'+' || p.peek() == b'-' {
            p.advance();
        }
        while p.peek().is_ascii_digit() {
            p.advance();
        }
    }
    let slice = &p.data[start..p.offset];
    std::str::from_utf8(slice).ok()?.parse().ok()
}

fn parse_rest_of_line(p: &mut Parser<'_>) -> Result<String, ParseError> {
    let start = p.offset;
    while !p.is_eof() && !p.match_char(b'\n') {
        p.advance();
    }
    Ok(std::str::from_utf8(&p.data[start..p.offset])
        .map_err(|e| ParseError::Io(e.to_string()))?
        .to_string())
}

fn skip_rest_of_line(p: &mut Parser<'_>) {
    while !p.is_eof() && !p.match_char(b'\n') {
        p.advance();
    }
}

trait IsAsciiDigit {
    fn is_ascii_digit(self) -> bool;
}
impl IsAsciiDigit for u8 {
    fn is_ascii_digit(self) -> bool {
        (b'0'..=b'9').contains(&self)
    }
}
