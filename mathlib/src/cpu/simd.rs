//! SIMD execution backend using the `wide` crate. Enabled with the `simd` feature.

use wide::f64x4;

#[inline]
fn load_f64x4(slice: &[f64], i: usize) -> f64x4 {
    f64x4::new([slice[i], slice[i + 1], slice[i + 2], slice[i + 3]])
}

#[inline]
fn store_f64x4(slice: &mut [f64], i: usize, v: f64x4) {
    let arr = v.to_array();
    slice[i] = arr[0];
    slice[i + 1] = arr[1];
    slice[i + 2] = arr[2];
    slice[i + 3] = arr[3];
}

/// SIMD-assisted dot product for f64 slices (processes 4 lanes at a time, remainder scalar).
#[inline]
pub fn dot_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut sum = 0.0_f64;
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f64x4(a, i * 4);
        let vb = load_f64x4(b, i * 4);
        sum += (va * vb).reduce_add();
    }
    for i in (chunks * 4)..n {
        sum += a[i] * b[i];
    }
    sum
}

/// SIMD set slice to zero (4 lanes at a time).
#[inline]
pub fn set_zero_f64(slice: &mut [f64]) {
    let zero = f64x4::ZERO;
    let n = slice.len();
    let chunks = n / 4;
    for i in 0..chunks {
        store_f64x4(slice, i * 4, zero);
    }
    for i in (chunks * 4)..n {
        slice[i] = 0.0;
    }
}

/// SIMD scalar multiply: out[i] = s * x[i].
#[inline]
pub fn scalar_mul_f64(s: f64, x: &[f64], out: &mut [f64]) {
    assert_eq!(x.len(), out.len());
    let n = x.len();
    let vs = f64x4::splat(s);
    let chunks = n / 4;
    for i in 0..chunks {
        let vx = load_f64x4(x, i * 4);
        store_f64x4(out, i * 4, vs * vx);
    }
    for i in (chunks * 4)..n {
        out[i] = s * x[i];
    }
}

/// SIMD element-wise add: out[i] = a[i] + b[i].
#[inline]
pub fn add_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    let n = a.len();
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f64x4(a, i * 4);
        let vb = load_f64x4(b, i * 4);
        store_f64x4(out, i * 4, va + vb);
    }
    for i in (chunks * 4)..n {
        out[i] = a[i] + b[i];
    }
}

/// SIMD element-wise subtract: out[i] = a[i] - b[i].
#[inline]
pub fn sub_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    let n = a.len();
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f64x4(a, i * 4);
        let vb = load_f64x4(b, i * 4);
        store_f64x4(out, i * 4, va - vb);
    }
    for i in (chunks * 4)..n {
        out[i] = a[i] - b[i];
    }
}

/// SIMD sum of squared differences: sum_i (a[i] - b[i])^2 (4 lanes at a time, remainder scalar).
#[inline]
pub fn squared_diff_sum_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut sum = 0.0_f64;
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f64x4(a, i * 4);
        let vb = load_f64x4(b, i * 4);
        let diff = va - vb;
        sum += (diff * diff).reduce_add();
    }
    for i in (chunks * 4)..n {
        let d = a[i] - b[i];
        sum += d * d;
    }
    sum
}
