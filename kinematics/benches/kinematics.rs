//! Benchmarks for kinematics crate.
//!
//! Includes IK solver comparison: FABRIK, Jacobian, Hessian, and Halley on the same chain.
//!
//! Run with features to measure SIMD/parallel impact:
//!   cargo bench -p kinematics --features simd
//!   cargo bench -p kinematics --features parallel

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use kinematics::ik::{FabrikIk, FabrikSqpIk, HessianIk, JacobianIk};
use kinematics::joints::RevoluteJoint;
use kinematics::{Armature, JointData, JointVariant};
use mathlib::cg::vector3;

fn bench_update_kinematics(c: &mut Criterion) {
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::default()));
    let mut arm = Armature::new(root);
    for i in 1..10 {
        let j = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
            vector3(i as f32 * 1.0, 0.0, 0.0),
            (0.0, 1.0, 0.0),
            0.1 * i as f32,
        )));
        arm.add_child(i - 1, i, j);
    }
    arm.update_kinematics();

    c.bench_function("update_kinematics_10", |b| {
        b.iter(|| {
            arm.update_kinematics();
            black_box(&arm);
        })
    });
}

/// Builds a 6-link revolute chain (nodes 0..=6, end-effector at 6) for IK benchmarks.
fn ik_bench_armature() -> Armature {
    let root = JointData::new(JointVariant::Revolute(RevoluteJoint::new(
        vector3(0.0, 0.0, 0.0),
        (0.0, 1.0, 0.0),
        0.0,
    )));
    let mut arm = Armature::new(root);
    let link_len = 0.5;
    for i in 1..=6 {
        arm.add_child(
            i - 1,
            i,
            JointData::new(JointVariant::Revolute(RevoluteJoint::new(
                vector3(link_len, 0.0, 0.0),
                (0.0, 1.0, 0.0),
                0.0,
            ))),
        );
    }
    arm.update_kinematics();
    arm
}

fn bench_ik_fabrik(c: &mut Criterion) {
    let target = vector3(2.0, 0.5, 0.2);
    c.bench_function("ik_fabrik_6link", |b| {
        b.iter(|| {
            let mut arm = ik_bench_armature();
            let err = FabrikIk::new(&mut arm, 6, target.clone())
                .with_max_iters(20)
                .solve();
            black_box(err);
        })
    });
}

fn bench_ik_fabrik_sqp(c: &mut Criterion) {
    let target = vector3(2.0, 0.5, 0.2);
    c.bench_function("ik_fabrik_sqp_6link", |b| {
        b.iter(|| {
            let mut arm = ik_bench_armature();
            let err = FabrikSqpIk::new(&mut arm, 6, target.clone())
                .with_nl(10)
                .with_max_iters_fallback(50)
                .solve();
            black_box(err);
        })
    });
}

fn bench_ik_jacobian(c: &mut Criterion) {
    let target = vector3(2.0, 0.5, 0.2);
    c.bench_function("ik_jacobian_6link", |b| {
        b.iter(|| {
            let mut arm = ik_bench_armature();
            let err = JacobianIk::new(&mut arm, 6, target.clone())
                .with_max_iters(30)
                .solve();
            black_box(err);
        })
    });
}

fn bench_ik_hessian(c: &mut Criterion) {
    let target = vector3(2.0, 0.5, 0.2);
    c.bench_function("ik_hessian_6link", |b| {
        b.iter(|| {
            let mut arm = ik_bench_armature();
            let err = HessianIk::new(&mut arm, 6, target.clone())
                .with_max_iters(32)
                .solve();
            black_box(err);
        })
    });
}

fn bench_ik_halley(c: &mut Criterion) {
    use kinematics::ik::HalleyIk;
    use mathlib::cg::matrix4f_identity;

    let mut target = matrix4f_identity();
    target.set(0, 3, 2.0);
    target.set(1, 3, 0.5);
    target.set(2, 3, 0.2);

    c.bench_function("ik_halley_6link", |b| {
        b.iter(|| {
            let mut arm = ik_bench_armature();
            let err = HalleyIk::new(&mut arm, 6, target.clone())
                .with_max_iters(32)
                .solve();
            black_box(err);
        })
    });
}

fn bench_ik_batch_jacobian(c: &mut Criterion) {
    use kinematics::ik::solve_batch_jacobian;

    let target = vector3(2.0, 0.5, 0.2);
    let batch: Vec<_> = (0..8)
        .map(|_| (ik_bench_armature(), 6, target.clone()))
        .collect();

    c.bench_function("ik_batch_jacobian_8x6link", |b| {
        b.iter(|| {
            let batch_clone: Vec<_> = batch
                .iter()
                .map(|(arm, ee, t)| (arm.clone(), *ee, t.clone()))
                .collect();
            let results = solve_batch_jacobian(batch_clone);
            black_box(results);
        })
    });
}

criterion_group!(
    benches,
    bench_update_kinematics,
    bench_ik_fabrik,
    bench_ik_fabrik_sqp,
    bench_ik_jacobian,
    bench_ik_hessian,
    bench_ik_halley,
    bench_ik_batch_jacobian,
);

criterion_main!(benches);
