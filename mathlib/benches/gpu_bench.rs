//! GPU vs CPU benchmarks for matmul, dot, norm, scale, mul, axpy, abs, sqrt, div, spmv.
//!
//! Run with: `cargo bench --features gpu --bench gpu`
//!
//! # Threshold guidance
//!
//! - **gpu_matmul** and **gpu_matvec** measure the full pipeline: upload → dispatch → readback.
//!   Compare with **cpu_matmul** / **cpu_matvec** for the same sizes (64, 128, 256, 512, 1024).
//! - GPU typically wins when M*K*N (matmul) or rows*cols (matvec) is above ~2M; tune
//!   `ExecutorThresholds::matmul_elements_min` and `matvec_elements_min` from your report.
//! - For "dispatch only" (tensors already on GPU), use persistent GPU buffers (future GpuTensor2D).

#![cfg(feature = "gpu")]

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use mathlib::{Matrix, SparseMatrixCRS, SparseStorage, Storage, Triplet, Vector};

fn generate_sparse_f32(n: usize, density: f64) -> Vec<Triplet<f32>> {
    let mut triplets = Vec::new();
    let nnz = ((n * n) as f64 * density).max(1.0) as usize;
    for k in 0..nnz {
        let i = (k * 7) % n;
        let j = (k * 13) % n;
        let val = (k as f32 + 1.0) * 0.1;
        triplets.push(Triplet::new(val, i as u32, j as u32));
    }
    triplets
}

fn bench_gpu_matmul_dot_norm(c: &mut Criterion) {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        eprintln!("GPU init failed, skipping GPU benchmarks");
        return;
    }

    let mut group = c.benchmark_group("gpu_matmul");
    group.sample_size(20);
    for n in [64, 128, 256, 512, 1024] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            b.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        group.bench_with_input(BenchmarkId::new("f32_matmul", n), &(&a, &b), |b, (x, y)| {
            b.iter(|| black_box(&**x) * black_box(&**y))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_dot");
    for n in [64, 256, 1024, 4096, 16384] {
        let mut a = Vector::<f32>::with_capacity(n);
        let mut b = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            a.set(i, (i % 100) as f32 * 0.01);
            b.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_dot", n), &(&a, &b), |b, (x, y)| {
            b.iter(|| black_box(&**x).dot(black_box(&**y)))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_norm");
    for n in [64, 256, 1024, 4096, 16384] {
        let mut v = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            v.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_norm", n), &v, |b, x| {
            b.iter(|| black_box(x).norm())
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_matvec");
    for n in [64, 128, 256, 512, 1024] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        let mut v = Vector::<f32>::with_capacity(n);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        for i in 0..n {
            v.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_matvec", n), &(&a, &v), |b, (x, y)| {
            b.iter(|| black_box(&**x) * black_box(&**y))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_add");
    for n in [256, 512, 1024] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01;
            b.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        group.bench_with_input(BenchmarkId::new("f32_add", n), &(&a, &b), |b, (x, y)| {
            b.iter(|| black_box(&**x) + black_box(&**y))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_scale");
    for n in [256, 1024, 4096, 16384] {
        let mut v = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            v.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_scale_vec", n), &v, |b, x| {
            b.iter(|| 2.0_f32 * black_box(x))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_mul");
    for n in [256, 1024, 4096] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01;
            b.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        let (ad, bd) = (a.data().to_vec(), b.data().to_vec());
        group.bench_with_input(BenchmarkId::new("f32_mul", n), &(ad, bd), |b, (x, y)| {
            b.iter(|| mathlib::gpu::try_mul_f32(black_box(x), black_box(y)))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_axpy");
    for n in [256, 1024, 4096, 16384] {
        let mut x = Vector::<f32>::with_capacity(n);
        let mut y = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            x.set(i, (i % 100) as f32 * 0.01);
            y.set(i, (n - i) as f32 * 0.01);
        }
        let (xd, yd) = (x.data().to_vec(), y.data().to_vec());
        group.bench_with_input(BenchmarkId::new("f32_axpy", n), &(xd, yd), |b, (xv, yv)| {
            b.iter(|| mathlib::gpu::try_axpy_f32(0.5, black_box(xv), black_box(yv)))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_abs");
    for n in [256, 1024, 4096] {
        let mut a = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            a.set(i, ((i as i32 % 100) - 50) as f32 * 0.01);
        }
        let ad = a.data().to_vec();
        group.bench_with_input(BenchmarkId::new("f32_abs", n), &ad, |b, x| {
            b.iter(|| mathlib::gpu::try_abs_f32(black_box(x)))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_sqrt");
    for n in [256, 1024, 4096] {
        let mut a = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            a.set(i, (i % 100) as f32 * 0.01 + 0.01);
        }
        let ad = a.data().to_vec();
        group.bench_with_input(BenchmarkId::new("f32_sqrt", n), &ad, |b, x| {
            b.iter(|| mathlib::gpu::try_sqrt_f32(black_box(x)))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_div");
    for n in [256, 1024] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01 + 0.01;
            b.data_mut()[i] = (i % 100) as f32 * 0.01 + 0.01;
        }
        let (ad, bd) = (a.data().to_vec(), b.data().to_vec());
        group.bench_with_input(BenchmarkId::new("f32_div", n), &(ad, bd), |b, (x, y)| {
            b.iter(|| mathlib::gpu::try_div_f32(black_box(x), black_box(y)))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("gpu_spmv");
    for n in [512, 1024, 2048] {
        let triplets = generate_sparse_f32(n, 0.01);
        let sparse = SparseMatrixCRS::from_triplets(n, n, &triplets);
        let mut v = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            v.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(
            BenchmarkId::new("f32_spmv", n),
            &(&sparse, &v),
            |b, (s, x)| b.iter(|| black_box(&**s) * black_box(&**x)),
        );
    }
    group.finish();
}

fn bench_cpu_matmul_dot_norm(c: &mut Criterion) {
    let mut group = c.benchmark_group("cpu_matmul");
    for n in [64, 128, 256, 512, 1024] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            b.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        group.bench_with_input(BenchmarkId::new("f32_matmul", n), &(&a, &b), |b, (x, y)| {
            b.iter(|| black_box(&**x) * black_box(&**y))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_matvec");
    for n in [64, 128, 256, 512, 1024] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        let mut v = Vector::<f32>::with_capacity(n);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        for i in 0..n {
            v.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_matvec", n), &(&a, &v), |b, (x, y)| {
            b.iter(|| black_box(&**x) * black_box(&**y))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_add");
    for n in [256, 512, 1024] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01;
            b.data_mut()[i] = (i % 100) as f32 * 0.01;
        }
        group.bench_with_input(BenchmarkId::new("f32_add", n), &(&a, &b), |b, (x, y)| {
            b.iter(|| black_box(&**x) + black_box(&**y))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_dot");
    for n in [64, 256, 1024, 4096, 16384] {
        let mut a = Vector::<f32>::with_capacity(n);
        let mut b = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            a.set(i, (i % 100) as f32 * 0.01);
            b.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_dot", n), &(&a, &b), |b, (x, y)| {
            b.iter(|| black_box(&**x).dot(black_box(&**y)))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_norm");
    for n in [64, 256, 1024, 4096, 16384] {
        let mut v = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            v.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_norm", n), &v, |b, x| {
            b.iter(|| black_box(x).norm())
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_scale");
    for n in [256, 1024, 4096, 16384] {
        let mut v = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            v.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_scale_vec", n), &v, |b, x| {
            b.iter(|| 2.0_f32 * black_box(x))
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_mul");
    for n in [256, 1024, 4096] {
        let total = n * n;
        let a: Vec<f32> = (0..total).map(|i| (i % 100) as f32 * 0.01).collect();
        let b: Vec<f32> = (0..total).map(|i| (i % 100) as f32 * 0.01).collect();
        group.bench_with_input(BenchmarkId::new("f32_mul", n), &(a, b), |b, (x, y)| {
            b.iter(|| {
                x.iter()
                    .zip(y.iter())
                    .map(|(a, b)| a * b)
                    .collect::<Vec<_>>()
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_axpy");
    for n in [256, 1024, 4096, 16384] {
        let mut x = Vector::<f32>::with_capacity(n);
        let mut y = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            x.set(i, (i % 100) as f32 * 0.01);
            y.set(i, (n - i) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_axpy", n), &(&x, &y), |b, (xv, yv)| {
            b.iter(|| {
                let mut z = Vector::with_capacity(xv.rows());
                for i in 0..xv.rows() {
                    z.set(i, 0.5 * xv.get(i) + yv.get(i));
                }
                z
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_abs");
    for n in [256, 1024, 4096] {
        let mut a = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            a.set(i, ((i as i32 % 100) - 50) as f32 * 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_abs", n), &a, |b, x| {
            b.iter(|| {
                let mut out = Vector::with_capacity(x.rows());
                for i in 0..x.rows() {
                    out.set(i, x.get(i).abs());
                }
                out
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_sqrt");
    for n in [256, 1024, 4096] {
        let mut a = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            a.set(i, (i % 100) as f32 * 0.01 + 0.01);
        }
        group.bench_with_input(BenchmarkId::new("f32_sqrt", n), &a, |b, x| {
            b.iter(|| {
                let mut out = Vector::with_capacity(x.rows());
                for i in 0..x.rows() {
                    out.set(i, x.get(i).sqrt());
                }
                out
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_div");
    for n in [256, 1024] {
        let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
        let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
        for i in 0..n * n {
            a.data_mut()[i] = (i % 100) as f32 * 0.01 + 0.01;
            b.data_mut()[i] = (i % 100) as f32 * 0.01 + 0.01;
        }
        group.bench_with_input(BenchmarkId::new("f32_div", n), &(&a, &b), |b, (x, y)| {
            b.iter(|| {
                let mut out = Matrix::with_storage(x.rows(), x.cols(), Storage::Column);
                for i in 0..x.data().len() {
                    let den = y.data()[i];
                    out.data_mut()[i] = if den == 0.0 { 0.0 } else { x.data()[i] / den };
                }
                out
            })
        });
    }
    group.finish();

    let mut group = c.benchmark_group("cpu_spmv");
    for n in [512, 1024, 2048] {
        let triplets = generate_sparse_f32(n, 0.01);
        let sparse = SparseMatrixCRS::from_triplets(n, n, &triplets);
        let mut v = Vector::<f32>::with_capacity(n);
        for i in 0..n {
            v.set(i, (i % 100) as f32 * 0.01);
        }
        group.bench_with_input(
            BenchmarkId::new("f32_spmv", n),
            &(&sparse, &v),
            |b, (s, x)| b.iter(|| black_box(&**s) * black_box(&**x)),
        );
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_cpu_matmul_dot_norm,
    bench_gpu_matmul_dot_norm
);
criterion_main!(benches);
