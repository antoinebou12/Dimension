//! Example: 3D soft body (jelly cube) with Neo-Hookean elasticity.

use physics::{Integrator, Particle, PhysicsState, SoftBody, TetMesh, XpbdIntegrator};

fn main() {
    // Generate a 3x3x3 tetrahedral cube
    let (verts, mesh) = TetMesh::cube(3, 3, 3, 0.5);
    let num_verts = verts.len();
    let num_tets = mesh.num_tets();

    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, -2.0, 0.0]; // gentle gravity
    state.config.substeps = 10;
    state.config.solver_iterations = 10;

    // Add vertices as particles (offset = 0)
    for v in &verts {
        state
            .particles
            .push(Particle::new([v[0], v[1] + 3.0, v[2]], 1.0)); // lift up
    }

    // Jelly-like material (very soft for stability)
    let (mu, lambda) = (100.0_f32, 500.0_f32);
    let sb = SoftBody::new(0, num_verts, mesh, mu, lambda);
    state.add_soft_body(sb);

    println!("Soft body: {num_verts} vertices, {num_tets} tetrahedra");
    println!("Material: mu={mu:.1}, lambda={lambda:.1}");

    let mut integrator = XpbdIntegrator::new();

    // Pin the bottom-center vertex to act as an anchor
    // (find vertex closest to bottom center)
    let center_x = 3.0 * 0.5 / 2.0;
    let center_z = 3.0 * 0.5 / 2.0;
    let mut best_idx = 0;
    let mut best_dist = f32::MAX;
    for (i, v) in verts.iter().enumerate() {
        if v[1] < 0.01 {
            let d = (v[0] - center_x).powi(2) + (v[2] - center_z).powi(2);
            if d < best_dist {
                best_dist = d;
                best_idx = i;
            }
        }
    }
    state.particles[best_idx].inv_mass = 0.0; // pin it

    for frame in 0..300 {
        integrator.step(&mut state, 1.0 / 60.0);
        if frame % 60 == 0 {
            // Compute center of mass
            let mut cx = 0.0_f32;
            let mut cy = 0.0_f32;
            let mut cz = 0.0_f32;
            for p in state.particles.iter().take(num_verts) {
                cx += p.x[0];
                cy += p.x[1];
                cz += p.x[2];
            }
            let n = num_verts as f32;
            println!(
                "t={:.1}s  center of mass = [{:.3}, {:.3}, {:.3}]",
                (frame + 1) as f32 / 60.0,
                cx / n,
                cy / n,
                cz / n
            );
        }
    }
}
