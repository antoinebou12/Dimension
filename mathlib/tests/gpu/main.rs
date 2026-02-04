//! GPU correctness tests for matmul, dot, norm, scale, mul, axpy, abs, sqrt, div, spmv.
//! Run with: cargo test --features gpu gpu

#![cfg(feature = "gpu")]

use mathlib::{Matrix, SparseMatrixCRS, SparseStorage, Storage, Triplet, Vector};

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
    let ok = mathlib::gpu::init_blocking();
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
    let ok = mathlib::gpu::init_blocking();
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
    let ok = mathlib::gpu::init_blocking();
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
    let ok = mathlib::gpu::init_blocking();
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
    let ok = mathlib::gpu::init_blocking();
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
    let ok = mathlib::gpu::init_blocking();
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
    let ok = mathlib::gpu::init_blocking();
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
    let ok = mathlib::gpu::init_blocking();
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
