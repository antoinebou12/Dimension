use mathlib::{Matrix, Storage, UmapOptions, umap};

#[test]
fn umap_output_shape() {
    let mut data = Matrix::with_storage(50, 5, Storage::Column);
    for i in 0..50 {
        for j in 0..5 {
            data.set(i, j, (i as f64) * 0.1 + (j as f64) * 0.5);
        }
    }
    let opts = UmapOptions {
        n_components: 2,
        n_neighbors: 10,
        max_iters: 50,
        seed: Some(42),
        ..Default::default()
    };
    let embedding = umap(&data, &opts);
    assert_eq!(embedding.rows(), 50);
    assert_eq!(embedding.cols(), 2);
}

#[test]
fn umap_3d_embedding() {
    let mut data = Matrix::with_storage(30, 4, Storage::Column);
    for i in 0..30 {
        for j in 0..4 {
            data.set(i, j, (i as f64) + (j as f64) * 0.2);
        }
    }
    let opts = UmapOptions {
        n_components: 3,
        n_neighbors: 8,
        max_iters: 30,
        seed: Some(123),
        ..Default::default()
    };
    let embedding = umap(&data, &opts);
    assert_eq!(embedding.rows(), 30);
    assert_eq!(embedding.cols(), 3);
}
