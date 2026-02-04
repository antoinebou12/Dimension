//! Depth-first search (DFS) for graph traversal.
//!
//! Treats the graph as undirected: traverses both outgoing and incoming edges.
//! Time complexity: O(V + E). Uses a `Vec<bool>` visited set for dense node IDs in `0..n`.
//!
//! [`dfs_preorder_forest`] and [`dfs_postorder_forest`] run DFS per connected component in
//! parallel when the `parallel` feature is enabled (not on wasm32).

use crate::graph::types::Graph;

/// Runs DFS from `source` in preorder (visit node before its descendants).
#[must_use]
pub fn dfs_preorder(graph: &Graph, source: usize) -> Vec<usize> {
    let n = graph.num_nodes();
    if source >= n {
        return Vec::new();
    }
    let mut order = Vec::with_capacity(n);
    let mut visited = vec![false; n];
    let mut stack = vec![source];
    while let Some(u) = stack.pop() {
        if visited[u] {
            continue;
        }
        visited[u] = true;
        order.push(u);
        for &(v, _) in graph.neighbors(u) {
            if !visited[v] {
                stack.push(v);
            }
        }
        for &(v, _) in graph.in_neighbors(u) {
            if !visited[v] {
                stack.push(v);
            }
        }
    }
    order
}

/// Runs DFS from `source` in postorder (visit node after its descendants).
#[must_use]
pub fn dfs_postorder(graph: &Graph, source: usize) -> Vec<usize> {
    let n = graph.num_nodes();
    if source >= n {
        return Vec::new();
    }
    let mut order = Vec::with_capacity(n);
    let mut visited = vec![false; n];
    let mut stack = vec![(source, false)];
    while let Some((u, children_done)) = stack.pop() {
        if children_done {
            order.push(u);
            continue;
        }
        if visited[u] {
            continue;
        }
        visited[u] = true;
        stack.push((u, true));
        for &(v, _) in graph.neighbors(u) {
            if !visited[v] {
                stack.push((v, false));
            }
        }
        for &(v, _) in graph.in_neighbors(u) {
            if !visited[v] {
                stack.push((v, false));
            }
        }
    }
    order
}

/// Runs DFS preorder on each connected component, concatenating results.
/// With the `parallel` feature (not on wasm32), processes components in parallel.
#[must_use]
pub fn dfs_preorder_forest(graph: &Graph) -> Vec<usize> {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    return dfs_preorder_forest_parallel(graph);
    #[allow(unreachable_code)]
    dfs_preorder_forest_sequential(graph)
}

/// Runs DFS postorder on each connected component, concatenating results.
/// With the `parallel` feature (not on wasm32), processes components in parallel.
#[must_use]
pub fn dfs_postorder_forest(graph: &Graph) -> Vec<usize> {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    return dfs_postorder_forest_parallel(graph);
    #[allow(unreachable_code)]
    dfs_postorder_forest_sequential(graph)
}

fn dfs_preorder_forest_sequential(graph: &Graph) -> Vec<usize> {
    let components = crate::graph::connected_components(graph);
    let mut order = Vec::with_capacity(graph.num_nodes());
    for comp in components {
        if let Some(&root) = comp.first() {
            order.extend(dfs_preorder(graph, root));
        }
    }
    order
}

fn dfs_postorder_forest_sequential(graph: &Graph) -> Vec<usize> {
    let components = crate::graph::connected_components(graph);
    let mut order = Vec::with_capacity(graph.num_nodes());
    for comp in components {
        if let Some(&root) = comp.first() {
            order.extend(dfs_postorder(graph, root));
        }
    }
    order
}

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
fn dfs_preorder_forest_parallel(graph: &Graph) -> Vec<usize> {
    use par_iter::prelude::*;
    let components = crate::graph::connected_components(graph);
    components
        .par_iter()
        .filter_map(|comp| comp.first().copied())
        .flat_map(|root| dfs_preorder(graph, root))
        .collect()
}

#[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
fn dfs_postorder_forest_parallel(graph: &Graph) -> Vec<usize> {
    use par_iter::prelude::*;
    let components = crate::graph::connected_components(graph);
    components
        .par_iter()
        .filter_map(|comp| comp.first().copied())
        .flat_map(|root| dfs_postorder(graph, root))
        .collect()
}
