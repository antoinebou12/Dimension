use mathlib::{DenseStorage, DenseStorageDynamic};

#[test]
fn dense_storage_fixed() {
    let buf: DenseStorage<f32, 4> = DenseStorage::new();
    assert_eq!(buf.size(), 4);
    let mut buf2: DenseStorage<f32, 4> = DenseStorage::new();
    buf2.resize(22);
    assert_eq!(buf2.size(), 4);
}

#[test]
fn dense_storage_fixed_zero() {
    let buf: DenseStorage<f32, 0> = DenseStorage::new();
    assert_eq!(buf.size(), 0);
}

#[test]
fn dense_storage_fixed_read_write_copy() {
    let mut buf: DenseStorage<f32, 4> = DenseStorage::new();
    buf[0] = 1.0f32;
    buf[1] = 2.0f32;
    buf[2] = 3.2f32;
    buf[3] = -1.0f32;
    assert!((buf[0] - 1.0f32).abs() < 1e-9);
    assert!((buf[1] - 2.0f32).abs() < 1e-9);
    assert!((buf[2] - 3.2f32).abs() < 1e-9);
    assert!((buf[3] + 1.0f32).abs() < 1e-9);
    let buf2: DenseStorage<f32, 4> = DenseStorage::from_slice(buf.data());
    assert!((buf2[0] - 1.0f32).abs() < 1e-9);
    assert!((buf2[3] + 1.0f32).abs() < 1e-9);
}

#[test]
fn dense_storage_dynamic() {
    let mut buf: DenseStorageDynamic<f32> = DenseStorageDynamic::with_capacity(8);
    assert_eq!(buf.size(), 8);
    buf.resize(102);
    assert_eq!(buf.size(), 102);
}

#[test]
fn dense_storage_dynamic_zero() {
    let buf: DenseStorageDynamic<f32> = DenseStorageDynamic::with_capacity(0);
    assert_eq!(buf.size(), 0);
}

#[test]
fn dense_storage_dynamic_read_write_copy() {
    let mut buf: DenseStorageDynamic<f32> = DenseStorageDynamic::with_capacity(4);
    buf[0] = 1.0f32;
    buf[1] = 2.0f32;
    buf[2] = 3.2f32;
    buf[3] = -1.0f32;
    assert!((buf[0] - 1.0f32).abs() < 1e-9);
    let buf2 = DenseStorageDynamic::from_slice(buf.data());
    assert!((buf2[2] - 3.2f32).abs() < 1e-9);
}
