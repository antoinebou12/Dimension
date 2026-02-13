# physics — Agent summary

PBD/XPBD crate: particles, constraints, contact, rigid bodies, spatial (broad phase), islands (graph coloring, parallel solve).

## Layout

| Path | Role |
|------|------|
| `src/lib.rs` | Crate root, prelude |
| `src/particle.rs` | Particle (x, v, inv_mass, phase, radius) |
| `src/constraint.rs` | Constraint trait; DistanceConstraint, PinConstraint, PlaneConstraint |
| `src/contact.rs` | ContactConstraint (non-penetration) |
| `src/rigid.rs` | ShapeMatchingConstraint (SVD polar decomp) |
| `src/state.rs` | PhysicsConfig, PhysicsState |
| `src/integration.rs` | Integrator trait; PbdIntegrator, ExplicitEulerIntegrator |
| `src/solver.rs` | step_pbd (substeps, pre-stab, constraint solve, velocity, sleeping); optional batch/parallel |
| `src/hooks.rs` | SubstepHooks (pre_substep, post_substep) |
| `src/spatial.rs` | build_bvh, broad_phase_pairs, aabb_overlap |
| `src/islands.rs` | build_body_graph, build_constraint_graph, compute_islands, constraint_colors, constraint_batches |
| `src/serialization.rs` | to_json, from_json, to_bytes, from_bytes (feature serde) |
| `tests/physics.rs` | Integration tests |
| `benches/physics.rs` | Criterion benchmarks |
| `demo/` | physics-demo crate (winit + WASM) |

## Features

- `simd`, `parallel` (native only), `serde`, `wasm`

## Conventions

- Same as main repo: cargo fmt, clippy, doc on public items, Result for fallible ops.
