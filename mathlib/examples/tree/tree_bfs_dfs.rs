//! Example: BFS and DFS on 4-node, Path 5, Star 5, Grid 6.
//!
//! Demonstrates both Graph-based and Tree-based traversal APIs.
//! Run with: `cargo run --example tree_bfs_dfs`

use mathlib::{Graph, Tree, bfs, dfs_postorder, dfs_preorder};

fn build_undirected(n: usize, edges: &[(usize, usize)]) -> Graph {
    let mut g = Graph::new(n);
    for &(u, v) in edges {
        g.add_edge_undirected(u, v, 1.0);
    }
    g
}

fn main() {
    let examples: &[(&str, usize, &[(usize, usize)], usize)] = &[
        ("4-node", 4, &[(0, 1), (0, 2), (1, 3)], 0),
        ("Path 5", 5, &[(0, 1), (1, 2), (2, 3), (3, 4)], 0),
        ("Star 5", 5, &[(0, 1), (0, 2), (0, 3), (0, 4)], 0),
        (
            "Grid 6",
            6,
            &[(0, 1), (0, 2), (1, 3), (2, 3), (2, 4), (3, 5), (4, 5)],
            0,
        ),
    ];

    for (title, n, edges, source) in examples {
        let g = build_undirected(*n, edges);
        println!("=== {} ===\nGraph: {} nodes, source {}\n", title, n, source);

        // Graph-based API
        let bfs_res = bfs(&g, *source);
        let pre = dfs_preorder(&g, *source);
        let post = dfs_postorder(&g, *source);
        println!("BFS order: {:?}", bfs_res.order);
        println!(
            "BFS depth: {:?}",
            bfs_res
                .depth
                .iter()
                .map(|d| if *d == usize::MAX {
                    "∞".to_string()
                } else {
                    d.to_string()
                })
                .collect::<Vec<_>>()
        );
        println!("DFS preorder:  {:?}", pre);
        println!("DFS postorder: {:?}\n", post);

        // Tree-based API (BFS spanning tree from same source)
        let tree: Tree<()> = Tree::from_bfs_spanning_tree(&g, *source);
        println!("Tree (from BFS spanning tree):");
        println!("  bfs_order:    {:?}", tree.bfs_order());
        println!("  dfs_preorder: {:?}", tree.dfs_preorder());
        println!("  dfs_postorder: {:?}\n", tree.dfs_postorder());
    }
}
