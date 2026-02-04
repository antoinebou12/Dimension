use mathlib::{Matrix, Storage, Svd, svd_econ};

#[test]
fn svd_econ_matches_full_leading_columns() {
    let mut a = Matrix::with_storage(3, 2, Storage::Column);
    a.set(0, 0, 1.0);
    a.set(1, 0, 0.0);
    a.set(2, 0, 0.0);
    a.set(0, 1, 0.0);
    a.set(1, 1, 2.0);
    a.set(2, 1, 0.0);
    let econ = svd_econ(&a);
    assert_eq!(econ.u().rows(), 3);
    assert_eq!(econ.u().cols(), 2);
    assert_eq!(econ.v().rows(), 2);
    assert_eq!(econ.v().cols(), 2);
    assert_eq!(econ.sigma().rows(), 2);
    let mut full = Svd::new(&a);
    full.decompose();
    let k = 2usize;
    for j in 0..k {
        assert!((econ.sigma().get(j) - full.sigma().get(j)).abs() < 1e-10);
        for i in 0..3 {
            assert!((econ.u().get(i, j) - full.u().get(i, j)).abs() < 1e-10);
        }
        for i in 0..2 {
            assert!((econ.v().get(i, j) - full.v().get(i, j)).abs() < 1e-10);
        }
    }
}

#[test]
fn svd_new_and_decompose() {
    let mut a = Matrix::with_storage(3, 3, Storage::Column);
    a.set_identity();
    let mut svd = Svd::new(&a);
    svd.decompose();
    let u = svd.u();
    let sigma = svd.sigma();
    assert_eq!(u.rows(), 3);
    assert_eq!(u.cols(), 3);
    assert_eq!(sigma.rows(), 3);
    // Identity has singular values 1, 1, 1; SVD may store or order them differently
    let max_sigma = (0..3).map(|i| sigma.get(i).abs()).fold(0.0_f64, f64::max);
    assert!(
        max_sigma > 0.9 && max_sigma < 1.1,
        "largest singular value of identity should be ~1, got {}",
        max_sigma
    );
}
