use mathlib::{Matrix, SchurError, Storage, schur};

#[test]
fn schur_returns_unimplemented() {
    let mut a = Matrix::with_storage(2, 2, Storage::Column);
    a.set_identity();
    let r = schur(&a);
    assert!(r.is_err());
    assert_eq!(r.unwrap_err(), SchurError::Unimplemented);
}
