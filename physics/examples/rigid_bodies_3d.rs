//! Example: 3D rigid bodies with gravity and a ball joint.

use physics::{
    CollisionShape, Integrator, PhysicsState, PositionalConstraint, RigidBody, XpbdIntegrator,
};

fn main() {
    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, -9.81, 0.0];
    state.config.substeps = 4;
    state.config.solver_iterations = 8;

    // Fixed anchor body (kinematic)
    state.add_rigid_body(RigidBody::kinematic(
        [0.0, 5.0, 0.0],
        CollisionShape::Sphere { radius: 0.1 },
    ));
    // Pendulum bob
    state.add_rigid_body(RigidBody::new(
        [2.0, 5.0, 0.0],
        1.0,
        CollisionShape::Sphere { radius: 0.3 },
    ));

    let mut integrator = XpbdIntegrator::new();
    integrator.add_positional_constraint(PositionalConstraint::new(
        0,
        1,
        [0.0, 0.0, 0.0],
        [-2.0, 0.0, 0.0],
        0.0,
    ));

    for frame in 0..300 {
        integrator.step(&mut state, 1.0 / 60.0);
        if frame % 60 == 0 {
            let rb = &state.rigid_bodies[1];
            println!(
                "t={:.1}s  bob pos={:?}  vel={:?}",
                (frame + 1) as f32 / 60.0,
                rb.x,
                rb.v
            );
        }
    }
}
