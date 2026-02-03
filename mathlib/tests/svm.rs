use mathlib::{Matrix, Storage, SvmError, svm, svm_rbf};

#[test]
fn svm_separable_two_classes() {
    let mut x = Matrix::with_storage(6, 2, Storage::Column);
    x.set(0, 0, 1.0);
    x.set(0, 1, 2.0);
    x.set(1, 0, 2.0);
    x.set(1, 1, 3.0);
    x.set(2, 0, 2.0);
    x.set(2, 1, 2.0);
    x.set(3, 0, 0.0);
    x.set(3, 1, 0.0);
    x.set(4, 0, 1.0);
    x.set(4, 1, 0.0);
    x.set(5, 0, 0.0);
    x.set(5, 1, 1.0);
    let y = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0];
    let result = svm(&x, &y, None).unwrap();
    let pred = result.predict(&x);
    for (i, &label) in y.iter().enumerate() {
        assert_eq!(
            pred[i], label,
            "sample {} predicted {} expected {}",
            i, pred[i], label
        );
    }
}

#[test]
fn svm_error_label_length() {
    let mut x = Matrix::with_storage(3, 2, Storage::Column);
    x.set(0, 0, 1.0);
    x.set(0, 1, 0.0);
    x.set(1, 0, 0.0);
    x.set(1, 1, 1.0);
    x.set(2, 0, 1.0);
    x.set(2, 1, 1.0);
    let y = [1.0, -1.0]; // length 2, x has 3 rows
    let err = svm(&x, &y, None).unwrap_err();
    assert!(matches!(err, SvmError::LabelLength));
}

#[test]
fn svm_error_single_class() {
    let mut x = Matrix::with_storage(3, 2, Storage::Column);
    x.set(0, 0, 1.0);
    x.set(0, 1, 0.0);
    x.set(1, 0, 0.0);
    x.set(1, 1, 1.0);
    x.set(2, 0, 1.0);
    x.set(2, 1, 1.0);
    let y = [1.0, 1.0, 1.0];
    let err = svm(&x, &y, None).unwrap_err();
    assert!(matches!(err, SvmError::SingleClass));
}

#[test]
fn svm_error_empty_data() {
    let x = Matrix::with_storage(0, 2, Storage::Column);
    let y: [f64; 0] = [];
    let err = svm(&x, &y, None).unwrap_err();
    assert!(matches!(err, SvmError::EmptyData));

    let x_no_cols = Matrix::with_storage(3, 0, Storage::Column);
    let y_no_cols = [1.0, -1.0, 1.0];
    let err2 = svm(&x_no_cols, &y_no_cols, None).unwrap_err();
    assert!(matches!(err2, SvmError::EmptyData));
}

// --- RBF ---

#[test]
fn svm_rbf_separable_two_classes() {
    let mut x = Matrix::with_storage(6, 2, Storage::Column);
    x.set(0, 0, 1.0);
    x.set(0, 1, 2.0);
    x.set(1, 0, 2.0);
    x.set(1, 1, 3.0);
    x.set(2, 0, 2.0);
    x.set(2, 1, 2.0);
    x.set(3, 0, 0.0);
    x.set(3, 1, 0.0);
    x.set(4, 0, 1.0);
    x.set(4, 1, 0.0);
    x.set(5, 0, 0.0);
    x.set(5, 1, 1.0);
    let y = [1.0, 1.0, 1.0, -1.0, -1.0, -1.0];
    let result = svm_rbf(&x, &y, 0.5, None).unwrap();
    let pred = result.predict(&x);
    for (i, &label) in y.iter().enumerate() {
        assert_eq!(
            pred[i], label,
            "sample {} predicted {} expected {}",
            i, pred[i], label
        );
    }
}

#[test]
fn svm_rbf_error_label_length() {
    let mut x = Matrix::with_storage(3, 2, Storage::Column);
    x.set(0, 0, 1.0);
    x.set(0, 1, 0.0);
    x.set(1, 0, 0.0);
    x.set(1, 1, 1.0);
    x.set(2, 0, 1.0);
    x.set(2, 1, 1.0);
    let y = [1.0, -1.0];
    let err = svm_rbf(&x, &y, 0.5, None).unwrap_err();
    assert!(matches!(err, SvmError::LabelLength));
}

#[test]
fn svm_rbf_error_single_class() {
    let mut x = Matrix::with_storage(3, 2, Storage::Column);
    x.set(0, 0, 1.0);
    x.set(0, 1, 0.0);
    x.set(1, 0, 0.0);
    x.set(1, 1, 1.0);
    x.set(2, 0, 1.0);
    x.set(2, 1, 1.0);
    let y = [1.0, 1.0, 1.0];
    let err = svm_rbf(&x, &y, 0.5, None).unwrap_err();
    assert!(matches!(err, SvmError::SingleClass));
}

#[test]
fn svm_rbf_error_empty_data() {
    let x = Matrix::with_storage(0, 2, Storage::Column);
    let y: [f64; 0] = [];
    let err = svm_rbf(&x, &y, 0.5, None).unwrap_err();
    assert!(matches!(err, SvmError::EmptyData));
}
