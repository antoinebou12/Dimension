use mathlib::{Cube, Matrix, Vector};

fn make_vector(data: &[f64]) -> Vector<f64> {
    let mut v = Vector::with_capacity(data.len());
    for (i, &val) in data.iter().enumerate() {
        v.set(i, val);
    }
    v
}

fn make_matrix(rows: usize, cols: usize, data: &[f64]) -> Matrix<f64> {
    let mut m = Matrix::with_dimensions(rows, cols);
    for i in 0..rows {
        for j in 0..cols {
            m.set(i, j, data[i * cols + j]);
        }
    }
    m
}

fn make_cube(rows: usize, cols: usize, slices: usize, data: &[f64]) -> Cube<f64> {
    let mut c = Cube::with_dimensions(rows, cols, slices);
    for (idx, &val) in data.iter().enumerate() {
        let n = rows * cols;
        let k = idx / n;
        let r = idx % n;
        let j = r / rows;
        let i = r % rows;
        c.set(i, j, k, val);
    }
    c
}

#[test]
fn operators_matrix_add() {
    let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let b = make_matrix(2, 2, &[5.0, 6.0, 7.0, 8.0]);
    let c = &a + &b;
    assert!((c.get(0, 0) - 6.0).abs() < 1e-10);
    assert!((c.get(0, 1) - 8.0).abs() < 1e-10);
    assert!((c.get(1, 0) - 10.0).abs() < 1e-10);
    assert!((c.get(1, 1) - 12.0).abs() < 1e-10);
}

#[test]
fn operators_matrix_sub() {
    let a = make_matrix(2, 2, &[5.0, 6.0, 7.0, 8.0]);
    let b = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let c = &a - &b;
    assert!((c.get(0, 0) - 4.0).abs() < 1e-10);
    assert!((c.get(1, 1) - 4.0).abs() < 1e-10);
}

#[test]
fn operators_matrix_mul() {
    let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let b = make_matrix(2, 2, &[5.0, 6.0, 7.0, 8.0]);
    let c = &a * &b;
    assert!((c.get(0, 0) - 19.0).abs() < 1e-10);
    assert!((c.get(0, 1) - 22.0).abs() < 1e-10);
    assert!((c.get(1, 0) - 43.0).abs() < 1e-10);
    assert!((c.get(1, 1) - 50.0).abs() < 1e-10);
}

#[test]
fn operators_matrix_mul_identity() {
    let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let mut i = Matrix::with_dimensions(2, 2);
    i.set_identity();
    let c = &a * &i;
    assert!((c.get(0, 0) - 1.0).abs() < 1e-10);
    assert!((c.get(0, 1) - 2.0).abs() < 1e-10);
    assert!((c.get(1, 0) - 3.0).abs() < 1e-10);
    assert!((c.get(1, 1) - 4.0).abs() < 1e-10);
}

#[test]
fn operators_matrix_mul_rectangular() {
    let a = make_matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let b = make_matrix(3, 1, &[7.0, 8.0, 9.0]);
    let c = &a * &b;
    assert_eq!(c.rows(), 2);
    assert_eq!(c.cols(), 1);
    assert!((c.get(0, 0) - 50.0).abs() < 1e-10);
    assert!((c.get(1, 0) - 122.0).abs() < 1e-10);
}

#[test]
fn operators_scalar_mul_matrix_f64() {
    let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let c = 2.0_f64 * &a;
    assert!((c.get(0, 0) - 2.0).abs() < 1e-10);
    assert!((c.get(1, 1) - 8.0).abs() < 1e-10);
}

#[test]
fn operators_scalar_mul_matrix_f32() {
    let mut a: Matrix<f32> = Matrix::with_dimensions(2, 2);
    a.set(0, 0, 1.0);
    a.set(1, 1, 4.0);
    let c = 3.0_f32 * &a;
    assert!((c.get(0, 0) - 3.0).abs() < 1e-6);
    assert!((c.get(1, 1) - 12.0).abs() < 1e-6);
}

#[test]
fn operators_matrix_vector_mul() {
    let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let v = make_vector(&[3.0, 4.0]);
    let c = &a * &v;
    assert_eq!(c.rows(), 2);
    assert!((c.get(0) - 11.0).abs() < 1e-10);
    assert!((c.get(1) - 25.0).abs() < 1e-10);
}

#[test]
fn operators_scalar_mul_vector_f64() {
    let v = make_vector(&[1.0, 2.0, 3.0]);
    let c = 2.0_f64 * &v;
    assert!((c.get(0) - 2.0).abs() < 1e-10);
    assert!((c.get(1) - 4.0).abs() < 1e-10);
    assert!((c.get(2) - 6.0).abs() < 1e-10);
}

#[test]
fn operators_scalar_mul_vector_f32() {
    let mut v: Vector<f32> = Vector::with_capacity(2);
    v.set(0, 1.0);
    v.set(1, 2.0);
    let c = 5.0_f32 * &v;
    assert!((c.get(0) - 5.0).abs() < 1e-6);
    assert!((c.get(1) - 10.0).abs() < 1e-6);
}

#[test]
fn operators_vector_add() {
    let a = make_vector(&[1.0, 2.0, 3.0]);
    let b = make_vector(&[4.0, 5.0, 6.0]);
    let c = &a + &b;
    assert!((c.get(0) - 5.0).abs() < 1e-10);
    assert!((c.get(1) - 7.0).abs() < 1e-10);
    assert!((c.get(2) - 9.0).abs() < 1e-10);
}

#[test]
fn operators_vector_sub() {
    let a = make_vector(&[4.0, 5.0, 6.0]);
    let b = make_vector(&[1.0, 2.0, 3.0]);
    let c = &a - &b;
    assert!((c.get(0) - 3.0).abs() < 1e-10);
    assert!((c.get(1) - 3.0).abs() < 1e-10);
    assert!((c.get(2) - 3.0).abs() < 1e-10);
}

#[test]
fn operators_cube_add() {
    let a = make_cube(2, 2, 2, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    let b = make_cube(2, 2, 2, &[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0]);
    let c = &a + &b;
    assert!((c.get(0, 0, 0) - 11.0).abs() < 1e-10);
    assert!((c.get(1, 1, 1) - 88.0).abs() < 1e-10);
}

#[test]
fn operators_cube_sub() {
    let a = make_cube(2, 2, 1, &[5.0, 6.0, 7.0, 8.0]);
    let b = make_cube(2, 2, 1, &[1.0, 2.0, 3.0, 4.0]);
    let c = &a - &b;
    assert!((c.get(0, 0, 0) - 4.0).abs() < 1e-10);
    assert!((c.get(1, 1, 0) - 4.0).abs() < 1e-10);
}

#[test]
fn operators_scalar_mul_cube_f64() {
    let a = make_cube(2, 2, 1, &[1.0, 2.0, 3.0, 4.0]);
    let c = 2.0_f64 * &a;
    assert!((c.get(0, 0, 0) - 2.0).abs() < 1e-10);
    assert!((c.get(1, 1, 0) - 8.0).abs() < 1e-10);
}

#[test]
fn operators_scalar_mul_cube_f32() {
    let mut a: Cube<f32> = Cube::with_dimensions(2, 2, 1);
    a.set(0, 0, 0, 1.0);
    a.set(1, 1, 0, 4.0);
    let c = 3.0_f32 * &a;
    assert!((c.get(0, 0, 0) - 3.0).abs() < 1e-6);
    assert!((c.get(1, 1, 0) - 12.0).abs() < 1e-6);
}

#[test]
fn matrix_identity_scalar() {
    let mut a: Matrix<f64> = Matrix::with_dimensions(6, 6);
    a.set_identity();
    let alpha = 2.5_f64;
    let b: Matrix<f64> = alpha * &a;
    for i in 0..6 {
        assert!((b.get(i, i) - alpha).abs() < 1e-9);
    }
    assert!(b.get(0, 1).abs() < 1e-9);
}

#[test]
fn matrix_matrix_mul() {
    let mut a: Matrix<f64> = Matrix::with_dimensions(6, 6);
    a.set_identity();
    let b: Matrix<f64> = 2.5_f64 * &a;
    let c: Matrix<f64> = &a * &b;
    for i in 0..6 {
        assert!((c.get(i, i) - 2.5).abs() < 1e-9);
    }
}

#[test]
fn matrix_add() {
    let mut a: Matrix<f64> = Matrix::with_dimensions(6, 6);
    a.set_identity();
    let b: Matrix<f64> = 2.5_f64 * &a;
    let a_plus_b: Matrix<f64> = &a + &b;
    for i in 0..6 {
        assert!((a_plus_b.get(i, i) - 3.5).abs() < 1e-9);
    }
}

#[test]
fn matrix_vector_mul() {
    let mut m: Matrix<f64> = Matrix::with_dimensions(5, 5);
    m.set_identity();
    let mut v: Vector<f64> = Vector::with_capacity(5);
    v.set(0, 1.0);
    v.set(1, 2.0);
    v.set(2, 4.0);
    v.set(3, 8.0);
    v.set(4, 16.0);
    let b = &m * &v;
    assert!((b.get(0) - 1.0).abs() < 1e-9);
    assert!((b.get(4) - 16.0).abs() < 1e-9);
}

#[test]
fn vector_ops() {
    let mut v: Vector<f64> = Vector::with_capacity(5);
    v.set(0, 0.1);
    v.set(1, 0.2);
    v.set(2, 0.4);
    v.set(3, 0.8);
    v.set(4, 1.6);
    let v2: Vector<f64> = 4.0_f64 * &v;
    assert!((v2.get(0) - 0.4_f64).abs() < 1e-9);
    let v3: Vector<f64> = &v + &v2;
    assert!((v3.get(0) - 0.5_f64).abs() < 1e-9);
}

#[test]
fn operators_vector_dot() {
    let u = make_vector(&[1.0, 2.0, 3.0]);
    let v = make_vector(&[4.0, 5.0, 6.0]);
    let dot = u.dot(&v);
    assert!((dot - 32.0_f64).abs() < 1e-10);
}
