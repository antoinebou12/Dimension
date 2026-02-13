use mathlib::{Lu, Matrix, SolveError, Storage, Vector, damped_least_squares, solve};

fn make_2x2() -> Matrix<f64> {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 2.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 2.0);
    a
}

#[test]
fn solve_2x2() {
    let a = make_2x2();
    let mut b = Vector::with_capacity(2);
    b.set(0, 3.0);
    b.set(1, 3.0);
    let x = solve(&a, &b).unwrap();
    // A x = b
    let mut ax = Vector::with_capacity(2);
    ax.set_zero();
    for i in 0..2 {
        let mut s = 0.0;
        for j in 0..2 {
            s += a.get(i, j) * x.get(j);
        }
        ax.set(i, s);
    }
    assert!((ax.get(0) - b.get(0)).abs() < 1e-10);
    assert!((ax.get(1) - b.get(1)).abs() < 1e-10);
}

#[test]
fn lu_solve() {
    let a = make_2x2();
    let lu = Lu::new(&a).unwrap();
    let mut b = Vector::with_capacity(2);
    b.set(0, 1.0);
    b.set(1, 2.0);
    let x = lu.solve(&b);
    let mut ax = Vector::with_capacity(2);
    ax.set_zero();
    for i in 0..2 {
        let mut s = 0.0;
        for j in 0..2 {
            s += a.get(i, j) * x.get(j);
        }
        ax.set(i, s);
    }
    assert!((ax.get(0) - b.get(0)).abs() < 1e-10);
    assert!((ax.get(1) - b.get(1)).abs() < 1e-10);
}

#[test]
fn solve_1x1() {
    let mut a = Matrix::with_storage(1, 1, Storage::Column);
    a.set(0, 0, 2.0);
    let mut b = Vector::with_capacity(1);
    b.set(0, 4.0);
    let x = solve(&a, &b).unwrap();
    assert!((x.get(0) - 2.0).abs() < 1e-10);
}

#[test]
fn solve_3x3_residual_norm() {
    let mut a = Matrix::with_storage(3, 3, Storage::Column);
    a.set(0, 0, 3.0);
    a.set(0, 1, 0.0);
    a.set(0, 2, 0.0);
    a.set(1, 0, 0.0);
    a.set(1, 1, 2.0);
    a.set(1, 2, 0.0);
    a.set(2, 0, 0.0);
    a.set(2, 1, 0.0);
    a.set(2, 2, 1.0);
    let mut b = Vector::with_capacity(3);
    b.set(0, 6.0);
    b.set(1, 4.0);
    b.set(2, 1.0);
    let x = solve(&a, &b).unwrap();
    let mut ax = Vector::with_capacity(3);
    ax.set_zero();
    for i in 0..3 {
        let mut s = 0.0;
        for j in 0..3 {
            s += a.get(i, j) * x.get(j);
        }
        ax.set(i, s);
    }
    let mut res_sq = 0.0;
    for i in 0..3 {
        let r = ax.get(i) - b.get(i);
        res_sq += r * r;
    }
    assert!(
        res_sq.sqrt() < 1e-9,
        "||Ax - b|| = {} should be < 1e-9",
        res_sq.sqrt()
    );
}

#[test]
fn solve_singular_3x3() {
    let mut a = Matrix::with_storage(3, 3, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 1.0);
    a.set(0, 2, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 1.0);
    a.set(1, 2, 1.0);
    a.set(2, 0, 1.0);
    a.set(2, 1, 1.0);
    a.set(2, 2, 1.0);
    let mut b = Vector::with_capacity(3);
    b.set(0, 1.0);
    b.set(1, 2.0);
    b.set(2, 3.0);
    let err = solve(&a, &b).unwrap_err();
    assert!(matches!(err, SolveError::Singular));
}

#[test]
fn solve_not_square() {
    let mut a = Matrix::with_storage(2, 3, Storage::Column);
    a.set_zero();
    let b = Vector::with_capacity(2);
    let err = solve(&a, &b).unwrap_err();
    assert!(matches!(err, SolveError::NotSquare));
}

#[test]
fn solve_singular() {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 1.0);
    let mut b = Vector::with_capacity(2);
    b.set(0, 1.0);
    b.set(1, 2.0);
    let err = solve(&a, &b).unwrap_err();
    assert!(matches!(err, SolveError::Singular));
}

#[test]
fn solve_error_display() {
    let mut a = Matrix::with_storage(2, 3, Storage::Column);
    a.set_zero();
    let b = Vector::with_capacity(2);
    let err = solve(&a, &b).unwrap_err();
    let s = format!("{}", err);
    assert!(s.contains("square"));

    let mut a2 = Matrix::with_storage(2, 2, Storage::Column);
    a2.set(0, 0, 1.0);
    a2.set(0, 1, 1.0);
    a2.set(1, 0, 1.0);
    a2.set(1, 1, 1.0);
    let mut b2 = Vector::with_capacity(2);
    b2.set(0, 1.0);
    b2.set(1, 2.0);
    let err2 = solve(&a2, &b2).unwrap_err();
    let s2 = format!("{}", err2);
    assert!(s2.contains("singular"));
}

#[test]
fn damped_least_squares_matches_normal_equations() {
    let mut a = Matrix::with_storage(3, 2, Storage::Column);
    // Column 0
    a.set(0, 0, 1.0);
    a.set(1, 0, 0.0);
    a.set(2, 0, 1.0);
    // Column 1
    a.set(0, 1, 0.0);
    a.set(1, 1, 1.0);
    a.set(2, 1, 1.0);

    let mut b = Vector::with_capacity(3);
    b.set(0, 1.0);
    b.set(1, 2.0);
    b.set(2, 3.0);

    // With λ² = 0 the normal equations A Aᵀ y = b are singular (3×2 A has rank ≤ 2). Use small λ².
    let x = damped_least_squares(&a, &b, 1e-6).unwrap();
    assert!((x.get(0) - 1.0).abs() < 1e-4);
    assert!((x.get(1) - 2.0).abs() < 1e-4);
}

#[test]
fn damping_reduces_step_magnitude() {
    let mut a = Matrix::with_storage(2, 1, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(1, 0, 1.0);

    let mut b = Vector::with_capacity(2);
    b.set(0, 10.0);
    b.set(1, 10.0);

    // A Aᵀ is 2×2 with rows [1,1] and [1,1], so singular when λ² = 0. Use small vs large λ².
    let x_small_damp = damped_least_squares(&a, &b, 0.01).unwrap();
    let x_large_damp = damped_least_squares(&a, &b, 100.0).unwrap();
    assert!(x_large_damp.get(0).abs() < x_small_damp.get(0).abs());
}

#[test]
fn damped_least_squares_underdetermined() {
    let mut a = Matrix::with_storage(2, 3, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 0.0);
    a.set(0, 2, 1.0);
    a.set(1, 0, 0.0);
    a.set(1, 1, 1.0);
    a.set(1, 2, 1.0);
    let mut b = Vector::with_capacity(2);
    b.set(0, 2.0);
    b.set(1, 2.0);
    let x = damped_least_squares(&a, &b, 0.01).unwrap();
    assert_eq!(x.rows(), 3);
    let mut ax = Vector::with_capacity(2);
    ax.set_zero();
    for i in 0..2 {
        let mut s = 0.0;
        for j in 0..3 {
            s += a.get(i, j) * x.get(j);
        }
        ax.set(i, s);
    }
    let res_sq = (ax.get(0) - b.get(0)).powi(2) + (ax.get(1) - b.get(1)).powi(2);
    assert!(res_sq < 0.1, "residual squared {} should be small", res_sq);
}

#[test]
fn damped_least_squares_damping_reduces_norm() {
    let mut a = Matrix::with_storage(3, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(0, 1, 0.0);
    a.set(1, 0, 0.0);
    a.set(1, 1, 1.0);
    a.set(2, 0, 1.0);
    a.set(2, 1, 1.0);
    let mut b = Vector::with_capacity(3);
    b.set(0, 1.0);
    b.set(1, 1.0);
    b.set(2, 2.0);
    // A Aᵀ is singular for 3×2 when λ² = 0; use small vs larger λ².
    let x_small_damp = damped_least_squares(&a, &b, 0.01).unwrap();
    let x_large_damp = damped_least_squares(&a, &b, 1.0).unwrap();
    let norm_small = (x_small_damp.get(0).powi(2) + x_small_damp.get(1).powi(2)).sqrt();
    let norm_large = (x_large_damp.get(0).powi(2) + x_large_damp.get(1).powi(2)).sqrt();
    assert!(
        norm_large <= norm_small + 1e-10,
        "larger damping should not increase solution norm: {} vs {}",
        norm_large,
        norm_small
    );
}
