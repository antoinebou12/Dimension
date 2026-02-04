use mathlib::{
    Matrix4f, Point3, Vector3f, Vector4f, center, from_homogeneous_point, from_homogeneous_vector,
    matrix4_mul_vector3, point_to_homogeneous, vector_to_homogeneous,
};

fn point_approx_eq(a: &Point3, b: &Point3, tol: f32) -> bool {
    (a.coords.get(0) - b.coords.get(0)).abs() < tol
        && (a.coords.get(1) - b.coords.get(1)).abs() < tol
        && (a.coords.get(2) - b.coords.get(2)).abs() < tol
}

#[test]
fn point3_new_and_from_coordinates() {
    let p = Point3::new(1.0, 2.0, 3.0);
    assert!((p.coords.get(0) - 1.0).abs() < 1e-9);
    assert!((p.coords.get(1) - 2.0).abs() < 1e-9);
    assert!((p.coords.get(2) - 3.0).abs() < 1e-9);
    let v = Vector3f::with_capacity(3);
    let mut v = v;
    v.set(0, 4.0);
    v.set(1, 5.0);
    v.set(2, 6.0);
    let p2 = Point3::from_coordinates(v);
    assert!((p2.coords.get(0) - 4.0).abs() < 1e-9);
    assert!((p2.coords.get(1) - 5.0).abs() < 1e-9);
    assert!((p2.coords.get(2) - 6.0).abs() < 1e-9);
}

#[test]
fn point3_origin() {
    let o = Point3::origin();
    assert!((o.coords.get(0) - 0.0).abs() < 1e-9);
    assert!((o.coords.get(1) - 0.0).abs() < 1e-9);
    assert!((o.coords.get(2) - 0.0).abs() < 1e-9);
}

#[test]
fn point_plus_vector() {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, 1.0);
    v.set(1, 2.0);
    v.set(2, 3.0);
    let p = Point3::origin() + v.clone();
    assert!((p.coords.get(0) - 1.0).abs() < 1e-9);
    assert!((p.coords.get(1) - 2.0).abs() < 1e-9);
    assert!((p.coords.get(2) - 3.0).abs() < 1e-9);
    let p_ref = &Point3::origin() + v;
    assert!((p_ref.coords.get(0) - 1.0).abs() < 1e-9);
}

#[test]
fn point_minus_point() {
    let a = Point3::new(1.0, 2.0, 3.0);
    let b = Point3::new(4.0, 6.0, 8.0);
    let v = &b - &a;
    assert!((v.get(0) - 3.0).abs() < 1e-9);
    assert!((v.get(1) - 4.0).abs() < 1e-9);
    assert!((v.get(2) - 5.0).abs() < 1e-9);
    let a_plus_diff = &a + v;
    assert!(point_approx_eq(&a_plus_diff, &b, 1e-5));
}

#[test]
fn center_midpoint() {
    let a = Point3::new(0.0, 0.0, 0.0);
    let b = Point3::new(2.0, 4.0, 6.0);
    let c = center(&a, &b);
    assert!((c.coords.get(0) - 1.0).abs() < 1e-9);
    assert!((c.coords.get(1) - 2.0).abs() < 1e-9);
    assert!((c.coords.get(2) - 3.0).abs() < 1e-9);
}

#[test]
fn to_homogeneous_point_ends_with_one() {
    let p = Point3::new(1.0, 2.0, 3.0);
    let h = point_to_homogeneous(&p);
    assert!((h.get(0) - 1.0).abs() < 1e-9);
    assert!((h.get(1) - 2.0).abs() < 1e-9);
    assert!((h.get(2) - 3.0).abs() < 1e-9);
    assert!((h.get(3) - 1.0).abs() < 1e-9);
}

#[test]
fn to_homogeneous_vector_ends_with_zero() {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, 1.0);
    v.set(1, 2.0);
    v.set(2, 3.0);
    let h = vector_to_homogeneous(&v);
    assert!((h.get(0) - 1.0).abs() < 1e-9);
    assert!((h.get(1) - 2.0).abs() < 1e-9);
    assert!((h.get(2) - 3.0).abs() < 1e-9);
    assert!((h.get(3) - 0.0).abs() < 1e-9);
}

#[test]
fn from_homogeneous_point_roundtrip() {
    let p = Point3::new(1.5, 2.5, 3.5);
    let h = point_to_homogeneous(&p);
    let q = from_homogeneous_point(&h).unwrap();
    assert!(point_approx_eq(&p, &q, 1e-5));
}

#[test]
fn from_homogeneous_vector_roundtrip() {
    let mut v = Vector3f::with_capacity(3);
    v.set(0, 1.0);
    v.set(1, 2.0);
    v.set(2, 3.0);
    let h = vector_to_homogeneous(&v);
    let w = from_homogeneous_vector(&h);
    assert!((w.get(0) - v.get(0)).abs() < 1e-9);
    assert!((w.get(1) - v.get(1)).abs() < 1e-9);
    assert!((w.get(2) - v.get(2)).abs() < 1e-9);
}

#[test]
fn transform_via_homogeneous_matches_matrix4_mul_vector3() {
    let mut m = Matrix4f::with_dimensions(4, 4);
    m.set_identity();
    m.set(0, 3, 10.0);
    m.set(1, 3, 20.0);
    m.set(2, 3, 30.0);
    let p = Point3::new(1.0, 2.0, 3.0);
    let direct = matrix4_mul_vector3(&m, &p.coords);
    let home_pt = point_to_homogeneous(&p);
    let home_out = &m * &home_pt;
    let via_home = from_homogeneous_point(&home_out).unwrap();
    assert!((via_home.coords.get(0) - direct.get(0)).abs() < 1e-5);
    assert!((via_home.coords.get(1) - direct.get(1)).abs() < 1e-5);
    assert!((via_home.coords.get(2) - direct.get(2)).abs() < 1e-5);
}

#[test]
fn from_homogeneous_point_zero_w_returns_none() {
    let mut h4 = Vector4f::with_capacity(4);
    h4.set(0, 1.0);
    h4.set(1, 2.0);
    h4.set(2, 3.0);
    h4.set(3, 0.0);
    let o = from_homogeneous_point(&h4);
    assert!(o.is_none());
}
