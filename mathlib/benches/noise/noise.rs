//! Benchmarks for noise (wave_2d, perlin_2d, fbm_2d).

use criterion::{Criterion, black_box, criterion_group};
use mathlib::noise::{fbm_2d, perlin_2d, wave_2d};

fn bench_noise(c: &mut Criterion) {
    let mut group = c.benchmark_group("noise");

    group.bench_function("wave_2d", |b| {
        b.iter(|| black_box(wave_2d(black_box(0.5), black_box(0.5))))
    });
    group.bench_function("perlin_2d", |b| {
        b.iter(|| black_box(perlin_2d(black_box(1.0), black_box(2.0))))
    });
    group.bench_function("fbm_2d_4oct", |b| {
        b.iter(|| {
            black_box(fbm_2d(
                black_box(1.0),
                black_box(1.0),
                4,
                2.0,
                0.5,
                perlin_2d,
            ))
        })
    });

    group.finish();
}

criterion_group!(benches, bench_noise);
