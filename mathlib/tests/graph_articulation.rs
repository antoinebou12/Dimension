//! Integration tests for articulation points and bridges.

use mathlib::{Graph, articulation_points, bridges};

#[test]
fn articulation_empty() {
    let g = Graph::new(0);
    let ap = articulation_points(&g);
    assert!(ap.is_empty());
}

#[test]
fn articulation_single_node() {
    let g = Graph::new(1);
    let ap = articulation_points(&g);
    assert!(ap.is_empty());
}

#[test]
fn articulation_bridge() {
    // 0 -- 1 -- 2: vertex 1 is articulation (bridge)
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    let ap = articulation_points(&g);
    assert_eq!(ap.len(), 1);
    assert_eq!(ap[0], 1);
}

#[test]
fn articulation_cycle() {
    // Triangle 0-1-2-0: no articulation points
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(2, 0, 1.0);
    let ap = articulation_points(&g);
    assert!(ap.is_empty());
}

#[test]
fn articulation_tree() {
    // Star: 0 -- 1, 0 -- 2, 0 -- 3. Root 0 is articulation (more than one child).
    let mut g = Graph::new(4);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(0, 2, 1.0);
    g.add_undirected_edge(0, 3, 1.0);
    let ap = articulation_points(&g);
    assert_eq!(ap.len(), 1);
    assert_eq!(ap[0], 0);
}

#[test]
fn articulation_path() {
    // Path 0-1-2-3: 1 and 2 are articulation points
    let mut g = Graph::new(4);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(2, 3, 1.0);
    let ap = articulation_points(&g);
    assert_eq!(ap.len(), 2);
    assert!(ap.contains(&1));
    assert!(ap.contains(&2));
}

#[test]
fn bridges_path() {
    // Path 0-1-2-3: every edge is a bridge
    let mut g = Graph::new(4);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(2, 3, 1.0);
    let br = bridges(&g);
    assert_eq!(br.len(), 3);
}

#[test]
fn bridges_bridge_edge() {
    // 0-1-2 and 0-2: edge (1,2) is a bridge (or (0,1) depending on DFS). Two triangles 0-1-2 and 0-2-3 would give one bridge.
    // Simpler: path 0-1-2, so (0,1) and (1,2) are bridges.
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    let ap = articulation_points(&g);
    let br = bridges(&g);
    assert_eq!(ap.len(), 1);
    assert_eq!(ap[0], 1);
    assert_eq!(br.len(), 2);
}

#[test]
fn bridges_cycle_none() {
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(2, 0, 1.0);
    let br = bridges(&g);
    assert!(br.is_empty());
}
