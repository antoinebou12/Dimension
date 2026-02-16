//! Tests for rigid body simulation.

use physics::{
    CollisionShape, Integrator, PhysicsState, PositionalConstraint, RigidBilateralSolver,
    RigidBody, RigidContactConstraint, XpbdIntegrator,
};

#[test]
fn rigid_body_creation() {
    let rb = RigidBody::new([0.0, 5.0, 0.0], 1.0, CollisionShape::Sphere { radius: 0.5 });
    assert!(!rb.is_kinematic());
    assert_eq!(rb.x, [0.0, 5.0, 0.0]);
    assert!((rb.inv_mass - 1.0).abs() < 1e-6);
    assert_eq!(rb.q, [1.0, 0.0, 0.0, 0.0]); // identity quaternion
}

#[test]
fn rigid_body_kinematic() {
    let rb = RigidBody::kinematic(
        [0.0, 0.0, 0.0],
        CollisionShape::Box {
            half_extents: [1.0, 1.0, 1.0],
        },
    );
    assert!(rb.is_kinematic());
    assert_eq!(rb.inv_mass, 0.0);
}

#[test]
fn rigid_body_free_fall() {
    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, -9.81, 0.0];
    state.add_rigid_body(RigidBody::new(
        [0.0, 10.0, 0.0],
        1.0,
        CollisionShape::Sphere { radius: 0.5 },
    ));

    let mut integrator = XpbdIntegrator::new();

    let initial_y = state.rigid_bodies[0].x[1];
    for _ in 0..60 {
        integrator.step(&mut state, 1.0 / 60.0);
    }
    // After 1 second of free-fall, y should decrease significantly
    assert!(
        state.rigid_bodies[0].x[1] < initial_y - 2.0,
        "body should fall: y = {}",
        state.rigid_bodies[0].x[1]
    );
}

#[test]
fn rigid_body_kinematic_no_motion() {
    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, -9.81, 0.0];
    state.add_rigid_body(RigidBody::kinematic(
        [0.0, 0.0, 0.0],
        CollisionShape::Box {
            half_extents: [10.0, 0.1, 10.0],
        },
    ));

    let mut integrator = XpbdIntegrator::new();
    for _ in 0..60 {
        integrator.step(&mut state, 1.0 / 60.0);
    }
    assert_eq!(state.rigid_bodies[0].x, [0.0, 0.0, 0.0]);
}

#[test]
fn ball_joint_keeps_bodies_together() {
    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, 0.0, 0.0];
    state.config.substeps = 4;
    state.config.solver_iterations = 8;

    // Body A at origin, body B at (2,0,0)
    state.add_rigid_body(RigidBody::new(
        [0.0, 0.0, 0.0],
        1.0,
        CollisionShape::Sphere { radius: 0.5 },
    ));
    state.add_rigid_body(RigidBody::new(
        [2.0, 0.0, 0.0],
        1.0,
        CollisionShape::Sphere { radius: 0.5 },
    ));

    let mut integrator = XpbdIntegrator::new();
    // Ball joint connecting body 0 at (+0.5,0,0) to body 1 at (-0.5,0,0)
    integrator.add_positional_constraint(PositionalConstraint::new(
        0,
        1,
        [0.5, 0.0, 0.0],
        [-0.5, 0.0, 0.0],
        0.0, // zero compliance = rigid
    ));

    for _ in 0..120 {
        integrator.step(&mut state, 1.0 / 60.0);
    }

    // The attachment points should be very close together
    let pa = [
        state.rigid_bodies[0].x[0] + 0.5,
        state.rigid_bodies[0].x[1],
        state.rigid_bodies[0].x[2],
    ];
    let pb = [
        state.rigid_bodies[1].x[0] - 0.5,
        state.rigid_bodies[1].x[1],
        state.rigid_bodies[1].x[2],
    ];
    let dist = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    assert!(
        dist < 0.5,
        "ball joint should keep attachment points together, got dist={dist}"
    );
}

#[test]
fn rigid_body_inv_inertia_world() {
    let rb = RigidBody::new([0.0, 0.0, 0.0], 1.0, CollisionShape::Sphere { radius: 1.0 });
    let iw = rb.inv_inertia_world();
    // For identity orientation, world inertia == body inertia
    for i in 0..9 {
        assert!(
            (iw[i] - rb.inv_inertia[i]).abs() < 1e-5,
            "inv_inertia_world[{i}] = {} but expected {}",
            iw[i],
            rb.inv_inertia[i]
        );
    }
}

#[test]
fn collision_shape_bounding_radius() {
    let s = CollisionShape::Sphere { radius: 2.0 };
    assert!((s.bounding_radius() - 2.0).abs() < 1e-6);

    let b = CollisionShape::Box {
        half_extents: [1.0, 1.0, 1.0],
    };
    assert!((b.bounding_radius() - 3.0_f32.sqrt()).abs() < 1e-5);
}

#[test]
fn contact_friction_lambda_clamped() {
    // Contact with friction: after solve, tangent lambdas must be in [-mu*lambda_n, mu*lambda_n]
    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, -9.81, 0.0];
    state.config.contact_friction = 0.5;
    state.config.substeps = 1;
    state.config.solver_iterations = 4;
    state.rigid_bodies.push(RigidBody::kinematic(
        [0.0, 0.0, 0.0],
        CollisionShape::Box {
            half_extents: [10.0, 0.1, 10.0],
        },
    ));
    state.rigid_bodies.push(RigidBody::new(
        [0.0, 1.0, 0.0],
        1.0,
        CollisionShape::Sphere { radius: 0.5 },
    ));
    let mut integrator = XpbdIntegrator::new();
    integrator.rigid_contacts.push(RigidContactConstraint::new(
        1,
        0,
        [0.0, -0.5, 0.0],
        [0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        0.1,
    ));
    for _ in 0..10 {
        integrator.step(&mut state, 1.0 / 60.0);
    }
    let c = &integrator.rigid_contacts[0];
    let mu = 0.5;
    let lambda_n = c.lambda_accum[0];
    let bound = mu * lambda_n;
    assert!(
        c.lambda_accum[1] >= -bound - 1e-5 && c.lambda_accum[1] <= bound + 1e-5,
        "tangent1 lambda should be in [-{bound}, {bound}], got {}",
        c.lambda_accum[1]
    );
    assert!(
        c.lambda_accum[2] >= -bound - 1e-5 && c.lambda_accum[2] <= bound + 1e-5,
        "tangent2 lambda should be in [-{bound}, {bound}], got {}",
        c.lambda_accum[2]
    );
}

#[test]
fn ball_joint_cg_solver() {
    // Ball joint with CG bilateral solver should keep bodies together (same as PGS)
    let mut state = PhysicsState::new();
    state.config.gravity = [0.0, 0.0, 0.0];
    state.config.substeps = 4;
    state.config.solver_iterations = 8;
    state.config.rigid_bilateral_solver = RigidBilateralSolver::ConjugateGradient;
    state.config.cg_max_iter = 30;
    state.config.cg_tolerance = 1e-10;

    state.add_rigid_body(RigidBody::new(
        [0.0, 0.0, 0.0],
        1.0,
        CollisionShape::Sphere { radius: 0.5 },
    ));
    state.add_rigid_body(RigidBody::new(
        [2.0, 0.0, 0.0],
        1.0,
        CollisionShape::Sphere { radius: 0.5 },
    ));

    let mut integrator = XpbdIntegrator::new();
    integrator.add_positional_constraint(PositionalConstraint::new(
        0,
        1,
        [0.5, 0.0, 0.0],
        [-0.5, 0.0, 0.0],
        0.0,
    ));

    for _ in 0..120 {
        integrator.step(&mut state, 1.0 / 60.0);
    }

    let pa = [
        state.rigid_bodies[0].x[0] + 0.5,
        state.rigid_bodies[0].x[1],
        state.rigid_bodies[0].x[2],
    ];
    let pb = [
        state.rigid_bodies[1].x[0] - 0.5,
        state.rigid_bodies[1].x[1],
        state.rigid_bodies[1].x[2],
    ];
    let dist = ((pa[0] - pb[0]).powi(2) + (pa[1] - pb[1]).powi(2) + (pa[2] - pb[2]).powi(2)).sqrt();
    assert!(
        dist < 0.5,
        "CG ball joint should keep attachment points together, got dist={dist}"
    );
}
