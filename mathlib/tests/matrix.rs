use mathlib::{Matrix, Storage};

#[test]
fn matrix_dynamic() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(3, 5);
    assert_eq!(m.cols(), 5);
    assert_eq!(m.rows(), 3);
    m.resize(100, 1000);
    assert_eq!(m.cols(), 1000);
    assert_eq!(m.rows(), 100);
}

#[test]
fn matrix_column_storage() {
    let mut m: Matrix<f64> = Matrix::with_storage(100, 99, Storage::Column);
    m.set_zero();
    m.set(0, 0, 1.0);
    m.set(90, 17, 99.0);
    m.set(10, 33, 7.2);
    assert!((m.get(0, 0) - 1.0).abs() < 1e-9);
    assert!((m.get(90, 17) - 99.0).abs() < 1e-9);
    assert!((m.get(10, 33) - 7.2).abs() < 1e-9);
}

#[test]
fn matrix_row_storage() {
    let mut m: Matrix<f64> = Matrix::with_storage(5, 4, Storage::Row);
    m.set_zero();
    m.set(0, 0, 2.1);
    m.set(3, 3, -0.2);
    m.set(4, 3, 1.2);
    assert_eq!(m.rows(), 5);
    assert_eq!(m.cols(), 4);
    assert!((m.get(0, 0) - 2.1).abs() < 1e-9);
    assert!((m.get(3, 3) + 0.2).abs() < 1e-9);
    assert!((m.get(4, 3) - 1.2).abs() < 1e-9);
    assert!(m.get(3, 2).abs() < 1e-9);
}

#[test]
fn matrix_transpose() {
    let mut m: Matrix<f64> = Matrix::with_storage(5, 4, Storage::Row);
    m.set_zero();
    m.set(0, 0, 2.1);
    m.set(3, 3, -0.2);
    m.set(4, 3, 1.2);
    let mt = m.transpose();
    assert_eq!(mt.rows(), 4);
    assert_eq!(mt.cols(), 5);
    assert!((mt.get(0, 0) - 2.1).abs() < 1e-9);
    assert!((mt.get(3, 3) + 0.2).abs() < 1e-9);
    assert!((mt.get(3, 4) - 1.2).abs() < 1e-9);
}

#[test]
fn matrix_identity() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(6, 6);
    m.set_identity();
    for i in 0..6 {
        assert!((m.get(i, i) - 1.0).abs() < 1e-9);
    }
    assert!(m.get(0, 1).abs() < 1e-9);
    assert!(m.get(1, 0).abs() < 1e-9);
}
