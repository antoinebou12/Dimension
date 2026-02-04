use mathlib::{Matrix, QzError, Storage, qz};

#[test]
fn qz_returns_unimplemented() {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    let mut b = Matrix::with_storage(2, 2, Storage::Column);
    a.set_identity();
    b.set_identity();
    let r = qz(&a, &b);
    assert!(r.is_err());
    assert_eq!(r.unwrap_err(), QzError::Unimplemented);
}
