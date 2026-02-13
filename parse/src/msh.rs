//! Gmsh MSH parser (ASCII v2/v4).
//!
//! Extracts nodes and tetrahedral elements (type 4). Other element types are skipped.

use crate::error::ParseError;

/// Tetrahedral mesh data from a MSH file.
#[derive(Clone, Debug, Default)]
pub struct MshData {
    /// Node positions (x, y, z).
    pub positions: Vec<[f32; 3]>,
    /// Tetrahedra as 4 node indices (1-based in file, we convert to 0-based).
    pub tets: Vec<[usize; 4]>,
}

/// Parses Gmsh MSH ASCII from bytes.
///
/// Looks for `$Nodes` / `$EndNodes` and `$Elements` / `$EndElements`. Node IDs can be 1-based;
/// output indices are 0-based. Only element type 4 (4-node tetrahedron) is read.
///
/// # Errors
/// Returns [`ParseError`](crate::ParseError) on invalid or truncated data.
pub fn parse(data: &[u8]) -> Result<MshData, ParseError> {
    let s = std::str::from_utf8(data).map_err(|e| ParseError::Io(e.to_string()))?;
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut tets: Vec<[usize; 4]> = Vec::new();
    let mut id_to_idx: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();

    let mut lines = s.lines().map(str::trim);
    while let Some(line) = lines.next() {
        if line == "$Nodes" || line == "$Node" {
            let count_line = lines
                .next()
                .ok_or_else(|| ParseError::Io("msh: expected node count".to_string()))?;
            let count: usize = count_line
                .split_whitespace()
                .next()
                .and_then(|w| w.parse().ok())
                .ok_or_else(|| ParseError::Io("msh: bad node count".to_string()))?;
            for _ in 0..count {
                let l = lines
                    .next()
                    .ok_or_else(|| ParseError::Io("msh: truncated nodes".to_string()))?;
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 4 {
                    let id: usize = parts[0]
                        .parse()
                        .map_err(|_| ParseError::Io("msh: bad node id".to_string()))?;
                    let x: f32 = parts[1]
                        .parse()
                        .map_err(|_| ParseError::Io("msh: bad x".to_string()))?;
                    let y: f32 = parts[2]
                        .parse()
                        .map_err(|_| ParseError::Io("msh: bad y".to_string()))?;
                    let z: f32 = parts[3]
                        .parse()
                        .map_err(|_| ParseError::Io("msh: bad z".to_string()))?;
                    let idx = positions.len();
                    id_to_idx.insert(id, idx);
                    positions.push([x, y, z]);
                }
            }
        }
        if line == "$Elements" || line == "$Element" {
            let count_line = lines
                .next()
                .ok_or_else(|| ParseError::Io("msh: expected element count".to_string()))?;
            let _count: usize = count_line
                .split_whitespace()
                .next()
                .and_then(|w| w.parse().ok())
                .unwrap_or(0);
            while let Some(l) = lines.next() {
                if l == "$EndElements" || l == "$EndElement" {
                    break;
                }
                let parts: Vec<&str> = l.split_whitespace().collect();
                if parts.len() >= 6 {
                    let el_type: usize = parts[1].parse().unwrap_or(0);
                    if el_type == 4 {
                        let n_tags: usize = parts[2].parse().unwrap_or(0);
                        let base = 3 + n_tags;
                        if parts.len() >= base + 4 {
                            let n0: usize = parts[base]
                                .parse()
                                .map_err(|_| ParseError::Io("msh: bad node id".to_string()))?;
                            let n1: usize = parts[base + 1]
                                .parse()
                                .map_err(|_| ParseError::Io("msh: bad node id".to_string()))?;
                            let n2: usize = parts[base + 2]
                                .parse()
                                .map_err(|_| ParseError::Io("msh: bad node id".to_string()))?;
                            let n3: usize = parts[base + 3]
                                .parse()
                                .map_err(|_| ParseError::Io("msh: bad node id".to_string()))?;
                            let i0 = *id_to_idx.get(&n0).unwrap_or(&0);
                            let i1 = *id_to_idx.get(&n1).unwrap_or(&0);
                            let i2 = *id_to_idx.get(&n2).unwrap_or(&0);
                            let i3 = *id_to_idx.get(&n3).unwrap_or(&0);
                            tets.push([i0, i1, i2, i3]);
                        }
                    }
                }
            }
        }
    }

    Ok(MshData { positions, tets })
}
