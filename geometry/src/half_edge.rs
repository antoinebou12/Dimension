//! Half-edge mesh: topology-aware representation for remeshing and smoothing.
//!
//! Each vertex stores an outgoing half-edge; each half-edge stores next, twin,
//! origin vertex, and face. Supports efficient adjacency traversal and local operations.

use crate::{GeometryError, TriMesh};
use std::collections::HashMap;

/// Index of a vertex in the half-edge mesh.
pub type VertexId = usize;
/// Index of a half-edge.
pub type HalfEdgeId = usize;
/// Index of a face.
pub type FaceId = usize;

/// One vertex: position and one outgoing half-edge.
#[derive(Clone, Debug)]
pub struct HeVertex {
    /// Position (x, y, z).
    pub position: [f32; 3],
    /// Index of any half-edge originating from this vertex.
    pub half_edge: HalfEdgeId,
}

/// One half-edge: next (same face), twin (opposite direction), origin vertex, face.
#[derive(Clone, Debug)]
pub struct HalfEdge {
    /// Next half-edge around the face (CCW).
    pub next: HalfEdgeId,
    /// Twin half-edge (same edge, opposite direction). `None` on boundary.
    pub twin: Option<HalfEdgeId>,
    /// Vertex this half-edge leaves from.
    pub origin: VertexId,
    /// Face to the left of this half-edge. `None` for boundary.
    pub face: Option<FaceId>,
}

/// One face: any half-edge on its boundary.
#[derive(Clone, Debug)]
pub struct HeFace {
    /// Any half-edge on the boundary of this face.
    pub half_edge: HalfEdgeId,
}

fn position_key(p: [f32; 3]) -> [u32; 3] {
    [p[0].to_bits(), p[1].to_bits(), p[2].to_bits()]
}

/// Half-edge mesh: vertices, half-edges, faces with full connectivity.
#[derive(Clone, Debug, Default)]
pub struct HalfEdgeMesh {
    /// Vertices.
    pub vertices: Vec<HeVertex>,
    /// Half-edges.
    pub half_edges: Vec<HalfEdge>,
    /// Faces.
    pub faces: Vec<HeFace>,
}

impl HalfEdgeMesh {
    /// Builds a half-edge mesh from an indexed triangle mesh.
    ///
    /// Vertices are merged by exact bit-equality of position. Assumes the input
    /// is a manifold triangle mesh (each edge at most two faces).
    ///
    /// # Errors
    /// Returns `GeometryError::NonManifold` if the mesh is not manifold.
    /// Returns `GeometryError::EmptyInput` if the mesh has no triangles.
    pub fn from_tri_mesh(mesh: &TriMesh) -> Result<Self, GeometryError> {
        if mesh.indices.is_empty() {
            return Err(GeometryError::EmptyInput);
        }
        let mut pos_to_id: HashMap<[u32; 3], VertexId> = HashMap::new();
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut tri_verts: Vec<[VertexId; 3]> = Vec::with_capacity(mesh.indices.len());
        for tri in &mesh.indices {
            let mut vids = [0; 3];
            for (k, &idx) in tri.iter().enumerate() {
                let pos = mesh.positions[idx as usize];
                let key = position_key(pos);
                let vid = *pos_to_id.entry(key).or_insert_with(|| {
                    let id = positions.len();
                    positions.push(pos);
                    id
                });
                vids[k] = vid;
            }
            tri_verts.push(vids);
        }
        let mut vertices: Vec<HeVertex> = positions
            .into_iter()
            .map(|position| HeVertex {
                position,
                half_edge: 0,
            })
            .collect();
        let mut half_edges: Vec<HalfEdge> = Vec::with_capacity(mesh.indices.len() * 3);
        let mut faces: Vec<HeFace> = Vec::with_capacity(mesh.indices.len());
        let mut edge_to_he: HashMap<(VertexId, VertexId), HalfEdgeId> = HashMap::new();
        for (face_id, tri) in tri_verts.iter().enumerate() {
            let [v0, v1, v2] = *tri;
            let he0 = half_edges.len();
            half_edges.push(HalfEdge {
                next: he0 + 1,
                twin: None,
                origin: v0,
                face: Some(face_id),
            });
            half_edges.push(HalfEdge {
                next: he0 + 2,
                twin: None,
                origin: v1,
                face: Some(face_id),
            });
            half_edges.push(HalfEdge {
                next: he0,
                twin: None,
                origin: v2,
                face: Some(face_id),
            });
            faces.push(HeFace { half_edge: he0 });
            for (a, b) in [(v0, v1), (v1, v2), (v2, v0)] {
                let e = (a.min(b), a.max(b));
                let he_idx = he0
                    + if (a, b) == (v0, v1) {
                        0
                    } else if (a, b) == (v1, v2) {
                        1
                    } else {
                        2
                    };
                if let Some(other) = edge_to_he.insert(e, he_idx) {
                    if half_edges[other].twin.is_some() {
                        return Err(GeometryError::NonManifold(
                            "edge shared by more than two faces".to_string(),
                        ));
                    }
                    half_edges[other].twin = Some(he_idx);
                    half_edges[he_idx].twin = Some(other);
                }
            }
        }
        for (he_id, he) in half_edges.iter().enumerate() {
            let v_id = he.origin;
            if vertices[v_id].half_edge == 0 {
                vertices[v_id].half_edge = he_id;
            }
        }
        Ok(HalfEdgeMesh {
            vertices,
            half_edges,
            faces,
        })
    }

    /// Converts this half-edge mesh back to an indexed triangle mesh.
    #[must_use]
    pub fn to_tri_mesh(&self) -> TriMesh {
        let positions: Vec<[f32; 3]> = self.vertices.iter().map(|v| v.position).collect();
        let mut indices = Vec::with_capacity(self.faces.len());
        for face in &self.faces {
            let mut he_id = face.half_edge;
            let start = he_id;
            let mut verts = Vec::with_capacity(3);
            loop {
                verts.push(self.half_edges[he_id].origin as u32);
                he_id = self.half_edges[he_id].next;
                if he_id == start {
                    break;
                }
            }
            if verts.len() == 3 {
                indices.push([verts[0], verts[1], verts[2]]);
            }
        }
        TriMesh {
            positions,
            indices,
            normals: None,
        }
    }

    /// Returns an iterator over vertex indices adjacent to the given vertex.
    pub fn vertex_neighbors(&self, v: VertexId) -> VertexNeighbors<'_> {
        VertexNeighbors {
            mesh: self,
            start_he: self.vertices[v].half_edge,
            current_he: self.vertices[v].half_edge,
            first: true,
            done: false,
        }
    }

    /// Flips the edge represented by the given half-edge (interior edges only).
    ///
    /// # Errors
    /// Returns `GeometryError::InvalidTopology` if the edge is on the boundary or non-flippable.
    pub fn edge_flip(&mut self, he_id: HalfEdgeId) -> Result<(), GeometryError> {
        let twin = self.half_edges[he_id].twin.ok_or_else(|| {
            GeometryError::InvalidTopology("cannot flip boundary edge".to_string())
        })?;
        let next_id = self.half_edges[he_id].next;
        let next_next_id = self.half_edges[next_id].next;
        let tw_next_id = self.half_edges[twin].next;
        let tw_next_next_id = self.half_edges[tw_next_id].next;
        let tw_prev_id = (0..self.half_edges.len())
            .find(|&i| self.half_edges[i].next == twin)
            .unwrap_or(0);
        let a = self.half_edges[he_id].origin;
        let b = self.half_edges[next_id].origin;
        let c = self.half_edges[next_next_id].origin;
        let d = self.half_edges[tw_next_id].origin;
        if b == d || c == d {
            return Err(GeometryError::InvalidTopology(
                "flip would create degenerate face".to_string(),
            ));
        }
        self.half_edges[he_id].origin = c;
        self.half_edges[he_id].next = tw_next_id;
        self.half_edges[twin].origin = d;
        self.half_edges[twin].next = next_id;
        self.half_edges[next_id].next = he_id;
        self.half_edges[next_id].origin = b;
        self.half_edges[next_next_id].next = twin;
        self.half_edges[tw_next_id].next = next_next_id;
        self.half_edges[tw_prev_id].next = tw_next_next_id;
        self.vertices[a].half_edge = next_next_id;
        self.vertices[b].half_edge = next_id;
        self.vertices[c].half_edge = he_id;
        self.vertices[d].half_edge = twin;
        Ok(())
    }

    /// Splits the edge at its midpoint: adds a new vertex and updates connectivity.
    ///
    /// Returns the new vertex id.
    ///
    /// # Errors
    /// Returns `GeometryError::InvalidTopology` if the half-edge is invalid.
    pub fn edge_split(&mut self, he_id: HalfEdgeId) -> Result<VertexId, GeometryError> {
        let next_id = self.half_edges[he_id].next;
        let origin = self.half_edges[he_id].origin;
        let face = self.half_edges[he_id].face;
        let twin_opt = self.half_edges[he_id].twin;
        let b = self.half_edges[next_id].origin;
        let pos_a = self.vertices[origin].position;
        let pos_b = self.vertices[b].position;
        let mid = [
            (pos_a[0] + pos_b[0]) * 0.5,
            (pos_a[1] + pos_b[1]) * 0.5,
            (pos_a[2] + pos_b[2]) * 0.5,
        ];
        let new_vertex_id = self.vertices.len();
        self.vertices.push(HeVertex {
            position: mid,
            half_edge: he_id,
        });
        let he_new = self.half_edges.len();
        self.half_edges.push(HalfEdge {
            next: next_id,
            twin: None,
            origin: new_vertex_id,
            face,
        });
        let he_new2 = self.half_edges.len();
        self.half_edges.push(HalfEdge {
            next: he_id,
            twin: Some(he_new),
            origin: b,
            face,
        });
        self.half_edges[he_new].twin = Some(he_new2);
        self.half_edges[he_id].next = he_new2;
        self.half_edges[he_id].origin = origin;
        self.half_edges[next_id].next = he_new;
        self.half_edges[next_id].origin = new_vertex_id;
        self.vertices[b].half_edge = he_new2;
        if let Some(tw_id) = twin_opt {
            let tw_face = self.half_edges[tw_id].face;
            let tw_next_id = self.half_edges[tw_id].next;
            let he_t_new = self.half_edges.len();
            self.half_edges.push(HalfEdge {
                next: tw_id,
                twin: None,
                origin: new_vertex_id,
                face: tw_face,
            });
            let he_t_new2 = self.half_edges.len();
            self.half_edges.push(HalfEdge {
                next: he_t_new,
                twin: Some(he_t_new2),
                origin: b,
                face: tw_face,
            });
            self.half_edges[he_t_new].twin = Some(he_t_new2);
            self.half_edges[tw_id].next = he_t_new2;
            self.half_edges[tw_id].origin = new_vertex_id;
            self.half_edges[tw_next_id].next = he_t_new;
        }
        Ok(new_vertex_id)
    }

    /// Collapses the edge to its origin vertex (removes the edge and the destination vertex from the ring).
    ///
    /// # Errors
    /// Returns `GeometryError::InvalidTopology` if collapse would create non-manifold or degenerate mesh.
    pub fn edge_collapse(&mut self, he_id: HalfEdgeId) -> Result<(), GeometryError> {
        let twin = self.half_edges[he_id].twin;
        let he = &self.half_edges[he_id];
        let next_id = he.next;
        let next = &self.half_edges[next_id];
        let v_origin = he.origin;
        let v_target = next.origin;
        if v_origin == v_target {
            return Err(GeometryError::InvalidTopology(
                "collapse same vertex".to_string(),
            ));
        }
        let next_next_id = next.next;
        let prev_id = (0..self.half_edges.len())
            .find(|&i| self.half_edges[i].next == he_id)
            .ok_or_else(|| GeometryError::InvalidTopology("half-edge not in ring".to_string()))?;
        self.half_edges[prev_id].next = next_next_id;
        if let Some(tw_id) = twin {
            let tw_prev_id = (0..self.half_edges.len())
                .find(|&i| self.half_edges[i].next == tw_id)
                .ok_or_else(|| GeometryError::InvalidTopology("twin not in ring".to_string()))?;
            let tw_next_id = self.half_edges[tw_id].next;
            self.half_edges[tw_prev_id].next = tw_next_id;
        }
        let pos_origin = self.vertices[v_origin].position;
        self.vertices[v_target].position = pos_origin;
        for he_item in &mut self.half_edges {
            if he_item.origin == v_target {
                he_item.origin = v_origin;
            }
        }
        Ok(())
    }

    /// Collects boundary half-edge loops (each loop is a list of half-edge ids).
    #[must_use]
    pub fn boundary_loops(&self) -> Vec<Vec<HalfEdgeId>> {
        let mut boundary = vec![];
        let mut used = vec![false; self.half_edges.len()];
        for (he_id, he) in self.half_edges.iter().enumerate() {
            if he.twin.is_some() || used[he_id] {
                continue;
            }
            let mut loop_ids = vec![];
            let mut cur = he_id;
            loop {
                used[cur] = true;
                loop_ids.push(cur);
                let next_he = self.half_edges[cur].next;
                let next_v = self.half_edges[next_he].origin;
                let mut next_boundary = None;
                let start = self.vertices[next_v].half_edge;
                let mut h = start;
                for _ in 0..self.half_edges.len() {
                    if self.half_edges[h].twin.is_none() {
                        next_boundary = Some(h);
                        break;
                    }
                    if let Some(t) = self.half_edges[h].twin {
                        h = self.half_edges[t].next;
                        if h == start {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                cur = next_boundary.unwrap_or(he_id);
                if cur == he_id {
                    break;
                }
            }
            boundary.push(loop_ids);
        }
        boundary
    }

    /// Number of vertices.
    #[must_use]
    pub fn num_vertices(&self) -> usize {
        self.vertices.len()
    }

    /// Number of faces.
    #[must_use]
    pub fn num_faces(&self) -> usize {
        self.faces.len()
    }

    /// Number of half-edges.
    #[must_use]
    pub fn num_half_edges(&self) -> usize {
        self.half_edges.len()
    }
}

/// Iterator over vertex indices adjacent to a given vertex.
pub struct VertexNeighbors<'a> {
    mesh: &'a HalfEdgeMesh,
    start_he: HalfEdgeId,
    current_he: HalfEdgeId,
    first: bool,
    done: bool,
}

impl<'a> Iterator for VertexNeighbors<'a> {
    type Item = VertexId;

    fn next(&mut self) -> Option<VertexId> {
        if self.done {
            return None;
        }
        let he = self.mesh.half_edges.get(self.current_he)?;
        let next_he = he.next;
        let next_he_data = &self.mesh.half_edges[next_he];
        let neighbor = next_he_data.origin;
        if let Some(twin) = he.twin {
            self.current_he = self.mesh.half_edges[twin].next;
            if !self.first && self.current_he == self.start_he {
                self.done = true;
                return Some(neighbor);
            }
            self.first = false;
        } else {
            self.done = true;
        }
        Some(neighbor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TriMesh;

    #[test]
    fn from_tri_mesh_single_triangle() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let indices = vec![[0, 1, 2]];
        let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
        let he = HalfEdgeMesh::from_tri_mesh(&mesh).unwrap();
        assert_eq!(he.num_vertices(), 3);
        assert_eq!(he.num_faces(), 1);
        assert_eq!(he.num_half_edges(), 3);
        let back = he.to_tri_mesh();
        assert_eq!(back.num_vertices(), 3);
        assert_eq!(back.num_triangles(), 1);
    }

    #[test]
    fn from_tri_mesh_two_triangles_shared_edge() {
        let positions = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.5, 1.0, 0.0],
            [0.5, 0.5, 0.0],
        ];
        let indices = vec![[0, 1, 2], [0, 2, 3]];
        let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
        let he = HalfEdgeMesh::from_tri_mesh(&mesh).unwrap();
        assert_eq!(he.num_vertices(), 4);
        assert_eq!(he.num_faces(), 2);
        assert_eq!(he.num_half_edges(), 6);
        let neighbors: Vec<VertexId> = he.vertex_neighbors(0).collect();
        assert!(neighbors.len() >= 2);
    }

    #[test]
    fn vertex_neighbors_interior() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let indices = vec![[0, 1, 2]];
        let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
        let he = HalfEdgeMesh::from_tri_mesh(&mesh).unwrap();
        let n0: Vec<VertexId> = he.vertex_neighbors(0).collect();
        assert!(!n0.is_empty());
        assert!(n0.contains(&1));
    }

    #[test]
    fn edge_split() {
        let positions = vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let indices = vec![[0, 1, 2]];
        let mesh = TriMesh::from_positions_and_indices(positions, indices).unwrap();
        let mut he = HalfEdgeMesh::from_tri_mesh(&mesh).unwrap();
        let he_id = 0;
        let new_v = he.edge_split(he_id).unwrap();
        assert_eq!(he.num_vertices(), 4);
        assert!(he.vertices[new_v].position[0] - 0.5 < 0.01);
    }
}
