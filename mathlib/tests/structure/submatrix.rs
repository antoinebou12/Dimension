use mathlib::{Matrix, Storage};

#[test]
fn submatrix_get_set() {
    let mut m: Matrix<f64> = Matrix::with_storage(5, 5, Storage::Column);
    m.set_zero();
    m.set(1, 1, 7.0);
    m.set(2, 2, 8.0);
    let block = m.block(1, 1, 2, 2);
    assert!((block.get(0, 0) - 7.0_f64).abs() < 1e-9);
    assert!((block.get(1, 1) - 8.0_f64).abs() < 1e-9);
}

#[test]
fn submatrix_transpose() {
    let mut m: Matrix<f64> = Matrix::with_storage(4, 4, Storage::Column);
    m.set_zero();
    m.set(0, 1, 1.0);
    m.set(1, 0, 2.0);
    let block = m.block(0, 0, 2, 2);
    let t = block.transpose();
    assert_eq!(t.rows(), 2);
    assert_eq!(t.cols(), 2);
    // Transpose of [[0,1],[2,0]] is [[0,2],[1,0]]
    assert!((t.get(0, 1) - 2.0_f64).abs() < 1e-9);
    assert!((t.get(1, 0) - 1.0_f64).abs() < 1e-9);
}

#[test]
fn submatrix_assign() {
    let mut m: Matrix<f64> = Matrix::with_storage(3, 3, Storage::Column);
    m.set_zero();
    let mut other: Matrix<f64> = Matrix::with_storage(2, 2, Storage::Column);
    other.set(0, 0, 1.0);
    other.set(1, 1, 1.0);
    let mut block = m.block(0, 0, 2, 2);
    block.assign_from(&other);
    assert!((m.get(0, 0) - 1.0_f64).abs() < 1e-9);
    assert!((m.get(1, 1) - 1.0_f64).abs() < 1e-9);
}

#[test]
fn submatrix_to_matrix() {
    let mut m: Matrix<f64> = Matrix::with_storage(4, 4, Storage::Column);
    m.set_zero();
    m.set(1, 1, 5.0_f64);
    let block = m.block(1, 1, 2, 2);
    let out = block.to_matrix();
    assert_eq!(out.rows(), 2);
    assert_eq!(out.cols(), 2);
    assert!((out.get(0, 0) - 5.0_f64).abs() < 1e-9);
}

#[test]
fn submatrix_add_assign() {
    let mut m: Matrix<f64> = Matrix::with_storage(4, 4, Storage::Column);
    m.set_zero();
    for i in 0..4 {
        for j in 0..4 {
            m.set(i, j, (i * 4 + j) as f64);
        }
    }
    let mut add: Matrix<f64> = Matrix::with_storage(2, 2, Storage::Column);
    add.set(0, 0, 1.0);
    add.set(0, 1, 2.0);
    add.set(1, 0, 3.0);
    add.set(1, 1, 4.0);
    {
        let mut block = m.block(1, 1, 2, 2);
        block += &add;
    }
    assert!((m.get(1, 1) - 6.0_f64).abs() < 1e-9);
    assert!((m.get(1, 2) - 8.0_f64).abs() < 1e-9);
    assert!((m.get(2, 1) - 12.0_f64).abs() < 1e-9);
    assert!((m.get(2, 2) - 14.0_f64).abs() < 1e-9);
    assert!((m.get(0, 0) - 0.0_f64).abs() < 1e-9);
}

#[test]
fn submatrix_row_major_to_matrix() {
    let mut m: Matrix<f64> = Matrix::with_storage(3, 3, Storage::Row);
    for i in 0..3 {
        for j in 0..3 {
            m.set(i, j, (i * 3 + j) as f64 + 1.0);
        }
    }
    let block = m.block(0, 1, 2, 2);
    let out = block.to_matrix();
    assert_eq!(out.rows(), 2);
    assert_eq!(out.cols(), 2);
    assert!((out.get(0, 0) - 2.0_f64).abs() < 1e-9);
    assert!((out.get(0, 1) - 3.0_f64).abs() < 1e-9);
    assert!((out.get(1, 0) - 5.0_f64).abs() < 1e-9);
    assert!((out.get(1, 1) - 6.0_f64).abs() < 1e-9);
}
