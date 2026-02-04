//! Benchmarks for 3D math: points, homogeneous coords, transforms.

use criterion::{Criterion, black_box, criterion_group};
use mathlib::{
    Matrix4f, Point3, Vector3f, center, from_homogeneous_point, from_homogeneous_vector,
    make_rotation, matrix4_mul_vector3, matrix4f_inverse, point_to_homogeneous, transform_vector,
    vector_to_homogeneous,
};

fn sample_matrix4() -> Matrix4f {
    let mut m = Matrix4f::with_dimensions(4, 4);
    m.set_identity();
    m.set(0, 3, 1.0);
    m.set(1, 3, 2.0);
    m.set(2, 3, 3.0);
    let r = make_rotation(0.1, 0.2, 0.3);
    for i in 0..3 {
        for j in 0..3 {
            m.set(i, j, r.get(i, j));
        }
    }
    m
}

fn sample_vector3() -> Vector3f {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, 1.0);
    v.set(1, 2.0);
    v.set(2, 3.0);
    v
}

fn sample_point() -> Point3 {
    Point3::new(1.0, 2.0, 3.0)
}

pub fn bench_math3d(c: &mut Criterion) {
    let mut group = c.benchmark_group("math3d");

    group.bench_function("matrix4_mul_vector3", |b| {
        let m = sample_matrix4();
        let v = sample_vector3();
        b.iter(|| black_box(matrix4_mul_vector3(black_box(&m), black_box(&v))))
    });

    group.bench_function("transform_vector", |b| {
        let m = sample_matrix4();
        let v = sample_vector3();
        b.iter(|| black_box(transform_vector(black_box(&m), black_box(&v))))
    });

    group.bench_function("make_rotation", |b| {
        b.iter(|| {
            black_box(make_rotation(
                black_box(0.1),
                black_box(0.2),
                black_box(0.3),
            ))
        })
    });

    group.bench_function("matrix4f_inverse", |b| {
        let m = sample_matrix4();
        b.iter(|| black_box(matrix4f_inverse(black_box(&m))))
    });

    group.bench_function("point_add_vector", |b| {
        let p = sample_point();
        let v = sample_vector3();
        b.iter(|| black_box(black_box(&p) + black_box(v.clone())))
    });

    group.bench_function("point_sub_point", |b| {
        let a = sample_point();
        let b_pt = Point3::new(4.0, 5.0, 6.0);
        b.iter(|| black_box(black_box(&a) - black_box(&b_pt)))
    });

    group.bench_function("center", |b| {
        let a = sample_point();
        let b_pt = Point3::new(4.0, 5.0, 6.0);
        b.iter(|| black_box(center(black_box(&a), black_box(&b_pt))))
    });

    group.bench_function("point_to_homogeneous", |b| {
        let p = sample_point();
        b.iter(|| black_box(point_to_homogeneous(black_box(&p))))
    });

    group.bench_function("from_homogeneous_point", |b| {
        let p = sample_point();
        let h = point_to_homogeneous(&p);
        b.iter(|| black_box(from_homogeneous_point(black_box(&h))))
    });

    group.bench_function("vector_to_homogeneous", |b| {
        let v = sample_vector3();
        b.iter(|| black_box(vector_to_homogeneous(black_box(&v))))
    });

    group.bench_function("from_homogeneous_vector", |b| {
        let v = sample_vector3();
        let h = vector_to_homogeneous(&v);
        b.iter(|| black_box(from_homogeneous_vector(black_box(&h))))
    });

    group.finish();
}

criterion_group!(benches, bench_math3d);
