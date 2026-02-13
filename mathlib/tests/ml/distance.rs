use mathlib::distance::{cosine_distance_rows, cosine_similarity_rows, minkowski_rows};
use mathlib::{
    Matrix, Storage, Vector, chebyshev, chebyshev_rows, cosine_distance, cosine_similarity,
    euclidean, euclidean_rows, manhattan, minkowski, squared_euclidean, squared_euclidean_rows,
};

#[test]
fn euclidean_vectors() {
    let mut a = Vector::with_capacity(3);
    a.set(0, 0.0);
    a.set(1, 0.0);
    a.set(2, 0.0);
    let mut b = Vector::with_capacity(3);
    b.set(0, 3.0);
    b.set(1, 4.0);
    b.set(2, 0.0);
    assert!((euclidean(&a, &b) - 5.0).abs() < 1e-10);
    assert!((squared_euclidean(&a, &b) - 25.0).abs() < 1e-10);
}

#[test]
fn test_euclidean_rows() {
    let mut data = Matrix::with_storage(2, 3, Storage::Column);
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(0, 2, 0.0);
    data.set(1, 0, 3.0);
    data.set(1, 1, 4.0);
    data.set(1, 2, 0.0);
    assert!((euclidean_rows(&data, 0, 1) - 5.0).abs() < 1e-10);
    assert!((squared_euclidean_rows(&data, 0, 1) - 25.0).abs() < 1e-10);
}

#[test]
fn manhattan_vectors() {
    let mut a = Vector::with_capacity(2);
    a.set(0, 0.0);
    a.set(1, 0.0);
    let mut b = Vector::with_capacity(2);
    b.set(0, 3.0);
    b.set(1, 4.0);
    assert!((manhattan(&a, &b) - 7.0).abs() < 1e-10);
}

#[test]
fn minkowski_p2_is_euclidean() {
    let mut a = Vector::with_capacity(2);
    a.set(0, 1.0);
    a.set(1, 2.0);
    let mut b = Vector::with_capacity(2);
    b.set(0, 4.0);
    b.set(1, 6.0);
    assert!((minkowski(&a, &b, 2.0) - euclidean(&a, &b)).abs() < 1e-10);
}

#[test]
fn cosine_similarity_same_direction() {
    let mut a = Vector::with_capacity(3);
    a.set(0, 1.0);
    a.set(1, 0.0);
    a.set(2, 0.0);
    let mut b = Vector::with_capacity(3);
    b.set(0, 2.0);
    b.set(1, 0.0);
    b.set(2, 0.0);
    assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-10);
    assert!(cosine_distance(&a, &b).abs() < 1e-10);
}

#[test]
fn cosine_similarity_orthogonal() {
    let mut a = Vector::with_capacity(2);
    a.set(0, 1.0);
    a.set(1, 0.0);
    let mut b = Vector::with_capacity(2);
    b.set(0, 0.0);
    b.set(1, 1.0);
    assert!(cosine_similarity(&a, &b).abs() < 1e-10);
    assert!((cosine_distance(&a, &b) - 1.0).abs() < 1e-10);
}

#[test]
fn chebyshev_vectors() {
    let mut a = Vector::with_capacity(3);
    a.set(0, 0.0);
    a.set(1, 0.0);
    a.set(2, 0.0);
    let mut b = Vector::with_capacity(3);
    b.set(0, 1.0);
    b.set(1, 5.0);
    b.set(2, 3.0);
    assert!((chebyshev(&a, &b) - 5.0).abs() < 1e-10);
}

#[test]
fn test_chebyshev_rows() {
    let mut data = Matrix::with_storage(2, 2, Storage::Column);
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(1, 0, 2.0);
    data.set(1, 1, 5.0);
    assert!((chebyshev_rows(&data, 0, 1) - 5.0).abs() < 1e-10);
}

#[test]
fn minkowski_infinity_is_chebyshev() {
    let mut a = Vector::with_capacity(3);
    a.set(0, 0.0);
    a.set(1, 0.0);
    a.set(2, 0.0);
    let mut b = Vector::with_capacity(3);
    b.set(0, 1.0);
    b.set(1, 5.0);
    b.set(2, 3.0);
    assert!((minkowski(&a, &b, f64::INFINITY) - chebyshev(&a, &b)).abs() < 1e-10);
}

#[test]
fn test_minkowski_rows() {
    let mut data = Matrix::with_storage(2, 3, Storage::Column);
    data.set(0, 0, 1.0);
    data.set(0, 1, 2.0);
    data.set(0, 2, 3.0);
    data.set(1, 0, 4.0);
    data.set(1, 1, 6.0);
    data.set(1, 2, 8.0);
    let d_p2 = minkowski_rows(&data, 0, 1, 2.0);
    assert!((d_p2 - euclidean_rows(&data, 0, 1)).abs() < 1e-10);
    let d_inf = minkowski_rows(&data, 0, 1, f64::INFINITY);
    assert!((d_inf - chebyshev_rows(&data, 0, 1)).abs() < 1e-10);
}

#[test]
fn cosine_similarity_rows_and_distance_rows() {
    let mut data = Matrix::with_storage(2, 3, Storage::Column);
    data.set(0, 0, 1.0);
    data.set(0, 1, 0.0);
    data.set(0, 2, 0.0);
    data.set(1, 0, 2.0);
    data.set(1, 1, 0.0);
    data.set(1, 2, 0.0);
    let sim = cosine_similarity_rows(&data, 0, 1);
    assert!((sim - 1.0).abs() < 1e-10);
    let dist = cosine_distance_rows(&data, 0, 1);
    assert!(dist.abs() < 1e-10);
}

#[test]
fn cosine_similarity_zero_norm() {
    let mut a = Vector::with_capacity(2);
    a.set(0, 0.0);
    a.set(1, 0.0);
    let mut b = Vector::with_capacity(2);
    b.set(0, 1.0);
    b.set(1, 0.0);
    let sim = cosine_similarity(&a, &b);
    assert!(sim.abs() < 1e-20);
}
