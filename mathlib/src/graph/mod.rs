//! Graph pathfinding and structure: Dijkstra, A*, D* Lite, disjoint set, articulation points,
//! vertex coloring (greedy, `DSatur`, bipartite), and tree traversals (BFS, DFS).
//!
//! Uses sequential execution by default; optional parallel (Rayon) and SIMD (wide) backends
//! when the `parallel` and `simd` features are enabled (same dispatch pattern as `cpu` and `distance`).

mod types;

pub mod articulation;
pub mod astar;
pub mod coloring;
pub mod dijkstra;
pub mod disjoint;
pub mod disjoint_set;
pub mod dstar;
pub mod matrix;
pub mod tree;

pub use articulation::{articulation_points, bridges};
pub use astar::{AStarResult, astar};
pub use coloring::{dsatur_coloring, greedy_vertex_coloring, is_bipartite};
pub use dijkstra::{DijkstraResult, dijkstra};
pub use disjoint::{UnionFind, connected_components_undirected};
pub use disjoint_set::{DisjointSet, connected_components};
pub use dstar::{DStarLite, DStarLiteResult, dstar_lite};
pub use matrix::{
    adjacency_ccs, adjacency_crs, adjacency_triplets, laplacian_2d_grid_crs,
    laplacian_2d_grid_triplets, laplacian_crs, laplacian_triplets, tree_adjacency_crs,
    tree_adjacency_triplets,
};
pub use tree::{
    BfsResult, Node, Tree, bfs, dfs_postorder, dfs_postorder_forest, dfs_preorder,
    dfs_preorder_forest, path_to_traversal_mapping,
};
pub use types::{Edge, Graph, NodeId, Weight, reverse_graph};
