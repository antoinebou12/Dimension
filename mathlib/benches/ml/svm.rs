//! Benchmarks for linear SVM and RBF-kernel SVM.

use criterion::{BenchmarkId, Criterion, criterion_group};
use mathlib::{Matrix, Storage, SvmOptions, svm, svm_rbf};
use std::hint::black_box;
use std::time::Duration;

fn synthetic_data_and_labels(n_samples: usize, n_features: usize) -> (Matrix<f64>, Vec<f64>) {
    let mut m = Matrix::with_storage(n_samples, n_features, Storage::Column);
    let mut y = Vec::with_capacity(n_samples);
    for i in 0..n_samples {
        for j in 0..n_features {
            m.set(i, j, (i as f64) * 0.13 + (j as f64) * 0.07 - 0.5);
        }
        y.push(if m.get(i, 0) > 0.0 { 1.0 } else { -1.0 });
    }
    (m, y)
}

pub fn bench_svm(c: &mut Criterion) {
    let mut group = c.benchmark_group("svm");
    group.measurement_time(Duration::from_secs(8));
    group.sample_size(50);

    let sizes = [(100, 4), (500, 10), (1000, 8)];
    let opts = Some(SvmOptions {
        c: 1.0,
        max_iters: 2000,
        tol: 1e-3,
    });

    for (n_samples, n_features) in sizes {
        let (data, labels) = synthetic_data_and_labels(n_samples, n_features);
        group.bench_with_input(
            BenchmarkId::new("linear", format!("{}x{}", n_samples, n_features)),
            &(data.clone(), labels.clone()),
            |b, (x, y)| b.iter(|| black_box(svm(black_box(x), black_box(y), opts.clone()))),
        );
    }

    let gamma = 0.5;
    for (n_samples, n_features) in sizes {
        let (data, labels) = synthetic_data_and_labels(n_samples, n_features);
        group.bench_with_input(
            BenchmarkId::new("rbf", format!("{}x{}", n_samples, n_features)),
            &(data.clone(), labels.clone()),
            |b, (x, y)| {
                b.iter(|| black_box(svm_rbf(black_box(x), black_box(y), gamma, opts.clone())))
            },
        );
    }

    group.finish();
}

criterion_group!(benches, bench_svm);
