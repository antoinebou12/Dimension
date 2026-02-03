use mathlib::{Matrix, Storage, pca};

#[test]
fn pca_dimensions() {
    let mut data = Matrix::with_storage(10, 4, Storage::Column);
    for i in 0..10 {
        for j in 0..4 {
            data.set(i, j, (i as f64) * 0.5 + (j as f64));
        }
    }
    let result = pca(&data, None);
    assert_eq!(result.mean().rows(), 4);
    assert_eq!(result.components().rows(), 4);
    assert_eq!(result.components().cols(), 4);
    assert_eq!(result.explained_variance().rows(), 4);
    assert_eq!(result.n_components(), 4);
}

#[test]
fn pca_n_components_limit() {
    let mut data = Matrix::with_storage(20, 8, Storage::Column);
    for i in 0..20 {
        for j in 0..8 {
            data.set(i, j, (i as f64) + (j as f64) * 0.3);
        }
    }
    let result = pca(&data, Some(3));
    assert_eq!(result.n_components(), 3);
    assert_eq!(result.components().cols(), 3);
    assert_eq!(result.explained_variance().rows(), 3);
}

#[test]
fn pca_centers_data() {
    let mut data = Matrix::with_storage(5, 2, Storage::Column);
    data.set(0, 0, 1.0);
    data.set(1, 0, 2.0);
    data.set(2, 0, 3.0);
    data.set(3, 0, 4.0);
    data.set(4, 0, 5.0);
    data.set(0, 1, 10.0);
    data.set(1, 1, 20.0);
    data.set(2, 1, 30.0);
    data.set(3, 1, 40.0);
    data.set(4, 1, 50.0);
    let result = pca(&data, None);
    let mean = result.mean();
    assert!((mean.get(0) - 3.0).abs() < 1e-10);
    assert!((mean.get(1) - 30.0).abs() < 1e-10);
}

#[test]
fn pca_explained_variance_ordering() {
    let mut data = Matrix::with_storage(50, 5, Storage::Column);
    for i in 0..50 {
        data.set(i, 0, (i as f64) * 2.0);
        for j in 1..5 {
            data.set(i, j, (i as f64) * 0.1 + (j as f64) * 0.01);
        }
    }
    let result = pca(&data, None);
    let ev = result.explained_variance();
    for i in 1..result.n_components() {
        assert!(
            ev.get(i) <= ev.get(i - 1) + 1e-10,
            "explained variance should be non-increasing: {} > {}",
            ev.get(i),
            ev.get(i - 1)
        );
    }
}

#[test]
fn pca_components_orthonormal() {
    let mut data = Matrix::with_storage(30, 4, Storage::Column);
    for i in 0..30 {
        for j in 0..4 {
            data.set(i, j, (i as f64) * 0.2 - (j as f64) * 0.5);
        }
    }
    let result = pca(&data, None);
    let comp = result.components();
    let n = result.n_components();
    for a in 0..n {
        let mut norm = 0.0;
        for i in 0..comp.rows() {
            let v = comp.get(i, a);
            norm += v * v;
        }
        assert!(
            (norm - 1.0).abs() < 1e-8,
            "component {} should have unit norm, got {}",
            a,
            norm
        );
        for b in (a + 1)..n {
            let mut dot = 0.0;
            for i in 0..comp.rows() {
                dot += comp.get(i, a) * comp.get(i, b);
            }
            assert!(
                dot.abs() < 1e-8,
                "components {} and {} should be orthogonal, dot={}",
                a,
                b,
                dot
            );
        }
    }
}

#[test]
fn pca_single_component() {
    let mut data = Matrix::with_storage(10, 3, Storage::Column);
    for i in 0..10 {
        for j in 0..3 {
            data.set(i, j, (i as f64) + (j as f64));
        }
    }
    let result = pca(&data, Some(1));
    assert_eq!(result.n_components(), 1);
    assert_eq!(result.components().rows(), 3);
    assert_eq!(result.components().cols(), 1);
}
