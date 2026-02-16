//! Benchmarks for computer-graphics recipes.

use criterion::{Criterion, criterion_group};
use mathlib::{
    Perspective3, from_euler_angles, from_scaled_axis, look_at_rh, model_view_projection,
    new_perspective, new_scaling, new_translation, transform_point, transform_vector, vector3,
};
use std::hint::black_box;

pub fn bench_cg_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("cg_construction");

    group.bench_function("new_scaling", |b| b.iter(|| new_scaling(black_box(2.0f32))));
    group.bench_function("new_translation", |b| {
        let t = vector3(1.0, 2.0, 3.0);
        b.iter(|| new_translation(black_box(&t)))
    });
    group.bench_function("from_scaled_axis", |b| {
        let ax = vector3(0.0, 0.0, 0.1);
        b.iter(|| from_scaled_axis(black_box(&ax)))
    });
    group.bench_function("from_euler_angles", |b| {
        b.iter(|| from_euler_angles(black_box(0.1), black_box(0.2), black_box(0.3)))
    });
    group.bench_function("new_perspective", |b| {
        b.iter(|| new_perspective(black_box(16.0 / 9.0), black_box(1.57), 1.0, 1000.0))
    });
    group.bench_function("look_at_rh", |b| {
        let eye = vector3(0.0, 0.0, 5.0);
        let target = vector3(0.0, 0.0, 0.0);
        let up = vector3(0.0, 1.0, 0.0);
        b.iter(|| look_at_rh(black_box(&eye), black_box(&target), black_box(&up)))
    });

    group.finish();
}

pub fn bench_cg_modifiers(c: &mut Criterion) {
    let mut group = c.benchmark_group("cg_modifiers");

    let m = new_scaling(2.0);
    let t = vector3(10.0, 0.0, 0.0);

    group.bench_function("append_translation_mut", |b| {
        b.iter(|| {
            let mut m2 = m.clone();
            mathlib::append_translation_mut(&mut m2, black_box(&t));
            black_box(m2)
        })
    });
    group.bench_function("prepend_scaling_mut", |b| {
        b.iter(|| {
            let mut m2 = m.clone();
            mathlib::prepend_scaling_mut(&mut m2, black_box(1.5));
            black_box(m2)
        })
    });

    group.finish();
}

pub fn bench_cg_transform(c: &mut Criterion) {
    let mut group = c.benchmark_group("cg_transform");

    let m = new_scaling(2.0);
    let p = vector3(1.0, 2.0, 3.0);

    group.bench_function("transform_point", |b| {
        b.iter(|| black_box(transform_point(black_box(&m), black_box(&p))))
    });
    group.bench_function("transform_vector", |b| {
        b.iter(|| black_box(transform_vector(black_box(&m), black_box(&p))))
    });

    group.finish();
}

pub fn bench_cg_mvp(c: &mut Criterion) {
    let mut group = c.benchmark_group("cg_mvp");

    let model = new_translation(&vector3(1.0, 0.0, 0.0));
    let eye = vector3(0.0, 0.0, 5.0);
    let view = look_at_rh(&eye, &vector3(0.0, 0.0, 0.0), &vector3(0.0, 1.0, 0.0));
    let proj = new_perspective(16.0 / 9.0, 1.57, 1.0, 1000.0);

    group.bench_function("model_view_projection", |b| {
        b.iter(|| model_view_projection(black_box(&model), black_box(&view), black_box(&proj)))
    });

    group.finish();
}

pub fn bench_cg_unproject(c: &mut Criterion) {
    let mut group = c.benchmark_group("cg_unproject");

    let proj = Perspective3::new(800.0 / 600.0, 1.57, 1.0, 1000.0);

    group.bench_function("unproject_point", |b| {
        let ndc = vector3(0.0, 0.0, -1.0);
        b.iter(|| black_box(proj.unproject_point(black_box(&ndc))))
    });
    group.bench_function("screen_to_view_ray", |b| {
        b.iter(|| {
            black_box(mathlib::screen_to_view_ray(
                black_box(&proj),
                black_box(400.0),
                black_box(300.0),
                800.0,
                600.0,
            ))
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_cg_construction,
    bench_cg_modifiers,
    bench_cg_transform,
    bench_cg_mvp,
    bench_cg_unproject
);
