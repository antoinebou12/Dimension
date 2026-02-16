# Physics crate — PBD/XPBD simulation

The **physics** crate provides position-based dynamics (PBD/XPBD) for real-time simulation: particles, constraints, contact (using the **collision** crate), rigid body shape-matching, and optional parallel solving via graph coloring and island decomposition.

## Overview

- **Particles**: Position, velocity, inverse mass, phase (for grouping), and optional collision radius.
- **Constraints**: Trait `Constraint`; built-in `DistanceConstraint`, `PinConstraint`, `PlaneConstraint` (half-space/ground), `ContactConstraint`, `ShapeMatchingConstraint`.
- **Solver**: Substeps, predict, constraint solve (with SOR and constraint averaging), velocity update, sleeping. Pre-stabilization for contacts. Optional parallel solve by color/island when feature `parallel` is enabled.
- **Collision**: Broad phase (`broad_phase_pairs`, `build_bvh`) and contact constraints; uses **collision** `Aabb`, `Sphere`, etc. No reimplementation of overlap logic.
- **Rigid bodies**: `ShapeMatchingConstraint` (covariance + SVD polar decomposition via mathlib).
- **Serialization**: Feature `serde` for `PhysicsState` (to_json/from_json, to_bytes/from_bytes).

## Build and run

From repo root:

```bash
just build-physics
just test-physics
just bench-physics
just run-physics-demo          # native (winit)
just build-physics-demo-wasm   # WASM demo
just wasm-physics-demo         # serve WASM demo
```

Features: `simd`, `parallel` (native only), `serde`, `wasm`.

## Main types

| Type | Role |
|------|------|
| `Particle` | x, v, inv_mass, phase, radius |
| `PhysicsState` | particles, config (dt, substeps, solver_iterations, etc.) |
| `PhysicsConfig` | dt, substeps, solver_iterations, stabilization_iterations, sor_omega, sleep_threshold, contact_friction, contact_restitution, contact_rolling_friction |
| `Constraint` | trait: num_particles, particle_index, solve |
| `PbdIntegrator` | Holds constraints and contact_constraints; Integrator::step runs solver |
| `PlaneConstraint` | Half-space (e.g. ground): keeps particle on or above a plane |
| `ContactConstraint` | Non-penetration (sphere–sphere); used with pre-stabilization |
| `ShapeMatchingConstraint` | Rigid body: rest positions, SVD rotation |
| `SubstepHooks` | pre_substep, post_substep callbacks |

## Connector (physics-demo)

`physics-demo` mirrors **kinematics-demo**: shared `scene.rs` with `PhysicsScene`, `build_physics_scene`, `step_physics`, `sync_bodies_to_world`. Native (winit) and WASM use the same scene; bodies are synced to render `World` via `world.set_transform(ent, Transform { position, ... })`. The demo uses **plane constraints** (ground at y = 0) and **contact constraints** (all-pairs sphere collision) so particles rest on a floor and collide with each other; a ground quad is rendered for reference.

## References

- Macklin et al., "Unified Particle Physics for Real-Time Applications", ACM TOG 33(4).
- Plan: physics package with XPBD (this crate and physics-demo).
