use mathlib::{Lu, Matrix, Storage, Vector, solve};

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
