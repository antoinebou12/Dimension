//! WASM build / integration tests: physics types compile and work on wasm32.
//!
//! Run with: cargo test --target wasm32-unknown-unknown --test wasm --features wasm

#[cfg(target_arch = "wasm32")]
use physics::{Particle, PhysicsState};

#[cfg(target_arch = "wasm32")]
#[test]
fn particle_wasm() {
    let p = Particle::new([0.0, 1.0, 0.0], 1.0);
    assert_eq!(p.inv_mass, 1.0);
}

#[cfg(target_arch = "wasm32")]
#[test]
fn physics_state_wasm() {
    let state = PhysicsState::with_particles(vec![
        Particle::new([0.0, 0.0, 0.0], 1.0),
        Particle::new([1.0, 0.0, 0.0], 1.0),
    ]);
    assert_eq!(state.particles.len(), 2);
}

// When building for non-wasm, provide a dummy test so the test binary compiles.
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn wasm_test_skipped_on_native() {
    // This test binary is intended for wasm32; on native, run other tests instead.
}
