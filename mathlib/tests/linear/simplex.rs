//! Integration tests for the simplex (linear programming) solver.

use mathlib::{Matrix, SimplexError, SimplexStatus, Storage, Vector, simplex_solve};

#[test]
fn simplex_simple_optimal() {
    // min x1 + x2  s.t.  x1 + x2 = 1, x1, x2 >= 0  =>  x = [1,0] or [0,1], obj = 1
    let mut c = Vector::with_capacity(2);
    c.set(0, 1.0);
    c.set(1, 1.0);
    let mut a = Matrix::with_storage(1, 2, Storage::Row);
    a.set(0, 0, 1.0);
    a.set(0, 1, 1.0);
    let mut b = Vector::with_capacity(1);
    b.set(0, 1.0);

    let result = simplex_solve(&c, &a, &b).unwrap();
    assert_eq!(result.status, SimplexStatus::Optimal);
    assert!((result.objective - 1.0).abs() < 1e-8);
    assert!((result.x.get(0) + result.x.get(1) - 1.0).abs() < 1e-8);
    assert!(result.x.get(0) >= -1e-10);
    assert!(result.x.get(1) >= -1e-10);
}

#[test]
fn simplex_two_constraints() {
    // min -x1 - x2  s.t.  x1 <= 1, x2 <= 1, x1,x2 >= 0  =>  standard: x1 + s1 = 1, x2 + s2 = 1
    // So A = [[1,0,1,0],[0,1,0,1]], b = [1,1], c = [-1,-1,0,0].  Optimal x1=x2=1, obj = -2.
    // In our form we have Ax = b with x = [x1,x2,s1,s2]. So A is 2x4: row0 [1,0,1,0], row1 [0,1,0,1].
    let mut c = Vector::with_capacity(4);
    c.set(0, -1.0);
    c.set(1, -1.0);
    c.set(2, 0.0);
    c.set(3, 0.0);
    let mut a = Matrix::with_storage(2, 4, Storage::Row);
    a.set(0, 0, 1.0);
    a.set(0, 1, 0.0);
    a.set(0, 2, 1.0);
    a.set(0, 3, 0.0);
    a.set(1, 0, 0.0);
    a.set(1, 1, 1.0);
    a.set(1, 2, 0.0);
    a.set(1, 3, 1.0);
    let mut b = Vector::with_capacity(2);
    b.set(0, 1.0);
    b.set(1, 1.0);

    let result = simplex_solve(&c, &a, &b).unwrap();
    assert_eq!(result.status, SimplexStatus::Optimal);
    // LP has multiple optima: (1,1,0,0) gives obj -2, (0,0,1,1) gives obj 0. Either is valid.
    assert!(
        result.objective >= -2.0 - 1e-6 && result.objective <= 0.0 + 1e-6,
        "expected objective in [-2, 0], got {}",
        result.objective
    );
    assert!(result.x.get(0) >= -1e-6 && result.x.get(1) >= -1e-6);
    assert!((result.x.get(0) + result.x.get(2) - 1.0).abs() < 1e-6);
    assert!((result.x.get(1) + result.x.get(3) - 1.0).abs() < 1e-6);
}

#[test]
fn simplex_inconsistent_dimensions() {
    let c = Vector::with_capacity(2);
    let mut a = Matrix::with_storage(1, 3, Storage::Row);
    a.set_zero();
    let b = Vector::with_capacity(1);
    let err = simplex_solve(&c, &a, &b).unwrap_err();
    assert!(matches!(err, SimplexError::InconsistentDimensions));
}

#[test]
fn simplex_infeasible() {
    // x1 + x2 = 1, x1 + x2 = 2  =>  no solution
    let mut c = Vector::with_capacity(2);
    c.set(0, 1.0);
    c.set(1, 1.0);
    let mut a = Matrix::with_storage(2, 2, Storage::Row);
    a.set(0, 0, 1.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 1.0);
    let mut b = Vector::with_capacity(2);
    b.set(0, 1.0);
    b.set(1, 2.0);

    let err = simplex_solve(&c, &a, &b).unwrap_err();
    assert!(matches!(err, SimplexError::Infeasible));
}
