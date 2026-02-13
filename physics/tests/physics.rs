//! Integration tests for the physics crate.

use physics::{
    Constraint, DistanceConstraint, Integrator, Particle, PbdIntegrator, PhysicsState,
    PinConstraint, PlaneConstraint, XpbdIntegrator,
};

#[test]
fn particle_new() {
    let p = Particle::new([1.0, 2.0, 3.0], 0.5);
    assert_eq!(p.x, [1.0, 2.0, 3.0]);
    assert_eq!(p.inv_mass, 0.5);
    assert!(!p.is_kinematic());
}

#[test]
fn particle_kinematic() {
    let p = Particle::kinematic([0.0, 0.0, 0.0]);
    assert!(p.is_kinematic());
    assert_eq!(p.inv_mass, 0.0);
}

#[test]
fn state_with_particles() {
    let state = PhysicsState::with_particles(vec![
        Particle::new([0.0, 0.0, 0.0], 1.0),
        Particle::new([1.0, 0.0, 0.0], 1.0),
    ]);
    assert_eq!(state.num_particles(), 2);
    assert_eq!(state.num_rigid_bodies(), 0);
    assert_eq!(state.num_soft_bodies(), 0);
}

#[test]
fn gravity_applied() {
    let mut state = PhysicsState::with_particles(vec![Particle::new([0.0, 10.0, 0.0], 1.0)]);
    state.config.gravity = [0.0, -9.81, 0.0];
    let mut integrator = PbdIntegrator::new();
    integrator.step(&mut state, 1.0 / 60.0);
    // After one frame with gravity, particle should have moved down
    assert!(state.particles[0].x[1] < 10.0);
}

#[test]
fn zero_gravity_no_motion() {
    let mut state = PhysicsState::with_particles(vec![Particle::new([0.0, 0.0, 0.0], 1.0)]);
    state.config.gravity = [0.0, 0.0, 0.0];
    let mut integrator = PbdIntegrator::new();
    integrator.step(&mut state, 1.0 / 60.0);
    assert!((state.particles[0].x[0]).abs() < 1e-6);
    assert!((state.particles[0].x[1]).abs() < 1e-6);
    assert!((state.particles[0].x[2]).abs() < 1e-6);
}

#[test]
fn distance_constraint_solve() {
    let mut state = PhysicsState::with_particles(vec![
        Particle::new([0.0, 0.0, 0.0], 1.0),
        Particle::new([3.0, 0.0, 0.0], 1.0),
    ]);
    state.config.gravity = [0.0, 0.0, 0.0];
    state.config.substeps = 4;
    state.config.solver_iterations = 8;

    let mut integrator = PbdIntegrator::new();
    integrator.add_constraint(Box::new(DistanceConstraint::new(0, 1, 1.0)));

    // Run several frames; particles should converge toward distance 1.0
    for _ in 0..120 {
        integrator.step(&mut state, 1.0 / 60.0);
    }

    let dx = state.particles[1].x[0] - state.particles[0].x[0];
    let dy = state.particles[1].x[1] - state.particles[0].x[1];
    let dz = state.particles[1].x[2] - state.particles[0].x[2];
    let dist = (dx * dx + dy * dy + dz * dz).sqrt();
    assert!(
        (dist - 1.0).abs() < 0.1,
        "distance should be near 1.0, got {dist}"
    );
}

#[test]
fn pin_constraint_holds() {
    let mut state = PhysicsState::with_particles(vec![Particle::new([5.0, 0.0, 0.0], 1.0)]);
    state.config.gravity = [0.0, 0.0, 0.0];
    state.config.substeps = 2;
    state.config.solver_iterations = 4;

    let mut integrator = PbdIntegrator::new();
    integrator.add_constraint(Box::new(PinConstraint::new(0, [0.0, 0.0, 0.0])));

    for _ in 0..60 {
        integrator.step(&mut state, 1.0 / 60.0);
    }

    let dist = (state.particles[0].x[0].powi(2)
        + state.particles[0].x[1].powi(2)
        + state.particles[0].x[2].powi(2))
    .sqrt();
    assert!(
        dist < 0.5,
        "particle should be near origin, got dist={dist}"
    );
}

#[test]
fn distance_constraint_num_particles() {
    let c = DistanceConstraint::new(0, 1, 1.0);
    assert_eq!(c.num_particles(), 2);
    assert_eq!(c.particle_index(0), 0);
    assert_eq!(c.particle_index(1), 1);
}

#[test]
fn pin_constraint_num_particles() {
    let c = PinConstraint::new(0, [1.0, 0.0, 0.0]);
    assert_eq!(c.num_particles(), 1);
    assert_eq!(c.particle_index(0), 0);
}

#[test]
fn xpbd_integrator_basic() {
    let mut state = PhysicsState::with_particles(vec![Particle::new([0.0, 5.0, 0.0], 1.0)]);
    state.config.gravity = [0.0, -9.81, 0.0];

    let mut integrator = XpbdIntegrator::new();
    integrator.step(&mut state, 1.0 / 60.0);
    assert!(state.particles[0].x[1] < 5.0, "particle should fall");
}

#[test]
fn plane_constraint_keeps_particles_above_ground() {
    const EPS: f32 = 1e-4;
    let ground = [0.0, 0.0, 0.0];
    let normal = [0.0, 1.0, 0.0];

    let mut state = PhysicsState::with_particles(vec![
        Particle::new([0.0, 0.5, 0.0], 1.0).with_radius(0.05),
        Particle::new([0.2, 0.3, 0.0], 1.0).with_radius(0.05),
    ]);
    state.config.gravity = [0.0, -9.81, 0.0];
    state.config.substeps = 4;
    state.config.solver_iterations = 8;

    let mut integrator = PbdIntegrator::new();
    integrator.add_constraint(Box::new(PlaneConstraint::new(0, ground, normal)));
    integrator.add_constraint(Box::new(PlaneConstraint::new(1, ground, normal)));

    for _ in 0..120 {
        integrator.step(&mut state, 1.0 / 60.0);
    }

    assert!(
        state.particles[0].x[1] >= -EPS,
        "particle 0 should stay at or above plane, got y={}",
        state.particles[0].x[1]
    );
    assert!(
        state.particles[1].x[1] >= -EPS,
        "particle 1 should stay at or above plane, got y={}",
        state.particles[1].x[1]
    );
}
