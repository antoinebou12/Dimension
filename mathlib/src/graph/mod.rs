//! Graph pathfinding and structure: Dijkstra, A*, D* Lite, disjoint set, articulation points, coloring.
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

pub use articulation::{articulation_points, bridges};
pub use astar::{AStarResult, astar};
pub use coloring::{greedy_vertex_coloring, is_bipartite};
pub use dijkstra::{DijkstraResult, dijkstra};
pub use disjoint::{UnionFind, connected_components_undirected};
pub use disjoint_set::{DisjointSet, connected_components};
pub use dstar::{DStarLite, DStarLiteResult, dstar_lite};
pub use types::{Edge, Graph, NodeId, Weight, reverse_graph};
