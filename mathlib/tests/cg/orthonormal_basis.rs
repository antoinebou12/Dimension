//! Tests for OrthonormalBasis on Vector3f.

use mathlib::{OrthonormalBasis, cg::vector3};

const EPS: f32 = 1e-5;

fn assert_near(a: f32, b: f32, eps: f32) {
    assert!((a - b).abs() < eps, "expected {} ~= {} (eps {})", a, b, eps);
}

#[test]
fn orthonormal_basis_unit_z() {
    let n = vector3(0.0, 0.0, 1.0);
    let [t, b] = n.clone().orthonormal_basis();
    assert_near(t.norm(), 1.0, EPS);
    assert_near(b.norm(), 1.0, EPS);
    assert_near(t.dot(&n).abs(), 0.0, EPS);
    assert_near(b.dot(&n).abs(), 0.0, EPS);
    assert_near(t.dot(&b).abs(), 0.0, EPS);
}

#[test]
fn orthonormal_basis_unit_y() {
    let n = vector3(0.0, 1.0, 0.0);
    let [t, b] = n.clone().orthonormal_basis();
    assert_near(t.norm(), 1.0, EPS);
    assert_near(b.norm(), 1.0, EPS);
    assert_near(t.dot(&n).abs(), 0.0, EPS);
    assert_near(b.dot(&n).abs(), 0.0, EPS);
    assert_near(t.dot(&b).abs(), 0.0, EPS);
}

#[test]
fn orthonormal_basis_unit_x() {
    let n = vector3(1.0, 0.0, 0.0);
    let [t, b] = n.clone().orthonormal_basis();
    assert_near(t.norm(), 1.0, EPS);
    assert_near(b.norm(), 1.0, EPS);
    assert_near(t.dot(&n).abs(), 0.0, EPS);
    assert_near(b.dot(&n).abs(), 0.0, EPS);
    assert_near(t.dot(&b).abs(), 0.0, EPS);
}

#[test]
fn orthonormal_basis_arbitrary_unit() {
    let n = vector3(1.0, 2.0, 3.0);
    let n = n.normalize();
    let [t, b] = n.clone().orthonormal_basis();
    assert_near(t.norm(), 1.0, EPS);
    assert_near(b.norm(), 1.0, EPS);
    assert_near(t.dot(&n).abs(), 0.0, EPS);
    assert_near(b.dot(&n).abs(), 0.0, EPS);
    assert_near(t.dot(&b).abs(), 0.0, EPS);
}

#[test]
fn orthonormal_vector_matches_second_basis() {
    let n = vector3(0.0, 1.0, 0.0);
    let [_, b] = n.clone().orthonormal_basis();
    let ov = n.clone().orthonormal_vector();
    assert_near(ov.get(0), b.get(0), EPS);
    assert_near(ov.get(1), b.get(1), EPS);
    assert_near(ov.get(2), b.get(2), EPS);
}
