//! Integration tests for wasm graph bindings (WasmGraph, Dijkstra, A*, D* Lite, coloring, BFS/DFS).
//! Run with: cargo test --features wasm wasm_graph

#![cfg(feature = "wasm")]

use mathlib::wasm::{
    WasmAstarResult, WasmBfsResult, WasmDStarLiteResult, WasmDijkstraResult, WasmGraph,
};

#[test]
fn wasm_graph_from_edges() {
    // 3 nodes: 0->1 (2), 0->2 (5), 1->2 (1)
    let edges = [0.0, 1.0, 2.0, 0.0, 2.0, 5.0, 1.0, 2.0, 1.0];
    let g = WasmGraph::from_edges(3, &edges).unwrap();
    assert_eq!(g.num_nodes(), 3);
    assert_eq!(g.num_edges(), 3);
}

#[test]
fn wasm_graph_dijkstra() {
    let edges = [
        0.0, 1.0, 1.0, 0.0, 2.0, 4.0, 1.0, 2.0, 2.0, 1.0, 3.0, 6.0, 2.0, 3.0, 1.0,
    ];
    let g = WasmGraph::from_edges(4, &edges).unwrap();
    let res: WasmDijkstraResult = g.run_dijkstra(0).unwrap();
    let dist = res.get_distances();
    assert!((dist[0] - 0.0).abs() < 1e-10);
    assert!((dist[1] - 1.0).abs() < 1e-10);
    assert!((dist[2] - 3.0).abs() < 1e-10);
    assert!((dist[3] - 4.0).abs() < 1e-10);
    let path = res.path_to(3);
    assert_eq!(path, [0, 1, 2, 3]);
    assert!((res.distance_to(3) - 4.0).abs() < 1e-10);
}

#[test]
fn wasm_graph_astar_zero_heuristic() {
    let edges = [0.0, 1.0, 1.0, 0.0, 2.0, 4.0, 1.0, 2.0, 2.0, 2.0, 3.0, 1.0];
    let g = WasmGraph::from_edges(4, &edges).unwrap();
    let res: WasmAstarResult = g.run_astar(0, 3).unwrap();
    assert!((res.get_dist() - 4.0).abs() < 1e-10);
    assert_eq!(res.get_path(), [0, 1, 2, 3]);
}

#[test]
fn wasm_graph_dstar_lite() {
    let edges = [0.0, 1.0, 1.0, 0.0, 2.0, 4.0, 1.0, 2.0, 2.0, 2.0, 3.0, 1.0];
    let mut g = WasmGraph::from_edges(4, &edges).unwrap();
    let res: WasmDStarLiteResult = g.dstar_lite(0, 3).unwrap();
    assert!((res.get_dist() - 4.0).abs() < 1e-10);
    assert_eq!(res.get_path(), [0, 1, 2, 3]);
}

#[test]
fn wasm_graph_greedy_coloring() {
    let mut g = WasmGraph::new(4);
    g.add_edge_undirected(0, 1, 1.0).unwrap();
    g.add_edge_undirected(1, 2, 1.0).unwrap();
    g.add_edge_undirected(2, 3, 1.0).unwrap();
    g.add_edge_undirected(3, 0, 1.0).unwrap();
    let colors = g.greedy_vertex_coloring();
    assert_eq!(colors.len(), 4);
    assert_ne!(colors[0], colors[1]);
    assert_ne!(colors[1], colors[2]);
    assert_ne!(colors[2], colors[3]);
    assert_ne!(colors[3], colors[0]);
}

#[test]
fn wasm_graph_dsatur_coloring() {
    let mut g = WasmGraph::new(3);
    g.add_edge_undirected(0, 1, 1.0).unwrap();
    g.add_edge_undirected(1, 2, 1.0).unwrap();
    g.add_edge_undirected(0, 2, 1.0).unwrap();
    let colors = g.dsatur_coloring();
    assert_eq!(colors.len(), 3);
    assert_ne!(colors[0], colors[1]);
    assert_ne!(colors[1], colors[2]);
    assert_ne!(colors[0], colors[2]);
}

#[test]
fn wasm_graph_is_bipartite() {
    let mut triangle = WasmGraph::new(3);
    triangle.add_edge_undirected(0, 1, 1.0).unwrap();
    triangle.add_edge_undirected(1, 2, 1.0).unwrap();
    triangle.add_edge_undirected(2, 0, 1.0).unwrap();
    let odd_cycle = triangle.is_bipartite();
    assert!(odd_cycle.is_none(), "triangle has odd cycle");
    let mut line = WasmGraph::new(3);
    line.add_edge_undirected(0, 1, 1.0).unwrap();
    line.add_edge_undirected(1, 2, 1.0).unwrap();
    let line_coloring = line.is_bipartite();
    assert!(line_coloring.is_some());
    assert_eq!(line_coloring.as_ref().unwrap().len(), 3);
}

#[test]
fn wasm_graph_bfs() {
    let edges = [0.0, 1.0, 1.0, 0.0, 2.0, 1.0, 1.0, 2.0, 1.0];
    let g = WasmGraph::from_edges(3, &edges).unwrap();
    let res: WasmBfsResult = g.run_bfs(0).unwrap();
    let order = res.get_order();
    assert_eq!(order.len(), 3);
    assert_eq!(order[0], 0);
    assert!(order.contains(&1) && order.contains(&2));
}

#[test]
fn wasm_graph_dfs_preorder() {
    let mut g = WasmGraph::new(4);
    g.add_edge_undirected(0, 1, 1.0).unwrap();
    g.add_edge_undirected(0, 2, 1.0).unwrap();
    g.add_edge_undirected(1, 3, 1.0).unwrap();
    let pre = g.dfs_preorder(0).unwrap();
    assert_eq!(pre.len(), 4);
    assert_eq!(pre[0], 0);
}

#[test]
fn wasm_graph_dfs_postorder() {
    let mut g = WasmGraph::new(3);
    g.add_edge_undirected(0, 1, 1.0).unwrap();
    g.add_edge_undirected(1, 2, 1.0).unwrap();
    let post = g.dfs_postorder(0).unwrap();
    assert_eq!(post.len(), 3);
    assert_eq!(post[post.len() - 1], 0);
}

/// Error path: only run on wasm32; JsError is not fully supported on native.
#[test]
#[cfg(target_arch = "wasm32")]
fn wasm_graph_from_edges_invalid() {
    assert!(WasmGraph::from_edges(2, &[0.0, 1.0]).is_err());
    assert!(WasmGraph::from_edges(2, &[0.0, 5.0, 1.0]).is_err());
    assert!(WasmGraph::from_edges(2, &[0.0, 1.0, -1.0]).is_err());
}
