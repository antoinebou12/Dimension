//! Main benchmark harness for mathlib.
//!
//! **Linear algebra**: construction, matvec, access, scaling, lu, chol, solve, qz, schur, pca.
//! **Math / CG**: math3d, quaternion_trig (quaternion + trig), easing, cg, clustering, svm (linear, rbf), distance, graph (dijkstra, astar, dstar), pso.

#![allow(clippy::duplicate_mod)]

use criterion::criterion_main;

#[cfg(feature = "genetic")]
#[path = "cmaes.rs"]
mod cmaes;

#[path = "access.rs"]
mod access;
#[path = "cg.rs"]
mod cg;
#[path = "chol.rs"]
mod chol;
#[path = "clustering.rs"]
mod clustering;
#[path = "common.rs"]
mod common;
#[path = "argmin/conjugate_gradient.rs"]
mod conjugate_gradient;
#[path = "construction.rs"]
mod construction;
#[path = "distance.rs"]
mod distance;
#[path = "easing.rs"]
mod easing;
#[path = "argmin/gauss_newton.rs"]
mod gauss_newton;
#[path = "argmin/gradient_descent.rs"]
mod gradient_descent;
#[path = "graph.rs"]
mod graph;
#[path = "argmin/linesearch.rs"]
mod linesearch;
#[path = "lu.rs"]
mod lu;
#[path = "math3d.rs"]
mod math3d;
#[path = "matvec.rs"]
mod matvec;
#[path = "argmin/muon.rs"]
mod muon;
#[path = "decomposition/pca.rs"]
mod pca;
#[path = "argmin/pso.rs"]
mod pso;
#[path = "quaternion_trig.rs"]
mod quaternion_trig;
#[path = "qz.rs"]
mod qz;
#[path = "scaling.rs"]
mod scaling;
#[path = "schur.rs"]
mod schur;
#[path = "solve.rs"]
mod solve;
#[path = "svm.rs"]
mod svm;

#[cfg(not(feature = "genetic"))]
criterion_main!(
    construction::benches,
    matvec::benches,
    access::benches,
    scaling::benches,
    lu::benches,
    chol::benches,
    solve::benches,
    qz::benches,
    schur::benches,
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
    graph::benches,
    distance::benches,
    easing::benches,
    quaternion_trig::benches,
    svm::benches
);

#[cfg(feature = "genetic")]
criterion_main!(
    construction::benches,
    matvec::benches,
    access::benches,
    scaling::benches,
    lu::benches,
    chol::benches,
    solve::benches,
    qz::benches,
    schur::benches,
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
    cmaes::benches,
    graph::benches,
    distance::benches,
    easing::benches,
    quaternion_trig::benches,
    svm::benches
);
