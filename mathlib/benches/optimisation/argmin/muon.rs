//! Benchmarks for Muon step.

use criterion::{BenchmarkId, Criterion, criterion_group};
use mathlib::structure::Storage;
use mathlib::{Matrix, muon_step};
use std::hint::black_box;

fn make_matrix_2d(n: usize, m: usize, fill: f64) -> Matrix<f64> {
    let mut a = Matrix::with_storage(n, m, Storage::Column);
    for i in 0..n {
        for j in 0..m {
            a.set(i, j, fill);
        }
    }
    a
}

pub fn bench_muon(c: &mut Criterion) {
    let mut group = c.benchmark_group("muon");

    for size in [8_usize, 32, 64] {
        group.bench_with_input(BenchmarkId::new("muon_step", size), &size, |b, &size| {
            let mut param = make_matrix_2d(size, size, 0.1);
            let grad = make_matrix_2d(size, size, 0.01);
            b.iter(|| {
                muon_step(black_box(&mut param), black_box(&grad), 0.1, 5);
                black_box(())
            })
        });
    }

    group.finish();
}

criterion_group!(benches, bench_muon);
