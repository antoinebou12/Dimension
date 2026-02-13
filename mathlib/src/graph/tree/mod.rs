//! Tree and graph traversal: BFS and DFS.
//!
//! All traversals treat the graph as undirected (consider both outgoing and incoming edges).
//! Useful for exploring connected components, computing visit orders, and level/depth info.
//!
//! [`Tree`] and [`Node`] provide an explicit tree structure with parent/children, buildable
//! from a graph via [`Tree::from_bfs_spanning_tree`].

mod bfs;
mod dfs;
mod structure;

pub use bfs::{BfsResult, bfs};
pub use dfs::{dfs_postorder, dfs_postorder_forest, dfs_preorder, dfs_preorder_forest};
pub use structure::{Node, Tree, path_to_traversal_mapping};
