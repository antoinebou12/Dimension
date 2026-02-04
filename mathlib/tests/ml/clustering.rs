use mathlib::{Matrix, NOISE, Storage, dbscan, kmeans};

#[test]
fn kmeans_two_clusters() {
    let mut data = Matrix::with_storage(6, 2, Storage::Column);
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(1, 0, 1.0);
    data.set(1, 1, 0.0);
    data.set(2, 0, 2.0);
    data.set(2, 1, 0.0);
    data.set(3, 0, 10.0);
    data.set(3, 1, 10.0);
    data.set(4, 0, 11.0);
    data.set(4, 1, 10.0);
    data.set(5, 0, 12.0);
    data.set(5, 1, 10.0);
    let result = kmeans(&data, 2, Some(20));
    assert_eq!(result.labels().len(), 6);
    assert_eq!(result.n_clusters(), 2);
    assert_eq!(result.centroids().rows(), 2);
    assert_eq!(result.centroids().cols(), 2);
    for &l in result.labels() {
        assert!(l < 2);
    }
}

#[test]
fn kmeans_labels_in_range() {
    let mut data = Matrix::with_storage(10, 3, Storage::Column);
    for i in 0..10 {
        for j in 0..3 {
            data.set(i, j, (i as f64) * 0.5 + (j as f64));
        }
    }
    let result = kmeans(&data, 3, Some(15));
    assert_eq!(result.labels().len(), 10);
    for &l in result.labels() {
        assert!(l < 3);
    }
}

#[test]
fn dbscan_two_blobs_one_noise() {
    let mut data = Matrix::with_storage(7, 2, Storage::Column);
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(1, 0, 0.1);
    data.set(1, 1, 0.0);
    data.set(2, 0, 0.0);
    data.set(2, 1, 0.1);
    data.set(3, 0, 5.0);
    data.set(3, 1, 5.0);
    data.set(4, 0, 5.1);
    data.set(4, 1, 5.0);
    data.set(5, 0, 5.0);
    data.set(5, 1, 5.1);
    data.set(6, 0, 100.0);
    data.set(6, 1, 100.0);
    let result = dbscan(&data, 1.0, 2);
    assert_eq!(result.labels().len(), 7);
    assert!(result.n_clusters() >= 1 && result.n_clusters() <= 3);
    let noise_count = result.labels().iter().filter(|&&l| l == NOISE).count();
    assert!(noise_count >= 1);
}

#[test]
fn dbscan_noise_constant() {
    let mut data = Matrix::with_storage(3, 2, Storage::Column);
    data.set(0, 0, 0.0);
    data.set(0, 1, 0.0);
    data.set(1, 0, 100.0);
    data.set(1, 1, 100.0);
    data.set(2, 0, 200.0);
    data.set(2, 1, 200.0);
    let result = dbscan(&data, 1.0, 2);
    assert!(result.is_noise(0) || result.is_noise(1) || result.is_noise(2));
}
