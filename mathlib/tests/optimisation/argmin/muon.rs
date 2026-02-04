//! Integration tests for Muon step.

use mathlib::structure::Storage;
use mathlib::{Matrix, muon_step};

#[test]
fn muon_step_integration() {
    let mut param = Matrix::with_storage(2, 2, Storage::Column);
    param.set(0, 0, 1.0);
    param.set(0, 1, 0.0);
    param.set(1, 0, 0.0);
    param.set(1, 1, 1.0);
    let mut grad = Matrix::with_storage(2, 2, Storage::Column);
    grad.set(0, 0, 0.1);
    grad.set(0, 1, 0.0);
    grad.set(1, 0, 0.0);
    grad.set(1, 1, 0.1);
    muon_step(&mut param, &grad, 0.1, 5);
    assert!(param.get(0, 0) < 1.0);
    assert!(param.get(1, 1) < 1.0);
}
