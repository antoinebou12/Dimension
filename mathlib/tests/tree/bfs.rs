//! BFS integration tests.

use mathlib::{Graph, bfs};

#[test]
fn bfs_empty() {
    let g = Graph::new(0);
    let res = bfs(&g, 0);
    assert!(res.order.is_empty());
    assert!(res.depth.is_empty());
}

#[test]
fn bfs_source_out_of_bounds() {
    let g = Graph::new(2);
    let res = bfs(&g, 5);
    assert!(res.order.is_empty());
    assert_eq!(res.depth.len(), 2);
}

#[test]
fn bfs_single_node() {
    let g = Graph::new(1);
    let res = bfs(&g, 0);
    assert_eq!(res.order, [0]);
    assert_eq!(res.depth, [0]);
}

#[test]
fn bfs_path() {
    let mut g = Graph::new(5);
    g.add_edge_undirected(0, 1, 1.0);
    g.add_edge_undirected(1, 2, 1.0);
    g.add_edge_undirected(2, 3, 1.0);
    g.add_edge_undirected(3, 4, 1.0);
    let res = bfs(&g, 0);
    assert_eq!(res.order.len(), 5);
    assert_eq!(res.order[0], 0);
    assert_eq!(res.depth[0], 0);
    assert_eq!(res.depth[1], 1);
    assert_eq!(res.depth[2], 2);
    assert_eq!(res.depth[3], 3);
    assert_eq!(res.depth[4], 4);
}

#[test]
fn bfs_grid() {
    let mut g = Graph::new(9);
    for i in 0..3 {
        for j in 0..3 {
            let u = i * 3 + j;
            if j + 1 < 3 {
                g.add_edge_undirected(u, i * 3 + (j + 1), 1.0);
            }
            if i + 1 < 3 {
                g.add_edge_undirected(u, (i + 1) * 3 + j, 1.0);
            }
        }
    }
    let res = bfs(&g, 0);
    assert_eq!(res.order.len(), 9);
    assert_eq!(res.depth[0], 0);
    assert_eq!(res.depth[4], 2);
    assert_eq!(res.depth[8], 4);
}

#[test]
fn bfs_disconnected() {
    let mut g = Graph::new(4);
    g.add_edge_undirected(0, 1, 1.0);
    g.add_edge_undirected(2, 3, 1.0);
    let res = bfs(&g, 0);
    assert_eq!(res.order.len(), 2);
    assert!(res.order.contains(&0));
    assert!(res.order.contains(&1));
    assert_eq!(res.depth[2], usize::MAX);
    assert_eq!(res.depth[3], usize::MAX);
}
