//! Benchmarks for kinematics crate.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use kinematics::joints::RevoluteJoint;
use kinematics::{Armature, JointData, JointVariant};
use mathlib::Vector3f;
use mathlib::cg::vector3;

fn bench_update_kinematics(c: &mut Criterion) {
    let mut t = Vector3f::with_capacity(3);
    t.set_zero();
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

criterion_group!(benches, bench_update_kinematics);
criterion_main!(benches);
