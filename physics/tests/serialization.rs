//! Serialization tests (feature `serde`).

#![cfg(feature = "serde")]

use physics::{
    Particle, PhysicsState,
    serialization::{from_bytes, from_json, to_bytes, to_json},
};

#[test]
fn round_trip_json() {
    let state = PhysicsState::with_particles(vec![
        Particle::new([1.0, 2.0, 3.0], 1.0),
        Particle::kinematic([0.0, 0.0, 0.0]),
    ]);
    let json = to_json(&state).expect("serialize");
    let restored: PhysicsState = from_json(&json).expect("deserialize");
    assert_eq!(restored.particles.len(), state.particles.len());
    assert!((restored.particles[0].x[0] - state.particles[0].x[0]).abs() < 1e-6);
}

#[test]
fn round_trip_bytes() {
    let state = PhysicsState::with_particles(vec![Particle::new([0.0, 1.0, 0.0], 0.5)]);
    let bytes = to_bytes(&state).expect("serialize");
    let restored = from_bytes(&bytes).expect("deserialize");
    assert_eq!(restored.num_particles(), 1);
    assert_eq!(restored.particles[0].inv_mass, 0.5);
}
