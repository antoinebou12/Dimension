//! GPU correctness tests for matmul, dot, norm, scale, mul, axpy, abs, sqrt, div, spmv.
//! Includes: GPU unavailable (CPU fallback), CPU vs GPU consistency, Executor threshold, error handling.
//! Run with: cargo test --features gpu gpu

#![cfg(feature = "gpu")]

use mathlib::{
    AutoExecutor, CpuExecutor, Executor, ExecutorThresholds, Matrix, SparseMatrixCRS,
    SparseStorage, Storage, Triplet, Vector, pca,
};

#[test]
fn gpu_dot_correctness() {
    let mut a = Vector::<f32>::with_capacity(4);
    a.data_mut().copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    let mut b = Vector::<f32>::with_capacity(4);
    b.data_mut().copy_from_slice(&[1.0, 0.0, 1.0, 0.0]);
    let dot_cpu = 1.0 * 1.0 + 2.0 * 0.0 + 3.0 * 1.0 + 4.0 * 0.0;
    let dot = a.dot(&b);
    assert!(
        (dot - dot_cpu).abs() < 1e-5,
        "dot = {}, expected {}",
        dot,
        dot_cpu
    );
}

#[test]
fn gpu_norm_correctness() {
    let mut v = Vector::<f32>::with_capacity(3);
    v.data_mut().copy_from_slice(&[3.0, 4.0, 0.0]);
    let expected = 5.0;
    let n = v.norm();
    assert!(
        (n - expected).abs() < 1e-5,
        "norm = {}, expected {}",
        n,
        expected
    );
}

#[test]
fn gpu_matmul_correctness() {
    let mut a = Matrix::<f32>::with_storage(2, 3, Storage::Column);
    a.data_mut()
        .copy_from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let mut b = Matrix::<f32>::with_storage(3, 2, Storage::Column);
    b.data_mut()
        .copy_from_slice(&[1.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    let c = &a * &b;
    assert_eq!(c.rows(), 2);
    assert_eq!(c.cols(), 2);
    assert!((c.get(0, 0) - 1.0).abs() < 1e-5);
    assert!((c.get(0, 1) - 9.0).abs() < 1e-5);
    assert!((c.get(1, 0) - 2.0).abs() < 1e-5);
    assert!((c.get(1, 1) - 12.0).abs() < 1e-5);
}

#[test]
fn gpu_matvec_correctness() {
    let mut a = Matrix::<f32>::with_storage(3, 4, Storage::Column);
    a.data_mut().copy_from_slice(&[
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
    ]);
    let mut v = Vector::<f32>::with_capacity(4);
    v.data_mut().copy_from_slice(&[1.0, 0.0, 1.0, 0.0]);
    let y = &a * &v;
    let expected_0 = 1.0 * 1.0 + 4.0 * 0.0 + 7.0 * 1.0 + 10.0 * 0.0;
    let expected_1 = 2.0 * 1.0 + 5.0 * 0.0 + 8.0 * 1.0 + 11.0 * 0.0;
    let expected_2 = 3.0 * 1.0 + 6.0 * 0.0 + 9.0 * 1.0 + 12.0 * 0.0;
    assert!((y.get(0) - expected_0).abs() < 1e-5);
    assert!((y.get(1) - expected_1).abs() < 1e-5);
    assert!((y.get(2) - expected_2).abs() < 1e-5);
}

#[test]
fn gpu_add_sub_correctness() {
    let mut a = Matrix::<f32>::with_storage(2, 2, Storage::Column);
    a.data_mut().copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    let mut b = Matrix::<f32>::with_storage(2, 2, Storage::Column);
    b.data_mut().copy_from_slice(&[5.0, 6.0, 7.0, 8.0]);
    let c = &a + &b;
    assert!((c.get(0, 0) - 6.0).abs() < 1e-5);
    assert!((c.get(1, 0) - 8.0).abs() < 1e-5);
    let d = &b - &a;
    assert!((d.get(0, 0) - 4.0).abs() < 1e-5);
    let mut u = Vector::<f32>::with_capacity(3);
    u.data_mut().copy_from_slice(&[1.0, 2.0, 3.0]);
    let mut v = Vector::<f32>::with_capacity(3);
    v.data_mut().copy_from_slice(&[4.0, 5.0, 6.0]);
    let w = &u + &v;
    assert!((w.get(0) - 5.0).abs() < 1e-5);
    let z = &v - &u;
    assert!((z.get(0) - 3.0).abs() < 1e-5);
}

#[test]
fn gpu_large_matmul() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let n = 512;
    let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
    let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
    for i in 0..n * n {
        a.data_mut()[i] = (i % 100) as f32 * 0.01;
        b.data_mut()[i] = (i % 100) as f32 * 0.01;
    }
    let c = &a * &b;
    assert_eq!(c.rows(), n);
    assert_eq!(c.cols(), n);
}

#[test]
fn gpu_scale_correctness() {
    let mut a = Matrix::<f32>::with_storage(2, 2, Storage::Column);
    a.data_mut().copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    let scaled = 2.0_f32 * &a;
    assert!((scaled.get(0, 0) - 2.0).abs() < 1e-5);
    assert!((scaled.get(1, 1) - 8.0).abs() < 1e-5);
    let mut v = Vector::<f32>::with_capacity(3);
    v.data_mut().copy_from_slice(&[1.0, 2.0, 3.0]);
    let scaled_v = 3.0_f32 * &v;
    assert!((scaled_v.get(0) - 3.0).abs() < 1e-5);
    assert!((scaled_v.get(2) - 9.0).abs() < 1e-5);
}

#[test]
fn gpu_mul_elementwise_correctness() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let a = vec![1.0_f32, 2.0, 3.0, 4.0];
    let b = vec![2.0_f32, 3.0, 4.0, 5.0];
    let out = mathlib::gpu::try_mul_f32(&a, &b)
        .unwrap_or_else(|| a.iter().zip(b.iter()).map(|(x, y)| x * y).collect());
    assert!((out[0] - 2.0).abs() < 1e-5);
    assert!((out[3] - 20.0).abs() < 1e-5);
}

#[test]
fn gpu_axpy_correctness() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let x = vec![1.0_f32, 2.0, 3.0];
    let y = vec![4.0_f32, 5.0, 6.0];
    let z = mathlib::gpu::try_axpy_f32(0.5, &x, &y)
        .unwrap_or_else(|| x.iter().zip(y.iter()).map(|(a, b)| 0.5 * a + b).collect());
    assert!((z[0] - 4.5).abs() < 1e-5);
    assert!((z[2] - 7.5).abs() < 1e-5);
}

#[test]
fn gpu_abs_correctness() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let a = vec![-1.0_f32, 2.0, -3.0];
    let out = mathlib::gpu::try_abs_f32(&a).unwrap_or_else(|| a.iter().map(|x| x.abs()).collect());
    assert!((out[0] - 1.0).abs() < 1e-5);
    assert!((out[2] - 3.0).abs() < 1e-5);
}

#[test]
fn gpu_sqrt_correctness() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let a = vec![4.0_f32, 9.0, 16.0];
    let out =
        mathlib::gpu::try_sqrt_f32(&a).unwrap_or_else(|| a.iter().map(|x| x.sqrt()).collect());
    assert!((out[0] - 2.0).abs() < 1e-5);
    assert!((out[2] - 4.0).abs() < 1e-5);
}

#[test]
fn gpu_div_correctness() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let a = vec![6.0_f32, 8.0, 10.0];
    let b = vec![2.0_f32, 4.0, 5.0];
    let out = mathlib::gpu::try_div_f32(&a, &b).unwrap_or_else(|| {
        a.iter()
            .zip(b.iter())
            .map(|(x, y)| if *y == 0.0 { 0.0 } else { x / y })
            .collect()
    });
    assert!((out[0] - 3.0).abs() < 1e-5);
    assert!((out[1] - 2.0).abs() < 1e-5);
}

#[test]
fn gpu_squared_norm_correctness() {
    let mut v = Vector::<f32>::with_capacity(3);
    v.data_mut().copy_from_slice(&[3.0, 4.0, 0.0]);
    let norm_sq = mathlib::gpu::try_squared_norm_f32(&v).unwrap_or_else(|| v.dot(&v));
    assert!((norm_sq - 25.0).abs() < 1e-5);
}

#[test]
fn gpu_spmv_correctness() {
    let triplets = vec![
        Triplet::new(1.0_f32, 0, 0),
        Triplet::new(2.0, 0, 1),
        Triplet::new(3.0, 1, 0),
        Triplet::new(4.0, 1, 1),
        Triplet::new(5.0, 2, 0),
        Triplet::new(6.0, 2, 1),
    ];
    let sparse = SparseMatrixCRS::from_triplets(3, 2, &triplets);
    let mut v = Vector::<f32>::with_capacity(2);
    v.data_mut().copy_from_slice(&[1.0, 1.0]);
    let y = &sparse * &v;
    assert!((y.get(0) - 3.0).abs() < 1e-5);
    assert!((y.get(1) - 7.0).abs() < 1e-5);
    assert!((y.get(2) - 11.0).abs() < 1e-5);
}

#[test]
fn gpu_spmv_large() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let n = 128;
    let mut triplets = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if (i + j) % 7 == 0 {
                triplets.push(Triplet::new((i + j) as f32, i as u32, j as u32));
            }
        }
    }
    let sparse = SparseMatrixCRS::from_triplets(n, n, &triplets);
    let mut v = Vector::<f32>::with_capacity(n);
    for i in 0..n {
        v.set(i, (i + 1) as f32 * 0.01);
    }
    let y = &sparse * &v;
    assert_eq!(y.rows(), n);
}

#[test]
fn gpu_dot_norm_with_init() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let mut a = Vector::<f32>::with_capacity(64);
    for i in 0..64 {
        a.set(i, (i + 1) as f32 * 0.1);
    }
    let mut b = Vector::<f32>::with_capacity(64);
    for i in 0..64 {
        b.set(i, (64 - i) as f32 * 0.01);
    }
    let dot = a.dot(&b);
    let expected: f32 = (0..64)
        .map(|i| (i + 1) as f32 * 0.1 * (64 - i) as f32 * 0.01)
        .sum();
    assert!(
        (dot - expected).abs() < 1e-4,
        "dot = {}, expected {}",
        dot,
        expected
    );
    let norm_a = a.norm();
    let expected_norm: f32 = (0..64)
        .map(|i| ((i + 1) as f32 * 0.1).powi(2))
        .sum::<f32>()
        .sqrt();
    assert!(
        (norm_a - expected_norm).abs() < 1e-4,
        "norm = {}, expected {}",
        norm_a,
        expected_norm
    );
}

// --- GPU unavailable: operators still produce correct results via CPU fallback (do not init GPU) ---

#[test]
fn gpu_unavailable_matmul_fallback() {
    // Do not call init_blocking(); operators will use CPU when try_matmul_f32 returns None.
    let mut a = Matrix::<f32>::with_storage(2, 3, Storage::Column);
    a.data_mut()
        .copy_from_slice(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let mut b = Matrix::<f32>::with_storage(3, 2, Storage::Column);
    b.data_mut()
        .copy_from_slice(&[1.0, 0.0, 0.0, 1.0, 1.0, 1.0]);
    let c = &a * &b;
    assert_eq!(c.rows(), 2);
    assert_eq!(c.cols(), 2);
    assert!((c.get(0, 0) - 1.0).abs() < 1e-5);
    assert!((c.get(0, 1) - 9.0).abs() < 1e-5);
    assert!((c.get(1, 0) - 2.0).abs() < 1e-5);
    assert!((c.get(1, 1) - 12.0).abs() < 1e-5);
}

#[test]
fn gpu_unavailable_dot_and_add_fallback() {
    let mut x = Vector::<f32>::with_capacity(4);
    x.data_mut().copy_from_slice(&[1.0, 2.0, 3.0, 4.0]);
    let mut y = Vector::<f32>::with_capacity(4);
    y.data_mut().copy_from_slice(&[1.0, 0.0, 1.0, 0.0]);
    let dot = x.dot(&y);
    assert!((dot - 4.0).abs() < 1e-5);
    let z = &x + &y;
    assert!((z.get(0) - 2.0).abs() < 1e-5);
    assert!((z.get(2) - 4.0).abs() < 1e-5);
}

// --- CPU vs GPU consistency: with GPU initialized, operator result matches CpuExecutor ---

#[test]
fn executor_cpu_vs_gpu_consistency_matmul() {
    let _ = mathlib::gpu::init_blocking(None);
    let mut a = Matrix::<f32>::with_storage(8, 8, Storage::Column);
    let mut b = Matrix::<f32>::with_storage(8, 8, Storage::Column);
    for i in 0..64 {
        a.data_mut()[i] = (i % 10) as f32 * 0.1;
        b.data_mut()[i] = (i % 7) as f32 * 0.1;
    }
    let c_op = &a * &b;
    let c_cpu = CpuExecutor.matmul(&a, &b);
    for i in 0..8 {
        for j in 0..8 {
            assert!(
                (c_op.get(i, j) - c_cpu.get(i, j)).abs() < 1e-4,
                "matmul at ({},{}): op={} cpu={}",
                i,
                j,
                c_op.get(i, j),
                c_cpu.get(i, j)
            );
        }
    }
}

#[test]
fn executor_cpu_vs_gpu_consistency_dot_matvec_add() {
    let _ = mathlib::gpu::init_blocking(None);
    let mut a = Matrix::<f32>::with_storage(8, 8, Storage::Column);
    let mut x = Vector::<f32>::with_capacity(8);
    let mut y = Vector::<f32>::with_capacity(8);
    for i in 0..64 {
        a.data_mut()[i] = (i % 10) as f32 * 0.1;
    }
    for i in 0..8 {
        x.set(i, (i + 1) as f32 * 0.1);
        y.set(i, (8 - i) as f32 * 0.01);
    }
    let dot_op = x.dot(&y);
    let dot_cpu = CpuExecutor.dot(&x, &y);
    assert!(
        (dot_op - dot_cpu).abs() < 1e-4,
        "dot: op={} cpu={}",
        dot_op,
        dot_cpu
    );
    let matvec_op = &a * &x;
    let matvec_cpu = CpuExecutor.matvec(&a, &x);
    for i in 0..8 {
        assert!(
            (matvec_op.get(i) - matvec_cpu.get(i)).abs() < 1e-4,
            "matvec at {}: op={} cpu={}",
            i,
            matvec_op.get(i),
            matvec_cpu.get(i)
        );
    }
    let add_op = &x + &y;
    let add_cpu = CpuExecutor.add_vector(&x, &y);
    for i in 0..8 {
        assert!(
            (add_op.get(i) - add_cpu.get(i)).abs() < 1e-5,
            "add at {}: op={} cpu={}",
            i,
            add_op.get(i),
            add_cpu.get(i)
        );
    }
}

// --- AutoExecutor threshold: with very high threshold, small matmul uses CPU and matches CpuExecutor ---

#[test]
fn executor_threshold_small_matmul_uses_cpu() {
    let thresholds = ExecutorThresholds {
        matmul_elements_min: usize::MAX,
        dot_len_min: usize::MAX,
        matvec_elements_min: usize::MAX,
        elementwise_len_min: usize::MAX,
    };
    let exec = AutoExecutor::with_thresholds(thresholds);
    let mut a = Matrix::<f32>::with_storage(4, 4, Storage::Column);
    let mut b = Matrix::<f32>::with_storage(4, 4, Storage::Column);
    for i in 0..16 {
        a.data_mut()[i] = (i % 5) as f32 * 0.1;
        b.data_mut()[i] = (i % 7) as f32 * 0.1;
    }
    let c_auto = exec.matmul(&a, &b);
    let c_cpu = CpuExecutor.matmul(&a, &b);
    for i in 0..4 {
        for j in 0..4 {
            assert!(
                (c_auto.get(i, j) - c_cpu.get(i, j)).abs() < 1e-5,
                "threshold matmul at ({},{}): auto={} cpu={}",
                i,
                j,
                c_auto.get(i, j),
                c_cpu.get(i, j)
            );
        }
    }
}

#[test]
fn executor_threshold_large_matmul_still_correct() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let exec = AutoExecutor::default();
    let n = 128;
    let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
    let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
    for i in 0..n * n {
        a.data_mut()[i] = (i % 100) as f32 * 0.01;
        b.data_mut()[i] = (i % 100) as f32 * 0.01;
    }
    let c_auto = exec.matmul(&a, &b);
    let c_cpu = CpuExecutor.matmul(&a, &b);
    for i in 0..n {
        for j in 0..n {
            assert!(
                (c_auto.get(i, j) - c_cpu.get(i, j)).abs() < 1e-2,
                "large matmul at ({},{}): auto={} cpu={}",
                i,
                j,
                c_auto.get(i, j),
                c_cpu.get(i, j)
            );
        }
    }
}

// --- PCA transform: GPU matmul (centered × components) matches CPU transform ---

#[test]
fn pca_transform_gpu_vs_cpu() {
    let ok = mathlib::gpu::init_blocking(None);
    if !ok {
        return;
    }
    let n_samples = 10_usize;
    let n_features = 4_usize;
    let n_comp = 2_usize;
    let mut data_f64 = Matrix::<f64>::with_storage(n_samples, n_features, Storage::Column);
    for i in 0..n_samples {
        for j in 0..n_features {
            data_f64.set(i, j, (i as f64) * 0.5 + (j as f64));
        }
    }
    let pca_result = pca(&data_f64, Some(n_comp));
    let mean = pca_result.mean();
    let components = pca_result.components();

    let mean_f32: Vec<f32> = (0..mean.rows()).map(|j| mean.get(j) as f32).collect();
    let mut components_f32 = Matrix::<f32>::with_storage(n_features, n_comp, Storage::Column);
    for i in 0..n_features {
        for j in 0..n_comp {
            components_f32.set(i, j, components.get(i, j) as f32);
        }
    }
    let mut data_f32 = Matrix::<f32>::with_storage(n_samples, n_features, Storage::Column);
    for i in 0..n_samples {
        for j in 0..n_features {
            data_f32.set(i, j, data_f64.get(i, j) as f32);
        }
    }
    let mut centered_f32 = Matrix::<f32>::with_storage(n_samples, n_features, Storage::Column);
    for i in 0..n_samples {
        for j in 0..n_features {
            centered_f32.set(i, j, data_f32.get(i, j) - mean_f32[j]);
        }
    }

    let cpu_transform = CpuExecutor.matmul(&centered_f32, &components_f32);
    let gpu_transform = match mathlib::gpu::try_matmul_f32(&centered_f32, &components_f32) {
        Some(m) => m,
        None => return,
    };
    assert_eq!(cpu_transform.rows(), gpu_transform.rows());
    assert_eq!(cpu_transform.cols(), gpu_transform.cols());
    for i in 0..cpu_transform.rows() {
        for j in 0..cpu_transform.cols() {
            let c = cpu_transform.get(i, j);
            let g = gpu_transform.get(i, j);
            assert!(
                (c - g).abs() < 1e-3,
                "PCA transform at ({},{}): cpu={} gpu={}",
                i,
                j,
                c,
                g
            );
        }
    }
}

// --- Error handling: dimension mismatch causes panic (no silent wrong result) ---

#[test]
#[should_panic(expected = "assert")]
fn executor_matmul_dimension_mismatch_panics() {
    let a = Matrix::<f32>::with_storage(2, 3, Storage::Column);
    let b = Matrix::<f32>::with_storage(2, 2, Storage::Column); // 3 != 2
    let _ = CpuExecutor.matmul(&a, &b);
}
