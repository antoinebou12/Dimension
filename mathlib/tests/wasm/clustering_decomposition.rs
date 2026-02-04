//! Integration tests for wasm K-means, PCA, Cholesky, DBSCAN, LU bindings.
//! Run with: cargo test --features wasm wasm_clustering_decomposition

#![cfg(feature = "wasm")]

use mathlib::wasm::{
    WasmCholesky, WasmDbscan, WasmKmeans, WasmLu, WasmMatrix, WasmPca, WasmVector,
};

#[test]
fn wasm_demo_kmeans() {
    // Demo: 6 points in 2D, k = 2; labels [0,0,0,1,1,1], centroids ~(1,0) and (11,10)
    let data = vec![
        0.0, 1.0, 2.0, 10.0, 11.0, 12.0, 0.0, 0.0, 0.0, 10.0, 10.0, 10.0,
    ];
    let m = WasmMatrix::from_array(6, 2, &data).unwrap();
    let km = WasmKmeans::new(&m, 2, 100).unwrap();
    let labels = km.get_labels();
    assert_eq!(labels.len(), 6);
    assert_eq!(labels[0], labels[1]);
    assert_eq!(labels[1], labels[2]);
    assert_eq!(labels[3], labels[4]);
    assert_eq!(labels[4], labels[5]);
    assert_ne!(labels[0], labels[3]);
    let centroids = km.get_centroids();
    assert_eq!(centroids.rows(), 2);
    assert_eq!(centroids.cols(), 2);
    assert!((centroids.get(0, 0) - 1.0).abs() < 2.0);
    assert!((centroids.get(1, 0) - 11.0).abs() < 2.0);
}

#[test]
fn wasm_demo_pca() {
    // Demo: 10×4 data, 2 components; mean, explained variance, transform
    let mut data = Vec::with_capacity(40);
    for i in 0..10 {
        for j in 0..4 {
            data.push(i as f64 * 0.5 + j as f64);
        }
    }
    let mat = WasmMatrix::from_array(10, 4, &data).unwrap();
    let pca = WasmPca::new(&mat, 2);
    let mean = pca.get_mean().to_array();
    assert_eq!(mean.len(), 4);
    let ev = pca.get_explained_variance().to_array();
    assert_eq!(ev.len(), 2);
    assert!(ev[0] >= ev[1]);
    let proj = pca.transform(&mat).unwrap();
    assert_eq!(proj.rows(), 10);
    assert_eq!(proj.cols(), 2);
}

#[test]
fn wasm_demo_cholesky() {
    // Demo: A 2×2 SPD [[4,2],[2,3]], b = [6, 5], solve Ax = b -> x = [1, 1]
    let a = WasmMatrix::from_array(2, 2, &[4.0, 2.0, 2.0, 3.0]).unwrap();
    let b = WasmVector::from_array(&[6.0, 5.0]);
    let chol = WasmCholesky::new(&a).unwrap();
    let l = chol.get_l();
    assert!((l.get(0, 0) - 2.0).abs() < 0.01);
    assert!((l.get(1, 0) - 1.0).abs() < 0.01);
    let x = chol.solve(&b).unwrap();
    let arr = x.to_array();
    assert!((arr[0] - 1.0).abs() < 1e-10);
    assert!((arr[1] - 1.0).abs() < 1e-10);
}

#[test]
fn wasm_dbscan() {
    // 4 points in 2D (rows = samples): (0,0), (1,0), (0,1), (10,10). eps=2, min_pts=2.
    let data = vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 10.0, 10.0];
    let m = WasmMatrix::from_array(4, 2, &data).unwrap();
    let db = WasmDbscan::new(&m, 2.0, 2).unwrap();
    let labels = db.get_labels();
    assert_eq!(labels.len(), 4);
    assert!(
        db.n_clusters() >= 1,
        "at least one cluster from the three close points"
    );
}

#[test]
fn wasm_lu_solve() {
    // 2x2 system: [[1,1],[1,-1]] x = [2, 0] -> x = [1, 1]
    let a_data = vec![1.0, 1.0, 1.0, -1.0];
    let a = WasmMatrix::from_array(2, 2, &a_data).unwrap();
    let lu = WasmLu::new(&a).unwrap();
    let b = WasmVector::from_array(&[2.0, 0.0]);
    let x = lu.solve(&b).unwrap();
    let x_arr = x.to_array();
    assert!((x_arr[0] - 1.0).abs() < 1e-10);
    assert!((x_arr[1] - 1.0).abs() < 1e-10);
}
