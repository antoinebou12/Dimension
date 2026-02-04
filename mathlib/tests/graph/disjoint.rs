//! Tests for UnionFind and connected_components_undirected.

use mathlib::{Graph, UnionFind, connected_components_undirected};

#[test]
fn union_find_singletons() {
    let mut uf = UnionFind::new(5);
    for i in 0..5 {
        assert_eq!(uf.find(i), i);
    }
    assert!(!uf.connected(0, 1));
}

#[test]
fn union_find_union_find() {
    let mut uf = UnionFind::new(5);
    uf.union(0, 1);
    uf.union(2, 3);
    assert!(uf.connected(0, 1));
    assert!(uf.connected(2, 3));
    assert!(!uf.connected(0, 2));
    uf.union(1, 2);
    assert!(uf.connected(0, 3));
}

#[test]
fn connected_components_undirected_single() {
    let mut g = Graph::new(3);
    g.add_edge_undirected(0, 1, 1.0);
    g.add_edge_undirected(1, 2, 1.0);
    let comps = connected_components_undirected(&g);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].len(), 3);
}

#[test]
fn connected_components_undirected_two_triangles_shared_vertex() {
    let mut g = Graph::new(5);
    g.add_edge_undirected(0, 1, 1.0);
    g.add_edge_undirected(1, 2, 1.0);
    g.add_edge_undirected(2, 0, 1.0);
    g.add_edge_undirected(1, 3, 1.0);
    g.add_edge_undirected(3, 4, 1.0);
    g.add_edge_undirected(4, 1, 1.0);
    let comps = connected_components_undirected(&g);
    assert_eq!(comps.len(), 1);
    assert_eq!(comps[0].len(), 5);
}
