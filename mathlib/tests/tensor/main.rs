//! Integration tests for tensor domain (Cube, N-dimensional).

use mathlib::{Cube, CubeSlice, Fill};

#[test]
fn cube_construction() {
    let c: Cube<f64> = Cube::with_dimensions(2, 3, 4);
    assert_eq!(c.rows(), 2);
    assert_eq!(c.cols(), 3);
    assert_eq!(c.slices(), 4);
}

#[test]
fn cube_get_set() {
    let mut c: Cube<f64> = Cube::with_dimensions(2, 2, 2);
    c.set(0, 0, 0, 1.0);
    c.set(1, 1, 1, 8.0);
    assert!((c.get(0, 0, 0) - 1.0).abs() < 1e-10);
    assert!((c.get(1, 1, 1) - 8.0).abs() < 1e-10);
}

#[test]
fn cube_slice_view() {
    let mut c: Cube<f64> = Cube::with_dimensions(2, 3, 2);
    c.set(0, 1, 0, 5.0);
    c.set(1, 2, 1, 9.0);
    let mut view: CubeSlice<'_, f64> = c.slice(0);
    assert!((view.get(0, 1) - 5.0).abs() < 1e-10);
    view.set(1, 0, 7.0);
    assert!((c.get(1, 0, 0) - 7.0).abs() < 1e-10);
}

#[test]
fn cube_slice_to_matrix() {
    let mut c: Cube<f64> = Cube::with_dimensions(2, 2, 1);
    c.set(0, 0, 0, 1.0);
    c.set(1, 0, 0, 2.0);
    c.set(0, 1, 0, 3.0);
    c.set(1, 1, 0, 4.0);
    let view = c.slice(0);
    let m = view.to_matrix();
    assert_eq!(m.rows(), 2);
    assert_eq!(m.cols(), 2);
    assert!((m.get(0, 0) - 1.0).abs() < 1e-10);
    assert!((m.get(1, 1) - 4.0).abs() < 1e-10);
}

#[test]
fn cube_set_zero() {
    let mut c: Cube<f64> = Cube::with_dimensions(2, 2, 2);
    c.set(0, 0, 0, 3.0);
    c.set_zero();
    assert!((c.get(0, 0, 0) - 0.0).abs() < 1e-10);
}

#[test]
fn cube_resize() {
    let mut c: Cube<f64> = Cube::with_dimensions(2, 2, 2);
    c.resize(3, 4, 2);
    assert_eq!(c.rows(), 3);
    assert_eq!(c.cols(), 4);
    assert_eq!(c.slices(), 2);
}

#[test]
fn cube_fill_ones() {
    let c: Cube<f64> = Cube::with_dimensions_fill(2, 2, 2, Fill::Ones);
    assert!((c.get(0, 0, 0) - 1.0).abs() < 1e-10);
    assert!((c.get(1, 1, 1) - 1.0).abs() < 1e-10);
}

#[test]
fn cube_add_sub() {
    let mut a: Cube<f64> = Cube::with_dimensions(2, 2, 1);
    a.set(0, 0, 0, 1.0);
    a.set(0, 1, 0, 2.0);
    a.set(1, 0, 0, 3.0);
    a.set(1, 1, 0, 4.0);
    let mut b: Cube<f64> = Cube::with_dimensions(2, 2, 1);
    b.set(0, 0, 0, 10.0);
    b.set(0, 1, 0, 20.0);
    b.set(1, 0, 0, 30.0);
    b.set(1, 1, 0, 40.0);
    let sum = &a + &b;
    assert!((sum.get(0, 0, 0) - 11.0).abs() < 1e-10);
    assert!((sum.get(1, 1, 0) - 44.0).abs() < 1e-10);
    let diff = &sum - &a;
    assert!((diff.get(0, 0, 0) - 10.0).abs() < 1e-10);
    assert!((diff.get(1, 1, 0) - 40.0).abs() < 1e-10);
}
