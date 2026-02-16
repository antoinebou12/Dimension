//! Main benchmark harness for mathlib.
//!
//! Domain-organized: linear, ml, optimisation, graph, tree, cg, noise, transforms.
//!
//! GPU vs CPU benchmarks (matmul, matvec, dot, etc.) and threshold guidance:
//! `cargo bench --features gpu --bench gpu`.

#![allow(clippy::duplicate_mod)]

use criterion::criterion_main;

#[path = "common.rs"]
mod common;

// linear
#[path = "linear/access.rs"]
mod access;
#[path = "linear/chol.rs"]
mod chol;
#[path = "linear/construction.rs"]
mod construction;
#[path = "linear/lu.rs"]
mod lu;
#[path = "linear/matvec.rs"]
mod matvec;
#[path = "linear/pca.rs"]
mod pca;
#[path = "linear/qz.rs"]
mod qz;
#[path = "linear/scaling.rs"]
mod scaling;
#[path = "linear/schur.rs"]
mod schur;
#[path = "linear/simplex.rs"]
mod simplex;
#[path = "linear/solve.rs"]
mod solve;
#[path = "linear/submatrix.rs"]
mod submatrix;

// ml
#[path = "ml/clustering.rs"]
mod clustering;
#[path = "ml/distance.rs"]
mod distance;
#[path = "ml/svm.rs"]
mod svm;

// optimisation
#[path = "optimisation/argmin/conjugate_gradient.rs"]
mod conjugate_gradient;
#[path = "optimisation/argmin/gauss_newton.rs"]
mod gauss_newton;
#[path = "optimisation/argmin/gradient_descent.rs"]
mod gradient_descent;
#[path = "optimisation/argmin/linesearch.rs"]
mod linesearch;
#[path = "optimisation/argmin/muon.rs"]
mod muon;
#[path = "optimisation/argmin/pso.rs"]
mod pso;

#[cfg(feature = "genetic")]
#[path = "optimisation/cmaes.rs"]
mod cmaes;

// cg
#[path = "cg/cg.rs"]
mod cg;
#[path = "cg/curve.rs"]
mod curve;
#[path = "cg/dual_quaternion.rs"]
mod dual_quaternion;
#[path = "cg/easing.rs"]
mod easing;
#[path = "cg/math3d.rs"]
mod math3d;
#[path = "cg/quaternion_trig.rs"]
mod quaternion_trig;

// noise
#[path = "noise/noise.rs"]
mod noise;

// monte_carlo
#[path = "monte_carlo/monte_carlo.rs"]
mod monte_carlo;

mod graph;
mod tree;

#[path = "transforms/convolution.rs"]
mod transforms_convolution;
#[path = "transforms/dct.rs"]
mod transforms_dct;
#[path = "transforms/fft.rs"]
mod transforms_fft;
#[path = "transforms/wavelets.rs"]
mod transforms_wavelets;
#[path = "transforms/windows.rs"]
mod transforms_windows;

#[cfg(not(feature = "genetic"))]
criterion_main!(
    construction::benches,
    matvec::benches,
    access::benches,
    scaling::benches,
    simplex::benches,
    lu::benches,
    chol::benches,
    solve::benches,
    qz::benches,
    schur::benches,
    submatrix::benches,
    pca::benches,
    math3d::benches,
    cg::benches,
    clustering::benches,
    pso::benches,
    linesearch::benches,
    gradient_descent::benches,
    conjugate_gradient::benches,
    gauss_newton::benches,
    muon::benches,
    noise::benches,
    monte_carlo::benches,
    graph::benches,
    tree::benches,
    distance::benches,
    easing::benches,
    curve::curve_benches,
    dual_quaternion::benches,
    quaternion_trig::benches,
    svm::benches,
    transforms_fft::benches,
    transforms_dct::benches,
    transforms_wavelets::benches,
    transforms_convolution::benches,
    transforms_windows::benches
);

#[cfg(feature = "genetic")]
criterion_main!(
    construction::benches,
    matvec::benches,
    access::benches,
    scaling::benches,
    simplex::benches,
    lu::benches,
    chol::benches,
    solve::benches,
    qz::benches,
    schur::benches,
    submatrix::benches,
    pca::benches,
    math3d::benches,
    cg::benches,
    clustering::benches,
    pso::benches,
    linesearch::benches,
    gradient_descent::benches,
    conjugate_gradient::benches,
    gauss_newton::benches,
    muon::benches,
    noise::benches,
    cmaes::benches,
    graph::benches,
    tree::benches,
    distance::benches,
    easing::benches,
    curve::curve_benches,
    dual_quaternion::benches,
    quaternion_trig::benches,
    svm::benches,
    transforms_fft::benches,
    transforms_dct::benches,
    transforms_wavelets::benches,
    transforms_convolution::benches,
    transforms_windows::benches
);
