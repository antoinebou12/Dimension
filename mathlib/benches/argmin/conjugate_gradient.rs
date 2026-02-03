//! Benchmarks for conjugate gradient (linear and nonlinear).

use criterion::{BenchmarkId, Criterion, black_box, criterion_group};
use mathlib::structure::Storage;
use mathlib::{Matrix, Vector};
use mathlib::{NonlinearCgOptions, nonlinear_cg, solve_cg};

fn make_spd(n: usize) -> Matrix<f64> {
    let mut a = Matrix::with_storage(n, n, Storage::Column);
    for i in 0..n {
        for j in 0..n {
            a.set(i, j, if i == j { 2.0 + (i as f64) * 0.1 } else { 0.1 });
        }
    }
    a
}

fn make_vector(data: &[f64]) -> Vector<f64> {
    let mut v = Vector::with_capacity(data.len());
    v.resize(data.len());
    for (i, &val) in data.iter().enumerate() {
        v.set(i, val);
    }
    v
}

pub fn bench_conjugate_gradient(c: &mut Criterion) {
    let mut group = c.benchmark_group("conjugate_gradient");

    for n in [8_usize, 32, 128] {
        let a = make_spd(n);
        let b = make_vector(&vec![1.0; n]);

        group.bench_with_input(
            BenchmarkId::new("solve_cg", n),
            &(a.clone(), b.clone()),
            |bencher, (a, rhs)| {
                bencher.iter(|| {
                    let x = solve_cg(black_box(a), black_box(rhs), 1e-12, n * 2).unwrap();
                    black_box(x)
                })
            },
        );
    }

    group.bench_function("nonlinear_cg_sphere_8d", |b| {
        let x0 = vec![2.0_f64; 8];
        let cost = |x: &[f64]| x.iter().map(|v| v * v).sum::<f64>();
        let gradient = |x: &[f64], g: &mut [f64]| {
            for (i, &xi) in x.iter().enumerate() {
                g[i] = 2.0 * xi;
            }
        };
        let opts = NonlinearCgOptions {
            max_iters: 300,
            tol: 1e-10,
            ..Default::default()
        };
        b.iter(|| {
            let result = nonlinear_cg(
                black_box(&x0),
                black_box(&cost),
                black_box(&gradient),
                black_box(&opts),
            );
            black_box(result)
        })
    });

    group.finish();
}

criterion_group!(benches, bench_conjugate_gradient);
