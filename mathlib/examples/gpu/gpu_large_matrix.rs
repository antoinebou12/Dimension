//! Example: large f32 matrix multiply, dot, and norm with GPU acceleration.
//!
//! Run with: cargo run --example gpu_large_matrix --features gpu
//!
//! Size can be overridden via env: MATHLIB_GPU_SIZE=512 (default 1024).

use mathlib::{Matrix, Storage, Vector};
use std::time::Instant;

fn main() {
    let n: usize = std::env::var("MATHLIB_GPU_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024);

    println!("GPU large matrix example (size = {})", n);
    println!("  Set MATHLIB_GPU_SIZE to override (e.g. 512, 2048).");

    let ok = mathlib::gpu::init_blocking();
    if !ok {
        println!("\nGPU init failed — no WebGPU/Vulkan/Metal/D3D adapter. Using CPU fallback.");
    } else {
        println!("\nGPU initialized.");
    }

    // Matrix multiply
    let mut a = Matrix::<f32>::with_storage(n, n, Storage::Column);
    let mut b = Matrix::<f32>::with_storage(n, n, Storage::Column);
    for i in 0..n * n {
        a.data_mut()[i] = (i % 100) as f32 * 0.01;
        b.data_mut()[i] = (i % 100) as f32 * 0.01;
    }
    let t0 = Instant::now();
    let c = &a * &b;
    let elapsed = t0.elapsed();
    println!(
        "\nMatmul {}x{} x {}x{}: {:?} (sample C[0,0] = {})",
        n,
        n,
        n,
        n,
        elapsed,
        c.get(0, 0)
    );

    // Vector dot and norm
    let mut u = Vector::<f32>::with_capacity(n);
    let mut v = Vector::<f32>::with_capacity(n);
    for i in 0..n {
        u.set(i, (i % 100) as f32 * 0.01);
        v.set(i, (n - i) as f32 * 0.01);
    }
    let t1 = Instant::now();
    let dot = u.dot(&v);
    let t2 = Instant::now();
    let norm_u = u.norm();
    let t3 = Instant::now();
    println!("Dot product (len {}): {:?}, result = {}", n, t2 - t1, dot);
    println!("Norm (len {}): {:?}, result = {}", n, t3 - t2, norm_u);

    println!("\nDone.");
}
