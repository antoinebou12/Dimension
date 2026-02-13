//! Island decomposition and constraint graph coloring for parallel PBD.
//!
//! - **Body graph**: nodes = particles; edge between particles connected by a constraint.
//! - **Constraint graph**: nodes = constraints; edge between constraints that share a particle.
//! - **Islands**: connected components of the body graph (disjoint sets of particles).
//! - **Coloring**: vertex coloring of the constraint graph so same-color constraints are independent.

use crate::constraint::Constraint;
use mathlib::{connected_components_undirected, greedy_vertex_coloring, Graph};

/// Builds the body graph: `n` nodes (particles), undirected edge between two particles
/// if some constraint involves both.
#[must_use]
pub fn build_body_graph(n: usize, constraints: &[&dyn Constraint]) -> Graph {
    let mut g = Graph::new(n);
    for c in constraints {
        let np = c.num_particles();
        for i in 0..np {
            let pi = c.particle_index(i);
            for j in (i + 1)..np {
                let pj = c.particle_index(j);
                if pi != pj && pi < n && pj < n {
                    g.add_undirected_edge(pi, pj, 1.0);
                }
            }
        }
    }
    g
}

/// Builds the constraint graph: nodes = constraints; undirected edge between two constraints
/// if they share at least one particle.
#[must_use]
pub fn build_constraint_graph(constraints: &[&dyn Constraint]) -> Graph {
    let n = constraints.len();
    let mut g = Graph::new(n);
    for i in 0..n {
        for j in (i + 1)..n {
            if constraints_share_particle(constraints[i], constraints[j]) {
                g.add_undirected_edge(i, j, 1.0);
            }
        }
    }
    g
}

fn constraints_share_particle(a: &dyn Constraint, b: &dyn Constraint) -> bool {
    let na = a.num_particles();
    let nb = b.num_particles();
    for i in 0..na {
        let pi = a.particle_index(i);
        for j in 0..nb {
            if b.particle_index(j) == pi {
                return true;
            }
        }
    }
    false
}

/// Returns connected components of the body graph (islands). Each component is a list of particle indices.
#[must_use]
pub fn compute_islands(body_graph: &Graph) -> Vec<Vec<usize>> {
    connected_components_undirected(body_graph)
}

/// Returns particle index -> island index. Islands are ordered by size (largest first).
#[must_use]
pub fn particle_to_island(islands: &[Vec<usize>], num_particles: usize) -> Vec<usize> {
    let mut out = vec![0; num_particles];
    for (island_id, component) in islands.iter().enumerate() {
        for &p in component {
            if p < num_particles {
                out[p] = island_id;
            }
        }
    }
    out
}

/// Returns a vertex coloring of the constraint graph. Same-color constraints do not share a particle.
#[must_use]
pub fn constraint_colors(constraint_graph: &Graph) -> Vec<usize> {
    greedy_vertex_coloring(constraint_graph)
}

/// Groups constraint indices by (island_id, color). Batch order: by color then island.
/// Each constraint is assigned to the island of its first particle.
#[must_use]
pub fn constraint_batches(
    constraints: &[&dyn Constraint],
    colors: &[usize],
    particle_to_island: &[usize],
) -> Vec<Vec<usize>> {
    let num_colors = colors.iter().copied().max().map_or(0, |c| c + 1);
    let num_islands = particle_to_island.iter().copied().max().map_or(0, |i| i + 1);
    let mut batches: Vec<Vec<usize>> = Vec::new();
    for color in 0..num_colors {
        for island_id in 0..num_islands {
            let batch: Vec<usize> = constraints
                .iter()
                .enumerate()
                .filter(|(ci, c)| {
                    colors[*ci] == color
                        && c.num_particles() > 0
                        && particle_to_island
                            .get(c.particle_index(0))
                            .copied()
                            .unwrap_or(0)
                            == island_id
                })
                .map(|(ci, _)| ci)
                .collect();
            if !batch.is_empty() {
                batches.push(batch);
            }
        }
    }
    batches
}
