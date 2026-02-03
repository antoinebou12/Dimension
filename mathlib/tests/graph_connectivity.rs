//! Integration tests for disjoint set and connected components.

use mathlib::{DisjointSet, Graph, connected_components};

#[test]
fn disjoint_set_singletons() {
    let mut ds = DisjointSet::new(5);
    for i in 0..5 {
        assert_eq!(ds.find(i), i);
    }
    assert!(!ds.connected(0, 1));
}

#[test]
fn disjoint_set_union_find() {
    let mut ds = DisjointSet::new(5);
    ds.union(0, 1);
    ds.union(2, 3);
    assert!(ds.connected(0, 1));
    assert!(ds.connected(2, 3));
    assert!(!ds.connected(0, 2));
    ds.union(1, 2);
    assert!(ds.connected(0, 3));
}

#[test]
fn connected_components_single() {
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    let comps = connected_components(&g);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].len(), 3);
}

#[test]
fn connected_components_disjoint() {
    let mut g = Graph::new(5);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(3, 4, 1.0);
    let comps = connected_components(&g);
    assert_eq!(comps.len(), 2);
    let sizes: Vec<usize> = comps.iter().map(|c| c.len()).collect();
    assert!(sizes.contains(&3));
    assert!(sizes.contains(&2));
}

#[test]
fn connected_components_two_triangles_shared_vertex() {
    // 0-1-2 and 0-3-4: two triangles sharing vertex 0 -> one component
    let mut g = Graph::new(5);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(2, 0, 1.0);
    g.add_undirected_edge(0, 3, 1.0);
    g.add_undirected_edge(3, 4, 1.0);
    g.add_undirected_edge(4, 0, 1.0);
    let comps = connected_components(&g);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].len(), 5);
}
