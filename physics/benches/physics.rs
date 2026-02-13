//! Criterion benchmarks for the physics crate (solver iterations, body count).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use physics::{
    DistanceConstraint, Integrator, Particle, PbdIntegrator, PhysicsState, PinConstraint,
};

fn bench_solver_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("solver");
    group.sample_size(100);

    group.bench_function("step_10_particles_9_constraints", |b| {
        let mut state = PhysicsState::with_particles(
            (0..10)
                .map(|i| Particle::new([i as f32 * 0.5, 0.0, 0.0], 1.0))
                .collect(),
        );
        state.config.substeps = 4;
        state.config.solver_iterations = 8;
        let mut integrator = PbdIntegrator::new();
        for i in 0..9 {
            integrator.add_constraint(Box::new(DistanceConstraint::new(i, i + 1, 0.5)));
        }
        b.iter(|| {
            integrator.step(black_box(&mut state), 1.0 / 60.0);
        });
    });

    group.bench_function("step_100_particles_99_constraints", |b| {
        let mut state = PhysicsState::with_particles(
            (0..100)
                .map(|i| Particle::new([i as f32 * 0.2, 0.0, 0.0], 1.0))
                .collect(),
        );
        state.config.substeps = 2;
        state.config.solver_iterations = 4;
        let mut integrator = PbdIntegrator::new();
        for i in 0..99 {
            integrator.add_constraint(Box::new(DistanceConstraint::new(i, i + 1, 0.2)));
        }
        b.iter(|| {
            integrator.step(black_box(&mut state), 1.0 / 60.0);
        });
    });

    group.bench_function("step_explicit_euler_1000_particles", |b| {
        let mut state = PhysicsState::with_particles(
            (0..1000)
                .map(|i| Particle::new([i as f32 * 0.1, 0.0, 0.0], 1.0))
                .collect(),
        );
        let mut integrator = PbdIntegrator::new();
        b.iter(|| {
            integrator.step(black_box(&mut state), 1.0 / 60.0);
        });
    });

    group.bench_function("step_pin_constraint_100_particles", |b| {
        let mut state = PhysicsState::with_particles(
            (0..100)
                .map(|i| Particle::new([i as f32, 0.0, 0.0], 1.0))
                .collect(),
        );
        state.config.substeps = 4;
        state.config.solver_iterations = 8;
        let mut integrator = PbdIntegrator::new();
        integrator.add_constraint(Box::new(PinConstraint::new(0, [0.0, 0.0, 0.0])));
        b.iter(|| {
            integrator.step(black_box(&mut state), 1.0 / 60.0);
        });
    });

    group.finish();
}

criterion_group!(benches, bench_solver_step);
criterion_main!(benches);
