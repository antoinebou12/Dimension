//! Integration tests for wasm bindings (demos: vector, matrix, k-means, PCA, SVM,
//! distance, Cholesky, SVD, simplex, camera, DBSCAN, LU, graph, PSO, noise, transforms, argmin, gpu).
//! Run with: cargo test --features wasm wasm
//! Run GPU tests: cargo test --features "wasm gpu" wasm

#![cfg(feature = "wasm")]

mod argmin;
mod camera;
mod clustering_decomposition;
mod distance;
#[cfg(feature = "gpu")]
mod gpu;
mod graph;
mod matrix;
mod noise;
mod pso;
mod simplex;
mod solve;
mod svm;
mod transforms;
mod vector;
