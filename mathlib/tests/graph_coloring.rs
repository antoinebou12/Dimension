//! Integration tests for graph coloring.

use mathlib::{Graph, greedy_vertex_coloring, is_bipartite};

#[test]
fn is_bipartite_empty() {
    let g = Graph::new(0);
    let colors = is_bipartite(&g);
    assert!(colors.is_some());
    assert!(colors.unwrap().is_empty());
}

#[test]
fn is_bipartite_single_node() {
    let g = Graph::new(1);
    let colors = is_bipartite(&g);
    assert!(colors.is_some());
    assert_eq!(colors.unwrap()[0], 0);
}

#[test]
fn is_bipartite_path_two() {
    let mut g = Graph::new(2);
    g.add_undirected_edge(0, 1, 1.0);
    let colors = is_bipartite(&g);
    assert!(colors.is_some());
    let c = colors.unwrap();
    assert_ne!(c[0], c[1]);
}

#[test]
fn is_bipartite_path_three() {
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    let colors = is_bipartite(&g);
    assert!(colors.is_some());
    let c = colors.unwrap();
    assert_ne!(c[0], c[1]);
    assert_ne!(c[1], c[2]);
}

#[test]
fn is_bipartite_odd_cycle() {
    let mut g = Graph::new(3);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(2, 0, 1.0);
    let colors = is_bipartite(&g);
    assert!(colors.is_none());
}

#[test]
fn is_bipartite_even_cycle() {
    let mut g = Graph::new(4);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(2, 3, 1.0);
    g.add_undirected_edge(3, 0, 1.0);
    let colors = is_bipartite(&g);
    assert!(colors.is_some());
    let c = colors.unwrap();
    assert_ne!(c[0], c[1]);
    assert_ne!(c[1], c[2]);
    assert_ne!(c[2], c[3]);
    assert_ne!(c[3], c[0]);
}

#[test]
fn greedy_vertex_coloring_empty() {
    let g = Graph::new(0);
    let colors = greedy_vertex_coloring(&g);
    assert!(colors.is_empty());
}

#[test]
fn greedy_vertex_coloring_no_edges() {
    let g = Graph::new(4);
    let colors = greedy_vertex_coloring(&g);
    assert_eq!(colors.len(), 4);
    assert!(colors.iter().all(|&c| c == 0));
}

#[test]
fn greedy_vertex_coloring_valid() {
    let mut g = Graph::new(4);
    g.add_undirected_edge(0, 1, 1.0);
    g.add_undirected_edge(1, 2, 1.0);
    g.add_undirected_edge(2, 3, 1.0);
    g.add_undirected_edge(0, 2, 1.0);
    let colors = greedy_vertex_coloring(&g);
    assert_eq!(colors.len(), 4);
    for u in 0..4 {
        for &(v, _) in g.neighbors(u) {
            assert_ne!(
                colors[u], colors[v],
                "adjacent nodes must have different colors"
            );
        }
    }
}
