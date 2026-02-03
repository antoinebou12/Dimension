//! Unit tests for computer-graphics recipes.

use mathlib::{
    Perspective3, Vector3f, from_euler_angles, from_homogeneous, from_scaled_axis, look_at_rh,
    model_view_projection, new_nonuniform_scaling, new_orthographic, new_perspective,
    new_rotation_wrt_point, new_scaling, new_translation, transform_point, transform_vector,
    vector3, vector4_from_point,
};

fn assert_near(a: f32, b: f32, tol: f32) {
    assert!((a - b).abs() < tol, "{} vs {}", a, b);
}

fn assert_vec3_near(a: &Vector3f, b: &Vector3f, tol: f32) {
    assert_near(a.get(0), b.get(0), tol);
    assert_near(a.get(1), b.get(1), tol);
    assert_near(a.get(2), b.get(2), tol);
}

#[test]
fn cg_new_scaling() {
    let m = new_scaling(2.0);
    let x = vector3(1.0, 0.0, 0.0);
    let y = vector3(0.0, 1.0, 0.0);
    let z = vector3(0.0, 0.0, 1.0);
    assert_vec3_near(&transform_vector(&m, &x), &vector3(2.0, 0.0, 0.0), 1e-5);
    assert_vec3_near(&transform_vector(&m, &y), &vector3(0.0, 2.0, 0.0), 1e-5);
    assert_vec3_near(&transform_vector(&m, &z), &vector3(0.0, 0.0, 2.0), 1e-5);
}

#[test]
fn cg_new_nonuniform_scaling() {
    let m = new_nonuniform_scaling(&vector3(1.0, 2.0, 3.0));
    assert_vec3_near(
        &transform_vector(&m, &vector3(1.0, 1.0, 1.0)),
        &vector3(1.0, 2.0, 3.0),
        1e-5,
    );
}

#[test]
fn cg_new_translation() {
    let t = vector3(10.0, 20.0, 30.0);
    let m = new_translation(&t);
    let p = vector3(1.0, 2.0, 3.0);
    let out = transform_point(&m, &p);
    assert_vec3_near(&out, &vector3(11.0, 22.0, 33.0), 1e-5);
    // Vector (direction) should not be translated
    assert_vec3_near(&transform_vector(&m, &p), &p, 1e-5);
}

#[test]
fn cg_from_scaled_axis_identity() {
    let m = from_scaled_axis(&vector3(0.0, 0.0, 0.0));
    let p = vector3(1.0, 2.0, 3.0);
    assert_vec3_near(&transform_vector(&m, &p), &p, 1e-5);
}

#[test]
fn cg_from_scaled_axis_rotation_90_z() {
    // 90° around Z: (1,0,0) -> (0,1,0)
    let angle = std::f32::consts::FRAC_PI_2;
    let m = from_scaled_axis(&vector3(0.0, 0.0, angle));
    let x = vector3(1.0, 0.0, 0.0);
    let out = transform_vector(&m, &x);
    assert_vec3_near(&out, &vector3(0.0, 1.0, 0.0), 1e-5);
}

#[test]
fn cg_new_rotation_wrt_point() {
    // Rotate 180° around Z at point (1, 0, 0); point (1,0,0) stays fixed
    let pt = vector3(1.0, 0.0, 0.0);
    let angle = std::f32::consts::PI;
    let m = new_rotation_wrt_point(&vector3(0.0, 0.0, angle), &pt);
    let out = transform_point(&m, &pt);
    assert_vec3_near(&out, &pt, 1e-5);
}

#[test]
fn cg_append_prepend_translation() {
    let m = new_scaling(2.0);
    let t = vector3(42.0, 0.0, 0.0);
    // append_translation(M, t) = M * T, so (M*T)*p = M*(p+t) = scale * (1+42, 1, 1) = (86, 2, 2)
    let m2 = mathlib::append_translation(&m, &t);
    let p = vector3(1.0, 1.0, 1.0);
    let out = transform_point(&m2, &p);
    assert_vec3_near(&out, &vector3(86.0, 2.0, 2.0), 1e-5);
}

#[test]
fn cg_append_nonuniform_scaling_mut() {
    let mut m = new_scaling(2.0);
    mathlib::append_nonuniform_scaling_mut(&mut m, &vector3(1.0, 2.0, 3.0));
    assert_vec3_near(
        &transform_vector(&m, &vector3(1.0, 1.0, 1.0)),
        &vector3(2.0, 4.0, 6.0),
        1e-5,
    );
}

#[test]
fn cg_from_euler_angles_identity() {
    let m = from_euler_angles(0.0, 0.0, 0.0);
    let p = vector3(1.0, 2.0, 3.0);
    assert_vec3_near(&transform_vector(&m, &p), &p, 1e-5);
}

#[test]
fn cg_look_at_rh_eye_at_origin() {
    let eye = vector3(0.0, 0.0, 5.0);
    let target = vector3(0.0, 0.0, 0.0);
    let up = vector3(0.0, 1.0, 0.0);
    let view = look_at_rh(&eye, &target, &up);
    // Eye in world should map to origin in view space
    let eye_view = transform_point(&view, &eye);
    assert_vec3_near(&eye_view, &vector3(0.0, 0.0, 0.0), 1e-4);
}

#[test]
fn cg_new_perspective_structure() {
    let m = new_perspective(16.0 / 9.0, std::f32::consts::FRAC_PI_2, 1.0, 1000.0);
    assert!(m.get(3, 2) < 0.0); // -1 for perspective divide
    assert_near(m.get(0, 0), m.get(1, 1) * 9.0 / 16.0, 1e-5);
}

#[test]
fn cg_new_orthographic() {
    let m = new_orthographic(-1.0, 1.0, -1.0, 1.0, 0.1, 100.0);
    let p = vector3(0.0, 0.0, -50.0);
    let out = transform_point(&m, &p);
    assert!(out.get(2).abs() < 1.0 + 1e-5);
}

#[test]
fn cg_model_view_projection_shape() {
    let model = new_translation(&vector3(1.0, 0.0, 0.0));
    let eye = vector3(0.0, 0.0, 5.0);
    let view = look_at_rh(&eye, &vector3(0.0, 0.0, 0.0), &vector3(0.0, 1.0, 0.0));
    let proj = new_perspective(16.0 / 9.0, std::f32::consts::FRAC_PI_2, 1.0, 1000.0);
    let mvp = model_view_projection(&model, &view, &proj);
    assert_eq!(mvp.rows(), 4);
    assert_eq!(mvp.cols(), 4);
}

#[test]
fn cg_from_homogeneous_point() {
    let p = vector3(2.0, 3.0, 4.0);
    let h = vector4_from_point(2.0, 3.0, 4.0);
    let recovered = from_homogeneous(&h).unwrap();
    assert_vec3_near(&recovered, &p, 1e-5);
}

#[test]
fn cg_transform_point_vs_homogeneous() {
    let m = new_rotation_wrt_point(
        &vector3(0.0, 0.0, std::f32::consts::FRAC_PI_2),
        &vector3(1.0, 2.0, 1.0),
    );
    let p = vector3(2.0, 3.0, 4.0);
    let t1 = transform_point(&m, &p);
    let h = vector4_from_point(p.get(0), p.get(1), p.get(2));
    let out4 = &m * &h;
    let t2 = from_homogeneous(&out4).unwrap();
    assert_vec3_near(&t1, &t2, 1e-4);
}

#[test]
fn cg_perspective3_unproject() {
    let proj = Perspective3::new(800.0 / 600.0, std::f32::consts::FRAC_PI_2, 1.0, 1000.0);
    let ndc = vector3(0.0, 0.0, -1.0);
    let view_pt = proj.unproject_point(&ndc);
    // Near plane in view space should have z near -1 (or convention)
    assert!(view_pt.get(2).abs() < 10.0);
}

#[test]
fn cg_screen_to_view_ray() {
    let proj = Perspective3::new(800.0 / 600.0, std::f32::consts::FRAC_PI_2, 1.0, 1000.0);
    let (_origin, direction) = mathlib::screen_to_view_ray(&proj, 400.0, 300.0, 800.0, 600.0);
    let n = direction.get(0) * direction.get(0)
        + direction.get(1) * direction.get(1)
        + direction.get(2) * direction.get(2);
    assert_near(n, 1.0, 1e-4); // direction normalized
}

#[test]
fn cg_as_slice_matrix_4x4() {
    let m = new_scaling(2.0);
    let s = m.as_slice();
    assert_eq!(s.len(), 16);
    assert_near(s[0], 2.0, 1e-5);
    assert_near(s[5], 2.0, 1e-5);
    assert_near(s[10], 2.0, 1e-5);
    assert_near(s[15], 1.0, 1e-5);
}

#[test]
fn cg_as_slice_vector() {
    let v = vector3(1.0, 0.0, 1.0);
    let s = v.as_slice();
    assert_eq!(s.len(), 3);
    assert_near(s[0], 1.0, 1e-5);
    assert_near(s[2], 1.0, 1e-5);
}
