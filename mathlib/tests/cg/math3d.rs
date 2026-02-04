use mathlib::{
    Matrix4f, Vector3f, make_rotation, matrix3f_inverse, matrix4_mul_vector3, matrix4f_inverse,
    transform_vector,
};

#[test]
fn matrix4f_identity() {
    let mut m = Matrix4f::with_dimensions(4, 4);
    m.set_zero();
    for i in 0..4 {
        m.set(i, i, 1.0f32);
    }
    assert!((m.get(0, 0) - 1.0f32).abs() < 1e-9);
    assert!((m.get(3, 3) - 1.0f32).abs() < 1e-9);
}

#[test]
fn matrix3f_inverse_rotation() {
    let m = make_rotation(0.0, 0.0, 0.0);
    let inv = matrix3f_inverse(&m);
    let prod = &m * &inv;
    for i in 0..3 {
        for j in 0..3 {
            let expected = if i == j { 1.0f32 } else { 0.0f32 };
            assert!((prod.get(i, j) - expected).abs() < 1e-5);
        }
    }
}

#[test]
fn test_matrix4_mul_vector3() {
    let mut m = Matrix4f::with_dimensions(4, 4);
    m.set_identity();
    m.set(0, 3, 1.0f32);
    m.set(1, 3, 2.0f32);
    m.set(2, 3, 3.0f32);
    let mut v = Vector3f::with_capacity(3);
    v.set(0, 0.0f32);
    v.set(1, 0.0f32);
    v.set(2, 0.0f32);
    let out = matrix4_mul_vector3(&m, &v);
    assert!((out.get(0) - 1.0f32).abs() < 1e-9);
    assert!((out.get(1) - 2.0f32).abs() < 1e-9);
    assert!((out.get(2) - 3.0f32).abs() < 1e-9);
}

#[test]
fn make_rotation_identity() {
    let r = make_rotation(0.0, 0.0, 0.0);
    assert!((r.get(0, 0) - 1.0).abs() < 1e-5);
    assert!((r.get(1, 1) - 1.0).abs() < 1e-5);
    assert!((r.get(2, 2) - 1.0).abs() < 1e-5);
}

#[test]
fn transform_vector_no_translation() {
    let mut m = Matrix4f::with_dimensions(4, 4);
    m.set_identity();
    m.set(0, 3, 10.0);
    m.set(1, 3, 20.0);
    m.set(2, 3, 30.0);
    let mut v = Vector3f::with_capacity(3);
    v.set(0, 1.0);
    v.set(1, 0.0);
    v.set(2, 0.0);
    let out = transform_vector(&m, &v);
    assert!((out.get(0) - 1.0).abs() < 1e-9);
    assert!((out.get(1) - 0.0).abs() < 1e-9);
    assert!((out.get(2) - 0.0).abs() < 1e-9);
}

#[test]
fn matrix4f_inverse_roundtrip() {
    let mut m = Matrix4f::with_dimensions(4, 4);
    m.set_identity();
    m.set(0, 3, 1.0);
    m.set(1, 3, 2.0);
    m.set(2, 3, 3.0);
    let r = make_rotation(0.1, 0.2, 0.3);
    for i in 0..3 {
        for j in 0..3 {
            m.set(i, j, r.get(i, j));
        }
    }
    let inv = matrix4f_inverse(&m);
    let prod = &m * &inv;
    for i in 0..4 {
        for j in 0..4 {
            let expected = if i == j { 1.0f32 } else { 0.0f32 };
            assert!((prod.get(i, j) - expected).abs() < 1e-5);
        }
    }
}

#[test]
fn make_rotation_half_pi_x() {
    let r = make_rotation(std::f32::consts::FRAC_PI_2, 0.0, 0.0);
    let mut y_axis = Vector3f::with_capacity(3);
    y_axis.set(0, 0.0);
    y_axis.set(1, 1.0);
    y_axis.set(2, 0.0);
    let out = transform_vector(
        &{
            let mut m4 = Matrix4f::with_dimensions(4, 4);
            m4.set_identity();
            for i in 0..3 {
                for j in 0..3 {
                    m4.set(i, j, r.get(i, j));
                }
            }
            m4
        },
        &y_axis,
    );
    assert!((out.get(0) - 0.0).abs() < 1e-5);
    assert!((out.get(1) - 0.0).abs() < 1e-5);
    assert!((out.get(2) - 1.0).abs() < 1e-5);
}
