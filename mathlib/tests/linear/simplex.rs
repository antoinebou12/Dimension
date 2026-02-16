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

/// Website optimization page examples (row-major A: rows × cols).
#[test]
fn simplex_website_examples() {
    // Example 1: c=[1,1], A=[[1,1],[2,0]], b=[4,2] => x1+x2=4, 2*x1=2 => x1=1, x2=3, obj=4
    let (c1, a1, b1) = (
        Vector::from_slice(&[1.0, 1.0]),
        Matrix::from_vec(&[1.0, 1.0, 2.0, 0.0], 2, 2, Storage::Row),
        Vector::from_slice(&[4.0, 2.0]),
    );
    let r1 = simplex_solve(&c1, &a1, &b1).expect("example 1 should be optimal");
    assert_eq!(r1.status, SimplexStatus::Optimal, "example 1");
    assert!((r1.objective - 4.0).abs() < 1e-6, "example 1 objective");

    // Example 2: c=[2,1], A=[[1,1],[1,0]], b=[6,4] => x1+x2=6, x1=4 => x2=2, obj=10
    let (c2, a2, b2) = (
        Vector::from_slice(&[2.0, 1.0]),
        Matrix::from_vec(&[1.0, 1.0, 1.0, 0.0], 2, 2, Storage::Row),
        Vector::from_slice(&[6.0, 4.0]),
    );
    let r2 = simplex_solve(&c2, &a2, &b2).expect("example 2 should be optimal");
    assert_eq!(r2.status, SimplexStatus::Optimal, "example 2");
    assert!((r2.objective - 10.0).abs() < 1e-6, "example 2 objective");

    // Example 3: c=[2,1,1], A=[[1,2,1],[1,1,0]], b=[6,5]
    let (c3, a3, b3) = (
        Vector::from_slice(&[2.0, 1.0, 1.0]),
        Matrix::from_vec(&[1.0, 2.0, 1.0, 1.0, 1.0, 0.0], 2, 3, Storage::Row),
        Vector::from_slice(&[6.0, 5.0]),
    );
    let r3 = simplex_solve(&c3, &a3, &b3).expect("example 3 should be optimal");
    assert_eq!(r3.status, SimplexStatus::Optimal, "example 3");
    assert!(
        r3.objective >= 9.0 - 1e-6 && r3.objective <= 11.0 + 1e-6,
        "example 3 objective"
    );

    // Same examples as the website: WasmMatrix.fromArray uses column-major storage.
    // Example 1: [1,1,2,0] 2x2 => col0=[1,1], col1=[2,0] => A=[[1,2],[1,0]], b=[4,2]
    let a1_col = Matrix::from_vec(&[1.0, 1.0, 2.0, 0.0], 2, 2, Storage::Column);
    let r1w = simplex_solve(&c1, &a1_col, &b1).expect("website example 1");
    assert_eq!(r1w.status, SimplexStatus::Optimal, "website ex1");
    // Example 2: [1,1,1,0] 2x2 => A=[[1,1],[1,0]]
    let a2_col = Matrix::from_vec(&[1.0, 1.0, 1.0, 0.0], 2, 2, Storage::Column);
    let r2w = simplex_solve(&c2, &a2_col, &b2).expect("website example 2");
    assert_eq!(r2w.status, SimplexStatus::Optimal, "website ex2");
    // Example 3: [1,2,1,1,1,0] 2x3 => col0=[1,2], col1=[1,1], col2=[1,0] => A=[[1,1,1],[2,1,0]]
    let a3_col = Matrix::from_vec(&[1.0, 2.0, 1.0, 1.0, 1.0, 0.0], 2, 3, Storage::Column);
    let r3w = simplex_solve(&c3, &a3_col, &b3).expect("website example 3");
    assert_eq!(r3w.status, SimplexStatus::Optimal, "website ex3");
}
