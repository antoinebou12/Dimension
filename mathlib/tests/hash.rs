//! Integration tests for Hash/PartialEq/Eq on Matrix and Vector.

use mathlib::{Matrix, Storage, Vector};
use std::collections::HashSet;

fn make_matrix_f64(rows: usize, cols: usize, data: &[f64]) -> Matrix<f64> {
    let mut m = Matrix::with_storage(rows, cols, Storage::Column);
    for i in 0..rows {
        for j in 0..cols {
            m.set(i, j, data[i * cols + j]);
        }
    }
    m
}

fn make_vector_f64(data: &[f64]) -> Vector<f64> {
    let mut v = Vector::with_capacity(data.len());
    for (i, &x) in data.iter().enumerate() {
        v.set(i, x);
    }
    v
}

#[test]
fn hash_matrix_f64_hashset_insert_contains() {
    let a = make_matrix_f64(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let b = make_matrix_f64(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    assert_eq!(a, b);
    let mut set = HashSet::new();
    set.insert(a.clone());
    assert!(set.contains(&b));
}

#[test]
fn hash_matrix_f64_hashset_different_not_contains() {
    let a = make_matrix_f64(2, 2, &[1.0, 2.0, 3.0, 4.0]);
    let b = make_matrix_f64(2, 2, &[1.0, 2.0, 3.0, 5.0]);
    assert_ne!(a, b);
    let mut set = HashSet::new();
    set.insert(a);
    assert!(!set.contains(&b));
}

#[test]
fn hash_vector_f64_hashset_insert_contains() {
    let a = make_vector_f64(&[1.0, 2.0, 3.0]);
    let b = make_vector_f64(&[1.0, 2.0, 3.0]);
    assert_eq!(a, b);
    let mut set = HashSet::new();
    set.insert(a.clone());
    assert!(set.contains(&b));
}

#[test]
fn hash_vector_f64_hashset_different_not_contains() {
    let a = make_vector_f64(&[1.0, 2.0, 3.0]);
    let b = make_vector_f64(&[1.0, 2.0, 4.0]);
    assert_ne!(a, b);
    let mut set = HashSet::new();
    set.insert(a);
    assert!(!set.contains(&b));
}

#[test]
fn hash_matrix_i32_hashset_insert_contains() {
    let mut a: Matrix<i32> = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 1);
    a.set(1, 0, 2);
    a.set(0, 1, 3);
    a.set(1, 1, 4);
    let mut b: Matrix<i32> = Matrix::with_storage(2, 2, Storage::Column);
    b.set(0, 0, 1);
    b.set(1, 0, 2);
    b.set(0, 1, 3);
    b.set(1, 1, 4);
    assert_eq!(a, b);
    let mut set = HashSet::new();
    set.insert(a.clone());
    assert!(set.contains(&b));
}

#[test]
fn hash_vector_i32_hashset_insert_contains() {
    let mut a: Vector<i32> = Vector::with_capacity(3);
    a.set(0, 1);
    a.set(1, 2);
    a.set(2, 3);
    let mut b: Vector<i32> = Vector::with_capacity(3);
    b.set(0, 1);
    b.set(1, 2);
    b.set(2, 3);
    assert_eq!(a, b);
    let mut set = HashSet::new();
    set.insert(a.clone());
    assert!(set.contains(&b));
}
