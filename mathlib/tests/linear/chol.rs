use mathlib::{Cholesky, Matrix, Storage, Vector, chol};

fn make_spd_2x2() -> Matrix<f64> {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set(0, 0, 4.0);
    a.set(0, 1, 1.0);
    a.set(1, 0, 1.0);
    a.set(1, 1, 3.0);
    a
}

#[test]
fn chol_new_and_solve() {
    let a = make_spd_2x2();
    let ch = Cholesky::new(&a).unwrap();
    assert_eq!(ch.size(), 2);
    let l = ch.l();
    assert_eq!(l.rows(), 2);
    assert_eq!(l.cols(), 2);
    // Check L L^T ≈ A
    let mut llt: Matrix<f64> = Matrix::with_storage(2, 2, Storage::Column);
    llt.set_zero();
    for i in 0..2 {
        for j in 0..2 {
            for k in 0..2 {
                llt.set(i, j, llt.get(i, j) + l.get(i, k) * l.get(j, k));
            }
        }
    }
    for i in 0..2 {
        for j in 0..2 {
            let diff: f64 = llt.get(i, j) - a.get(i, j);
            assert!(diff.abs() < 1e-10, "L L^T should equal A");
        }
    }
}

#[test]
fn chol_solve() {
    let a = make_spd_2x2();
    let ch = Cholesky::new(&a).unwrap();
    let mut b = Vector::with_capacity(2);
    b.set(0, 1.0);
    b.set(1, 2.0);
    let x = ch.solve(&b);
    // A x = b => check residual
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
fn chol_free_function() {
    let a = make_spd_2x2();
    let ch = chol(&a).unwrap();
    assert_eq!(ch.size(), 2);
}

#[test]
fn chol_not_square() {
    let mut a = Matrix::with_storage(2, 3, Storage::Column);
    a.set_zero();
    let r = Cholesky::new(&a);
    assert!(r.is_err());
}
