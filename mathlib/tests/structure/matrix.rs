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

#[test]
fn matrix_new_empty() {
    let m: Matrix<f64> = Matrix::new();
    assert_eq!(m.rows(), 0);
    assert_eq!(m.cols(), 0);
}

#[test]
fn matrix_with_storage() {
    let m: Matrix<f64> = Matrix::with_storage(2, 3, Storage::Row);
    assert_eq!(m.rows(), 2);
    assert_eq!(m.cols(), 3);
}

#[test]
fn matrix_get_set_column_storage() {
    let mut m: Matrix<f64> = Matrix::with_storage(3, 3, Storage::Column);
    m.set(1, 2, 5.5);
    assert!((m.get(1, 2) - 5.5).abs() < 1e-10);
    assert!((m.get(0, 0) - 0.0).abs() < 1e-10);
}

#[test]
fn matrix_get_set_row_storage() {
    let mut m: Matrix<f64> = Matrix::with_storage(3, 3, Storage::Row);
    m.set(1, 2, 7.7);
    assert!((m.get(1, 2) - 7.7).abs() < 1e-10);
}

#[test]
fn matrix_index_operators() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(2, 2);
    m[(0, 1)] = 2.5;
    assert!((m[(0, 1)] - 2.5).abs() < 1e-10);
}

#[test]
fn matrix_resize() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(2, 2);
    m.resize(4, 5);
    assert_eq!(m.rows(), 4);
    assert_eq!(m.cols(), 5);
}

#[test]
fn matrix_set_zero() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(2, 2);
    m.set(0, 0, 10.0);
    m.set_zero();
    assert!((m.get(0, 0) - 0.0).abs() < 1e-10);
}

#[test]
fn matrix_transpose_column() {
    let mut m: Matrix<f64> = Matrix::with_storage(2, 3, Storage::Column);
    m.set(0, 0, 1.0);
    m.set(0, 1, 2.0);
    m.set(0, 2, 3.0);
    m.set(1, 0, 4.0);
    m.set(1, 1, 5.0);
    m.set(1, 2, 6.0);
    let t = m.transpose();
    assert_eq!(t.rows(), 3);
    assert_eq!(t.cols(), 2);
    assert!((t.get(0, 0) - 1.0).abs() < 1e-10);
    assert!((t.get(1, 0) - 2.0).abs() < 1e-10);
    assert!((t.get(2, 1) - 6.0).abs() < 1e-10);
}

#[test]
fn matrix_transpose_row() {
    let mut m: Matrix<f64> = Matrix::with_storage(2, 3, Storage::Row);
    m.set(0, 0, 1.0);
    m.set(1, 2, 6.0);
    let t = m.transpose();
    assert_eq!(t.rows(), 3);
    assert_eq!(t.cols(), 2);
    assert!((t.get(0, 0) - 1.0).abs() < 1e-10);
    assert!((t.get(2, 1) - 6.0).abs() < 1e-10);
}

#[test]
fn matrix_double_transpose() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(2, 3);
    m.set(0, 1, 5.0);
    m.set(1, 2, 7.0);
    let tt = m.transpose().transpose();
    assert_eq!(tt.rows(), 2);
    assert_eq!(tt.cols(), 3);
    assert!((tt.get(0, 1) - 5.0).abs() < 1e-10);
    assert!((tt.get(1, 2) - 7.0).abs() < 1e-10);
}

#[test]
fn matrix_rectangular_identity() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(3, 5);
    m.set_identity();
    assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
    assert!((m.get(1, 1) - 1.0).abs() < 1e-10);
    assert!((m.get(2, 2) - 1.0).abs() < 1e-10);
    assert!((m.get(0, 3) - 0.0).abs() < 1e-10);
}

#[test]
fn matrix_data_access() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(2, 2);
    m.data_mut()[0] = 9.0;
    assert!((m.data()[0] - 9.0).abs() < 1e-10);
}

#[test]
fn matrix_display() {
    let mut m: Matrix<i32> = Matrix::with_dimensions(2, 2);
    m.set(0, 0, 1);
    m.set(1, 1, 4);
    let display = format!("{}", m);
    assert!(display.contains("2x2"));
}

#[test]
fn matrix_pow() {
    let mut a: Matrix<f64> = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 2.0);
    a.set(1, 0, 3.0);
    a.set(1, 1, 4.0);
    let id = a.pow(0);
    assert!((id.get(0, 0) - 1.0).abs() < 1e-10);
    assert!((id.get(1, 1) - 1.0).abs() < 1e-10);
    assert!((id.get(0, 1) - 0.0).abs() < 1e-10);
    assert!((id.get(1, 0) - 0.0).abs() < 1e-10);
    let a1 = a.pow(1);
    assert!((a1.get(0, 0) - a.get(0, 0)).abs() < 1e-10);
    assert!((a1.get(1, 1) - a.get(1, 1)).abs() < 1e-10);
    let a2 = &a * &a;
    let a2_pow = a.pow(2);
    assert!((a2_pow.get(0, 0) - a2.get(0, 0)).abs() < 1e-10);
    assert!((a2_pow.get(0, 1) - a2.get(0, 1)).abs() < 1e-10);
    assert!((a2_pow.get(1, 0) - a2.get(1, 0)).abs() < 1e-10);
    assert!((a2_pow.get(1, 1) - a2.get(1, 1)).abs() < 1e-10);
}
