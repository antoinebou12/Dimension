//! Example: Wasm API (WasmMatrix, WasmVector, solve, SVD, WasmMatrix32, WasmCg).
//! Run with: cargo run --example wasm --features wasm
//!
//! Demonstrates the same API that JavaScript would use after `wasm-pack build --target web --features wasm`.
//! Other wasm examples (mirroring the browser demo): `wasm_clustering`, `wasm_graph`, `wasm_noise`, `wasm_optim`.

#[cfg(not(feature = "wasm"))]
fn main() {
    eprintln!("Build with: cargo run --example wasm --features wasm");
}

#[cfg(feature = "wasm")]
fn main() {
    use mathlib::wasm::{WasmCg, WasmMatrix, WasmMatrix32, WasmVector};

    // --- WasmMatrix / WasmVector ---
    let mut m = WasmMatrix::new(2, 2);
    m.set(0, 0, 1.0);
    m.set(0, 1, 2.0);
    m.set(1, 0, 3.0);
    m.set(1, 1, 4.0);
    println!("WasmMatrix 2x2 to_array: {:?}", m.to_array());

    let v = WasmVector::from_array(&[1.0, 2.0]);
    let mv = m.mul_vector(&v).unwrap();
    println!("M * v = {:?}", mv.to_array());

    // --- Solve Ax = b ---
    let b = WasmVector::from_array(&[5.0, 11.0]);
    let x = m.solve(&b).unwrap();
    println!("Solve Ax = b, x = {:?}", x.to_array());

    // --- SVD ---
    let svd = m.svd_econ();
    println!("SVD sigma = {:?}", svd.get_sigma().to_array());

    // --- Vector lerp and distance ---
    let a = WasmVector::from_array(&[0.0, 0.0]);
    let b_vec = WasmVector::from_array(&[3.0, 4.0]);
    let mid = a.lerp(&b_vec, 0.5).unwrap();
    println!("lerp(a, b, 0.5) = {:?}", mid.to_array());
    println!(
        "euclidean_distance(a, b) = {}",
        a.euclidean_distance(&b_vec).unwrap()
    );

    // --- WasmMatrix32: 3D rotation and transform ---
    let rot = WasmMatrix32::rotation(0.0, 0.0, std::f32::consts::FRAC_PI_2);
    let pt = rot.transform_point(1.0, 0.0, 0.0).unwrap();
    println!("rotation(z=π/2) * (1,0,0) = {:?}", pt);

    // --- WasmCg: camera and projection ---
    let view = WasmCg::look_at_rh(0.0, 0.0, 5.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0);
    let _proj = WasmCg::new_perspective(16.0 / 9.0, std::f32::consts::FRAC_PI_4, 0.1, 100.0);
    let _view_inv = view.inverse().unwrap();
    println!("view 4x4 inverse ok; proj 4x4 ok");
}
