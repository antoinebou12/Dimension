//! STL parser (binary and ASCII).
//!
//! Auto-detects format: ASCII starts with "solid "; otherwise binary (80-byte header, 4-byte count, 50 bytes per triangle).

use crate::error::ParseError;
use crate::mesh::{Mesh, Vertex};
use mathlib::{Point3, Vector3f};
use std::io::Cursor;

/// Parses STL from bytes (auto-detects ASCII vs binary).
///
/// Binary: 80-byte header, 4-byte little-endian triangle count, then 50 bytes per triangle.
/// ASCII: starts with "solid ", contains "facet" and "vertex" lines.
///
/// # Errors
/// Returns [`ParseError`](crate::ParseError) on invalid or truncated data.
pub fn parse(data: &[u8]) -> Result<Mesh, ParseError> {
    if data.len() < 6 {
        return Err(ParseError::Io("stl: too short".to_string()));
    }
    let as_ascii = std::str::from_utf8(data).ok();
    let looks_ascii = data.starts_with(b"solid ")
        && as_ascii
            .map(|s| s.contains("facet") && s.contains("vertex"))
            .unwrap_or(false);
    if looks_ascii {
        return parse_ascii(data);
    }
    if data.len() >= 84 {
        let count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
        if count <= (data.len().saturating_sub(84)) / 50 {
            return parse_binary(data);
        }
    }
    parse_ascii(data)
}

fn parse_ascii(data: &[u8]) -> Result<Mesh, ParseError> {
    let s = std::str::from_utf8(data).map_err(|e| ParseError::Io(e.to_string()))?;
    let mut vertices = Vec::new();
    let mut lines = s.lines();
    while let Some(line) = lines.next() {
        let line = line.trim();
        if line.starts_with("vertex ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let x: f32 = parts[1]
                    .parse()
                    .map_err(|_| ParseError::Io("stl ascii: bad float".to_string()))?;
                let y: f32 = parts[2]
                    .parse()
                    .map_err(|_| ParseError::Io("stl ascii: bad float".to_string()))?;
                let z: f32 = parts[3]
                    .parse()
                    .map_err(|_| ParseError::Io("stl ascii: bad float".to_string()))?;
                let mut normal = Vector3f::with_capacity(3);
                normal.set(0, 0.0);
                normal.set(1, 1.0);
                normal.set(2, 0.0);
                vertices.push(Vertex::new(Point3::new(x, y, z), normal, (0.0, 0.0)));
            }
        }
    }
    Ok(Mesh {
        name: "stl".to_string(),
        vertices,
        material: None,
    })
}

fn parse_binary(data: &[u8]) -> Result<Mesh, ParseError> {
    let mut r = Cursor::new(data);
    r.set_position(80);
    let mut count_buf = [0u8; 4];
    std::io::Read::read_exact(&mut r, &mut count_buf).map_err(|e| ParseError::Io(e.to_string()))?;
    let count = u32::from_le_bytes(count_buf) as usize;
    let mut vertices = Vec::with_capacity(count * 3);
    let mut normal = Vector3f::with_capacity(3);
    normal.set(0, 0.0);
    normal.set(1, 1.0);
    normal.set(2, 0.0);
    for _ in 0..count {
        let mut buf = [0f32; 12];
        for i in 0..12 {
            let mut b = [0u8; 4];
            std::io::Read::read_exact(&mut r, &mut b).map_err(|e| ParseError::Io(e.to_string()))?;
            buf[i] = f32::from_le_bytes(b);
        }
        let _attr: [u8; 2] = [0; 2];
        let mut attr = [0u8; 2];
        std::io::Read::read_exact(&mut r, &mut attr).map_err(|e| ParseError::Io(e.to_string()))?;
        vertices.push(Vertex::new(
            Point3::new(buf[3], buf[4], buf[5]),
            normal.clone(),
            (0.0, 0.0),
        ));
        vertices.push(Vertex::new(
            Point3::new(buf[6], buf[7], buf[8]),
            normal.clone(),
            (0.0, 0.0),
        ));
        vertices.push(Vertex::new(
            Point3::new(buf[9], buf[10], buf[11]),
            normal.clone(),
            (0.0, 0.0),
        ));
    }
    Ok(Mesh {
        name: "stl".to_string(),
        vertices,
        material: None,
    })
}
