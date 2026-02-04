//! Integration tests for wasm bindings (demos: vector, matrix, k-means, PCA, SVM,
//! distance, Cholesky, SVD, simplex, camera, DBSCAN, LU, graph, PSO, noise, transforms, argmin).
//! Run with: cargo test --features wasm wasm

#![cfg(feature = "wasm")]

mod argmin;
mod camera;
mod clustering_decomposition;
mod distance;
mod graph;
mod matrix;
mod noise;
mod pso;
mod simplex;
mod svm;
mod transforms;
mod vector;
