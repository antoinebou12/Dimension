//! Round-trip serialization tests for types that implement Serialize + Deserialize.
//! Run with: `cargo test --features serde`

#![cfg(feature = "serde")]

use mathlib::{
    Graph, Matrix, Point3, Quat4f, SparseMatrixCRS, SparseStorage, Storage, Triplet, Vector, chol,
};

fn round_trip_json<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) {
    let json = serde_json::to_string(value).expect("serialize");
    let restored: T = serde_json::from_str(&json).expect("deserialize");
    let json2 = serde_json::to_string(&restored).expect("serialize again");
    assert_eq!(json, json2, "round-trip produced different JSON");
}

fn matrix_2x2(a: f64, b: f64, c: f64, d: f64) -> Matrix<f64> {
    let mut m = Matrix::with_dimensions(2, 2);
    m.set(0, 0, a);
    m.set(0, 1, b);
    m.set(1, 0, c);
    m.set(1, 1, d);
    m
}

fn matrix_4x2(data: &[f64]) -> Matrix<f64> {
    let mut m = Matrix::with_dimensions(4, 2);
    for (idx, &v) in data.iter().enumerate() {
        let i = idx / 2;
        let j = idx % 2;
        m.set(i, j, v);
    }
    m
}

fn vector_3(a: f64, b: f64, c: f64) -> Vector<f64> {
    let mut v = Vector::with_capacity(3);
    v.set(0, a);
    v.set(1, b);
    v.set(2, c);
    v
}

#[test]
fn test_storage_roundtrip() {
    round_trip_json(&Storage::Column);
    round_trip_json(&Storage::Row);
}

#[test]
fn test_triplet_roundtrip() {
    round_trip_json(&Triplet::new(1.0_f64, 0, 1));
    round_trip_json(&Triplet::new(-2.5, 2, 0));
}

#[test]
fn test_matrix_roundtrip() {
    let m = matrix_2x2(1.0, 2.0, 3.0, 4.0);
    round_trip_json(&m);
}

#[test]
fn test_vector_roundtrip() {
    let v = vector_3(1.0, 2.0, 3.0);
    round_trip_json(&v);
}

#[test]
fn test_graph_roundtrip() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 2.0);
    round_trip_json(&g);
}

#[test]
fn test_dijkstra_result_roundtrip() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    let result = mathlib::dijkstra(&g, 0);
    round_trip_json(&result);
}

#[test]
fn test_cholesky_roundtrip() {
    let m = matrix_2x2(4.0, 1.0, 1.0, 3.0);
    let factor = chol(&m).expect("chol");
    round_trip_json(&factor);
}

#[test]
fn test_pca_roundtrip() {
    let data = matrix_4x2(&[1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0, -1.0]);
    let pca_result = mathlib::pca(&data, Some(2));
    round_trip_json(&pca_result);
}

#[test]
fn test_dbscan_result_roundtrip() {
    let data = matrix_4x2(&[0.0, 0.0, 1.0, 1.0, 10.0, 10.0, 11.0, 11.0]);
    let result = mathlib::dbscan(&data, 2.0, 2);
    round_trip_json(&result);
}

#[test]
fn test_kmeans_result_roundtrip() {
    let data = matrix_4x2(&[0.0, 0.0, 1.0, 1.0, 10.0, 10.0, 11.0, 11.0]);
    let result = mathlib::kmeans(&data, 2, Some(10));
    round_trip_json(&result);
}

#[test]
fn test_quat4f_roundtrip() {
    let q = Quat4f::identity();
    round_trip_json(&q);
}

#[test]
fn test_point3_roundtrip() {
    let p = Point3::new(1.0, 2.0, 3.0);
    round_trip_json(&p);
}

#[test]
fn test_cube_roundtrip() {
    let mut c = mathlib::Cube::with_dimensions(2, 2, 2);
    c.set(0, 0, 0, 1.0);
    c.set(1, 1, 1, 8.0);
    round_trip_json(&c);
}

#[test]
fn test_svd_econ_roundtrip() {
    let data = matrix_4x2(&[1.0, 0.0, 0.0, 1.0, -1.0, 0.0, 0.0, -1.0]);
    let svd = mathlib::svd_econ(&data);
    round_trip_json(&svd);
}

#[test]
fn test_astar_result_roundtrip() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    let result = mathlib::astar(&g, 0, 2, |_u, _goal| 0.0);
    round_trip_json(&result);
}

#[test]
fn test_dstar_lite_result_roundtrip() {
    let mut g = Graph::new(3);
    g.add_edge(0, 1, 1.0);
    g.add_edge(1, 2, 1.0);
    let result = mathlib::dstar_lite(&mut g, 0, 2);
    round_trip_json(&result);
}

#[test]
fn test_lu_roundtrip() {
    let m = matrix_2x2(1.0, 2.0, 3.0, 4.0);
    let lu = mathlib::Lu::new(&m).expect("lu");
    round_trip_json(&lu);
}

#[test]
fn test_sparse_matrix_crs_roundtrip() {
    let triplets = [Triplet::new(1.0_f64, 0, 0), Triplet::new(2.0, 1, 1)];
    let m = SparseMatrixCRS::from_triplets(2, 2, &triplets);
    round_trip_json(&m);
}
