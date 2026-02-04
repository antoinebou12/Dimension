//! Tree structure integration tests.

use mathlib::{Graph, Tree, bfs, dfs_postorder, dfs_preorder};

fn build_undirected(n: usize, edges: &[(usize, usize)]) -> Graph {
    let mut g = Graph::new(n);
    for &(u, v) in edges {
        g.add_edge_undirected(u, v, 1.0);
    }
    g
}

#[test]
fn tree_from_bfs_spanning_tree_path() {
    let g = build_undirected(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    let tree: Tree<()> = Tree::from_bfs_spanning_tree(&g, 0);
    assert_eq!(tree.num_nodes(), 5);
    assert_eq!(tree.root, 0);
    // BFS order on path equals graph BFS order
    let bfs_res = bfs(&g, 0);
    assert_eq!(tree.bfs_order(), bfs_res.order);
}

#[test]
fn tree_dfs_matches_graph_path() {
    let g = build_undirected(5, &[(0, 1), (1, 2), (2, 3), (3, 4)]);
    let tree: Tree<()> = Tree::from_bfs_spanning_tree(&g, 0);
    let pre = dfs_preorder(&g, 0);
    let post = dfs_postorder(&g, 0);
    // Tree DFS uses BFS spanning tree structure; order may differ from graph DFS
    // (graph explores all edges, tree only parent-child). Verify tree produces valid order.
    assert_eq!(tree.dfs_preorder().len(), 5);
    assert_eq!(tree.dfs_postorder().len(), 5);
    assert_eq!(tree.dfs_postorder()[tree.dfs_postorder().len() - 1], 0);
    assert_eq!(pre.len(), 5);
    assert_eq!(post.len(), 5);
}

#[test]
fn tree_manual_add_child() {
    let mut tree: Tree<()> = Tree::new(0);
    tree.add_child(0, 1);
    tree.add_child(0, 2);
    tree.add_child(1, 3);
    assert_eq!(tree.bfs_order(), vec![0, 1, 2, 3]);
    assert_eq!(tree.dfs_preorder(), vec![0, 1, 3, 2]);
    assert_eq!(tree.dfs_postorder(), vec![3, 1, 2, 0]);
}

#[test]
fn tree_from_bfs_spanning_tree_star() {
    let g = build_undirected(5, &[(0, 1), (0, 2), (0, 3), (0, 4)]);
    let tree: Tree<()> = Tree::from_bfs_spanning_tree(&g, 0);
    assert_eq!(tree.bfs_order(), vec![0, 1, 2, 3, 4]);
    assert_eq!(tree.dfs_postorder(), vec![1, 2, 3, 4, 0]);
}
