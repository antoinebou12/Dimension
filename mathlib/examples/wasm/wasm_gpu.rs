//! Example: GPU init and async ops (matmul, dot, matvec).
//!
//! Run natively: cargo run --example wasm_gpu --features "wasm gpu"
//!
//! This demonstrates the same API used from JavaScript: initGpuAsync, matmulF32GpuAsync,
//! dotF32GpuAsync, matvecF32GpuAsync. On native we use Forte's thread pool to block_on the async code.

use mathlib::{Matrix, Storage, Vector};

static FORTE_POOL: forte::ThreadPool = forte::ThreadPool::new();

fn main() {
    println!("WASM GPU example — async init and ops");
    println!(
        "  Same API as JS: initGpuAsync, matmulF32GpuAsync, dotF32GpuAsync, matvecF32GpuAsync\n"
    );

    FORTE_POOL.populate();
    let ok = FORTE_POOL.block_on(mathlib::gpu::init_async(None));
    if !ok {
        eprintln!("GPU init failed. On wasm, ensure HTTPS/localhost and WebGPU enabled.");
        return;
    }
    println!("GPU initialized.");

    // Matmul
    let mut a = Matrix::<f32>::with_storage(4, 4, Storage::Column);
    for i in 0..16 {
        a.data_mut()[i] = (i as f32) * 0.1;
    }
    let mut b = Matrix::<f32>::with_storage(4, 4, Storage::Column);
    for i in 0..16 {
        b.data_mut()[i] = if i % 5 == 0 { 1.0 } else { 0.0 };
    }

    match FORTE_POOL.block_on(mathlib::gpu::try_matmul_f32_async(&a, &b)) {
        Some(c) => println!("Matmul 4x4: C[0,0] = {}", c.get(0, 0)),
        None => println!("Matmul: GPU unavailable or failed"),
    }

    // Dot
    let u = Vector::<f32>::from_slice(&[1.0, 2.0, 3.0]);
    let v = Vector::<f32>::from_slice(&[4.0, 5.0, 6.0]);
    match FORTE_POOL.block_on(mathlib::gpu::try_dot_f32_async(&u, &v)) {
        Some(d) => println!("Dot [1,2,3]·[4,5,6] = {}", d),
        None => println!("Dot: GPU unavailable or failed"),
    }

    // Matvec
    let mut m = Matrix::<f32>::with_storage(3, 2, Storage::Column);
    m.data_mut()
        .copy_from_slice(&[1.0, 0.0, 0.0, 0.0, 1.0, 0.0]);
    let x = Vector::<f32>::from_slice(&[1.0, 2.0]);
    match FORTE_POOL.block_on(mathlib::gpu::try_matvec_f32_async(&m, &x)) {
        Some(y) => println!(
            "Matvec 3x2 × [1,2] = [{}, {}, {}]",
            y.get(0),
            y.get(1),
            y.get(2)
        ),
        None => println!("Matvec: GPU unavailable or failed"),
    }

    println!("\nDone.");
}
