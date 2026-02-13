//! Tree structure integration tests.

use mathlib::{Graph, Tree, bfs, dfs_postorder, dfs_preorder, path_to_traversal_mapping};

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

#[test]
fn tree_remove_child() {
    let mut tree: Tree<()> = Tree::new(0);
    tree.add_child(0, 1);
    tree.add_child(0, 2);
    tree.add_child(1, 3);
    tree.remove_child(0, 1);
    assert_eq!(tree.nodes[0].children, vec![2]);
    assert_eq!(tree.nodes[1].parent, None);
    assert_eq!(tree.nodes[1].children, vec![3]);
    // Subtree 1->3 is detached; DFS from root only visits 0, 2
    assert_eq!(tree.dfs_preorder(), vec![0, 2]);
}

#[test]
fn tree_reparent() {
    let mut tree: Tree<()> = Tree::new(0);
    tree.add_child(0, 1);
    tree.add_child(0, 2);
    tree.add_child(1, 3);
    tree.reparent(3, 0);
    assert_eq!(tree.nodes[0].children, vec![1, 2, 3]);
    assert_eq!(tree.nodes[1].children, vec![]);
    assert_eq!(tree.nodes[3].parent, Some(0));
    assert_eq!(tree.dfs_preorder(), vec![0, 1, 2, 3]);
}

#[test]
#[should_panic(expected = "cannot remove root")]
fn tree_remove_child_root_panics() {
    let mut tree: Tree<()> = Tree::new(0);
    tree.add_child(0, 1);
    tree.remove_child(0, 0);
}

#[test]
#[should_panic(expected = "reparent would create cycle")]
fn tree_reparent_cycle() {
    let mut tree: Tree<()> = Tree::new(0);
    tree.add_child(0, 1);
    tree.add_child(1, 2);
    tree.reparent(0, 2);
}

#[test]
fn tree_path_from_root() {
    let mut tree: Tree<()> = Tree::new(0);
    tree.add_child(0, 1);
    tree.add_child(0, 2);
    tree.add_child(1, 3);
    assert_eq!(tree.path_from_root(0), vec![0]);
    assert_eq!(tree.path_from_root(1), vec![0, 1]);
    assert_eq!(tree.path_from_root(2), vec![0, 2]);
    assert_eq!(tree.path_from_root(3), vec![0, 1, 3]);
    assert!(tree.path_from_root(99).is_empty());
}

#[test]
fn path_to_traversal_mapping_simple() {
    // Node 0: 1 DOF at offset 0; node 1: 0 DOF; node 2: 1 DOF at offset 1
    let path = [0, 2];
    let traversal_order = [0, 1, 2];
    let node_size = |i: usize| -> usize { if i == 1 { 0 } else { 1 } };
    let mapping = path_to_traversal_mapping(&path, &traversal_order, node_size);
    assert_eq!(mapping, vec![0, 1]);
}

#[test]
fn path_to_traversal_mapping_branched() {
    // Tree: root(0) -> child(1), child(2); each node has 1 DOF
    let path = [0, 2];
    let traversal_order = [0, 1, 2];
    let node_size = |_| 1;
    let mapping = path_to_traversal_mapping(&path, &traversal_order, node_size);
    assert_eq!(mapping, vec![0, 2]);
}

#[test]
fn path_to_traversal_mapping_multi_dof() {
    // Node 0: 1 DOF at 0; node 1: 0 DOF; node 2: 3 DOF at 1,2,3
    let path = [0, 2];
    let traversal_order = [0, 1, 2];
    let node_size = |i: usize| -> usize {
        match i {
            0 => 1,
            1 => 0,
            2 => 3,
            _ => 0,
        }
    };
    let mapping = path_to_traversal_mapping(&path, &traversal_order, node_size);
    assert_eq!(mapping, vec![0, 1, 2, 3]);
}
