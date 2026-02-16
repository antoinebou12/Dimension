//! Criterion benchmarks for the physics crate (solver iterations, body count).

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use physics::{
    CollisionShape, DistanceConstraint, Integrator, Particle, PbdIntegrator, PhysicsState,
    PinConstraint, PositionalConstraint, RigidBilateralSolver, RigidBody, XpbdIntegrator,
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

fn bench_matrix_free_pgs_contacts(c: &mut Criterion) {
    // Benchmark the matrix-free PGS contact solver scaling (like BM_MatrixFreePGS_Solve).
    let mut group = c.benchmark_group("MatrixFreePGS_Solve");
    group.sample_size(50);

    for n_contacts in [10, 100, 500] {
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{}Contacts", n_contacts)),
            &n_contacts,
            |b, &n_contacts| {
                let mut state = PhysicsState::default();
                state.config.substeps = 1;
                state.config.solver_iterations = 4;
                state.config.gravity = [0.0, -9.81, 0.0];
                state.rigid_bodies.push(RigidBody::kinematic(
                    [0.0, 0.0, 0.0],
                    CollisionShape::Box {
                        half_extents: [100.0, 0.1, 100.0],
                    },
                ));
                for i in 0..(n_contacts + 1) {
                    state.rigid_bodies.push(RigidBody::new(
                        [i as f32 * 0.3, 1.0, 0.0],
                        1.0,
                        CollisionShape::Sphere { radius: 0.15 },
                    ));
                }
                let mut integrator = XpbdIntegrator::new();
                integrator.rigid_contacts = (1..=n_contacts)
                    .map(|i| {
                        physics::RigidContactConstraint::new(
                            i,
                            0,
                            [0.0, -0.15, 0.0],
                            [i as f32 * 0.3, 0.0, 0.0],
                            [0.0, 1.0, 0.0],
                            0.01,
                        )
                    })
                    .collect();
                b.iter(|| integrator.step(black_box(&mut state), 1.0 / 60.0));
            },
        );
    }
    group.finish();
}

fn bench_rigid_bilateral_pgs_vs_cg(c: &mut Criterion) {
    let mut group = c.benchmark_group("rigid_bilateral");
    group.sample_size(40);

    for n_joints in [5, 15, 30] {
        let n_joints = n_joints;
        group.bench_with_input(
            BenchmarkId::new("PGS", format!("{}Joints", n_joints)),
            &n_joints,
            |b, &n_joints| {
                let mut state = PhysicsState::new();
                state.config.gravity = [0.0, -9.81, 0.0];
                state.config.substeps = 2;
                state.config.solver_iterations = 6;
                state.config.rigid_bilateral_solver = RigidBilateralSolver::Pgs;
                for i in 0..=n_joints {
                    state.add_rigid_body(RigidBody::new(
                        [i as f32 * 0.4, 2.0, 0.0],
                        1.0,
                        CollisionShape::Sphere { radius: 0.2 },
                    ));
                }
                let mut integrator = XpbdIntegrator::new();
                for i in 0..n_joints {
                    integrator.add_positional_constraint(PositionalConstraint::new(
                        i,
                        i + 1,
                        [0.2, 0.0, 0.0],
                        [-0.2, 0.0, 0.0],
                        0.0,
                    ));
                }
                b.iter(|| integrator.step(black_box(&mut state), 1.0 / 60.0));
            },
        );

        group.bench_with_input(
            BenchmarkId::new("CG", format!("{}Joints", n_joints)),
            &n_joints,
            |b, &n_joints| {
                let mut state = PhysicsState::new();
                state.config.gravity = [0.0, -9.81, 0.0];
                state.config.substeps = 2;
                state.config.solver_iterations = 6;
                state.config.rigid_bilateral_solver = RigidBilateralSolver::ConjugateGradient;
                state.config.cg_max_iter = 50;
                state.config.cg_tolerance = 1e-10;
                for i in 0..=n_joints {
                    state.add_rigid_body(RigidBody::new(
                        [i as f32 * 0.4, 2.0, 0.0],
                        1.0,
                        CollisionShape::Sphere { radius: 0.2 },
                    ));
                }
                let mut integrator = XpbdIntegrator::new();
                for i in 0..n_joints {
                    integrator.add_positional_constraint(PositionalConstraint::new(
                        i,
                        i + 1,
                        [0.2, 0.0, 0.0],
                        [-0.2, 0.0, 0.0],
                        0.0,
                    ));
                }
                b.iter(|| integrator.step(black_box(&mut state), 1.0 / 60.0));
            },
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_solver_step,
    bench_matrix_free_pgs_contacts,
    bench_rigid_bilateral_pgs_vs_cg
);
criterion_main!(benches);
