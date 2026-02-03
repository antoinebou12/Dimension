//! Example: graph construction, adjacency, pathfinding, connectivity, articulation, coloring.

use mathlib::{
    AStarResult, DijkstraResult, Graph, articulation_points, astar, bridges, connected_components,
    dijkstra, greedy_vertex_coloring, is_bipartite, reverse_graph,
};

fn main() {
    // ---- Directed weighted graph (pathfinding + reverse) ----
    let mut g = Graph::new(5);
    g.add_edge(0, 1, 1.0);
    g.add_edge(0, 2, 4.0);
    g.add_edge(1, 2, 2.0);
    g.add_edge(1, 3, 6.0);
    g.add_edge(2, 3, 1.0);
    g.add_edge(2, 4, 3.0);
    g.add_edge(3, 4, 1.0);

    println!(
        "Directed graph: {} nodes, {} edges",
        g.num_nodes(),
        g.num_edges()
    );
    println!(
        "  Node 0: out_degree={}, in_degree={}",
        g.out_degree(0),
        g.in_degree(0)
    );
    println!("  Node 0 neighbors: {:?}", g.neighbors(0));
    println!("  Node 2 in_neighbors: {:?}", g.in_neighbors(2));
    println!(
        "  is_adjacent(0, 1) = {}, is_adjacent(1, 0) = {}",
        g.is_adjacent(0, 1),
        g.is_adjacent(1, 0)
    );

    let edges: Vec<_> = g.edges().collect();
    println!("  Edges (u -> v, weight):");
    for e in &edges {
        println!("    {} -> {} (w={})", e.u, e.v, e.weight);
    }

    let rev = reverse_graph(&g);
    println!(
        "\nReverse graph: edge 1->2 becomes 2->1: is_adjacent(2, 1) = {}",
        rev.is_adjacent(2, 1)
    );

    let dres: DijkstraResult = dijkstra(&g, 0);
    println!("\nDijkstra from 0:");
    for u in 0..g.num_nodes() {
        println!("  dist[{}] = {}", u, dres.dist[u]);
    }
    let path_to_4 = path_from_prev(&dres.prev, 0, 4);
    println!("  Path 0 -> 4: {:?}", path_to_4);

    let goal = 4;
    let ares: AStarResult = astar(&g, 0, goal, |_, _| 0.0);
    println!(
        "\nA* from 0 to {} (zero heuristic): path = {:?}, dist = {}",
        goal, ares.path, ares.dist
    );

    // ---- Undirected graph (connectivity, articulation, bridges, coloring) ----
    let mut undir = Graph::new(4);
    undir.add_edge_undirected(0, 1, 1.0);
    undir.add_edge_undirected(1, 2, 1.0);
    undir.add_edge_undirected(2, 3, 1.0);

    println!(
        "\nUndirected path 0-1-2-3: {} nodes, {} edges",
        undir.num_nodes(),
        undir.num_edges()
    );

    let components = connected_components(&undir);
    println!("  Connected components: {:?}", components);

    let ap = articulation_points(&undir);
    println!("  Articulation points: {:?}", ap);

    let br = bridges(&undir);
    println!(
        "  Bridges: {:?}",
        br.iter().map(|e| (e.u, e.v)).collect::<Vec<_>>()
    );

    match is_bipartite(&undir) {
        Some(colors) => println!("  is_bipartite: Some({:?})", colors),
        None => println!("  is_bipartite: None"),
    }

    let colors = greedy_vertex_coloring(&undir);
    println!("  greedy_vertex_coloring: {:?}", colors);
}

fn path_from_prev(prev: &[Option<usize>], start: usize, end: usize) -> Vec<usize> {
    let mut path = vec![end];
    let mut u = end;
    while let Some(p) = prev[u] {
        path.push(p);
        if p == start {
            break;
        }
        u = p;
    }
    path.reverse();
    path
}
