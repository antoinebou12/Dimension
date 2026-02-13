//! Example: 2D particles with distance constraint.

use physics::{DistanceConstraint, Integrator, Particle, PbdIntegrator, PhysicsState};

fn main() {
    let mut state = PhysicsState::with_particles(vec![
        Particle::new([0.0, 5.0, 0.0], 1.0),
        Particle::new([1.0, 5.0, 0.0], 1.0),
    ]);
    state.config.gravity = [0.0, -9.81, 0.0];

    let mut integrator = PbdIntegrator::new();
    integrator.add_constraint(Box::new(DistanceConstraint::new(0, 1, 1.0)));

    for frame in 0..120 {
        integrator.step(&mut state, 1.0 / 60.0);
        if frame % 30 == 0 {
            println!(
                "t={:.2}s  p0={:?}  p1={:?}",
                (frame + 1) as f32 / 60.0,
                state.particles[0].x,
                state.particles[1].x
            );
        }
    }
}
