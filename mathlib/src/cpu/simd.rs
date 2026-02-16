//! SIMD execution backend using the `wide` crate. Enabled with the `simd` feature.

use wide::{f32x4, f64x4};

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

/// SIMD sum of squares: sum_i x[i]² (4 lanes at a time, remainder scalar).
#[inline]
pub fn squared_sum_f64(x: &[f64]) -> f64 {
    let n = x.len();
    let mut sum = 0.0_f64;
    let chunks = n / 4;
    for i in 0..chunks {
        let vx = load_f64x4(x, i * 4);
        sum += (vx * vx).reduce_add();
    }
    for i in (chunks * 4)..n {
        let v = x[i];
        sum += v * v;
    }
    sum
}

/// SIMD sum of absolute differences: sum_i |a[i] - b[i]|.
#[inline]
pub fn abs_diff_sum_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut sum = 0.0_f64;
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f64x4(a, i * 4);
        let vb = load_f64x4(b, i * 4);
        let diff = (va - vb).abs();
        sum += diff.reduce_add();
    }
    for i in (chunks * 4)..n {
        sum += (a[i] - b[i]).abs();
    }
    sum
}

/// SIMD max of absolute differences: max_i |a[i] - b[i]|.
#[inline]
pub fn max_abs_diff_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut max_val = 0.0_f64;
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f64x4(a, i * 4);
        let vb = load_f64x4(b, i * 4);
        let diff = (va - vb).abs();
        let arr = diff.to_array();
        for &v in &arr {
            if v > max_val {
                max_val = v;
            }
        }
    }
    for i in (chunks * 4)..n {
        let d = (a[i] - b[i]).abs();
        if d > max_val {
            max_val = d;
        }
    }
    max_val
}

/// Batch RBF kernel: out[i] = exp(-gamma * dist_sq[i]). Uses scalar exp per element
/// (wide does not provide f64x4::exp); keeps batch API ready for future vectorized exp.
#[inline]
pub fn rbf_kernel_batch_f64(gamma: f64, dist_sq: &[f64], out: &mut [f64]) {
    assert_eq!(dist_sq.len(), out.len());
    for (i, &d) in dist_sq.iter().enumerate() {
        out[i] = (-gamma * d).exp();
    }
}

// --- f32 SIMD ---

#[inline]
fn load_f32x4(slice: &[f32], i: usize) -> f32x4 {
    f32x4::new([slice[i], slice[i + 1], slice[i + 2], slice[i + 3]])
}

#[inline]
fn store_f32x4(slice: &mut [f32], i: usize, v: f32x4) {
    let arr = v.to_array();
    slice[i] = arr[0];
    slice[i + 1] = arr[1];
    slice[i + 2] = arr[2];
    slice[i + 3] = arr[3];
}

/// SIMD-assisted dot product for f32 slices (4 lanes at a time, remainder scalar).
#[inline]
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut sum = 0.0_f32;
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f32x4(a, i * 4);
        let vb = load_f32x4(b, i * 4);
        sum += (va * vb).reduce_add();
    }
    for i in (chunks * 4)..n {
        sum += a[i] * b[i];
    }
    sum
}

/// SIMD set slice to zero for f32 (4 lanes at a time).
#[inline]
pub fn set_zero_f32(slice: &mut [f32]) {
    let zero = f32x4::ZERO;
    let n = slice.len();
    let chunks = n / 4;
    for i in 0..chunks {
        store_f32x4(slice, i * 4, zero);
    }
    for i in (chunks * 4)..n {
        slice[i] = 0.0;
    }
}

/// SIMD scalar multiply for f32: out[i] = s * x[i].
#[inline]
pub fn scalar_mul_f32(s: f32, x: &[f32], out: &mut [f32]) {
    assert_eq!(x.len(), out.len());
    let n = x.len();
    let vs = f32x4::splat(s);
    let chunks = n / 4;
    for i in 0..chunks {
        let vx = load_f32x4(x, i * 4);
        store_f32x4(out, i * 4, vs * vx);
    }
    for i in (chunks * 4)..n {
        out[i] = s * x[i];
    }
}

/// SIMD element-wise add for f32: out[i] = a[i] + b[i].
#[inline]
pub fn add_f32(a: &[f32], b: &[f32], out: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    let n = a.len();
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f32x4(a, i * 4);
        let vb = load_f32x4(b, i * 4);
        store_f32x4(out, i * 4, va + vb);
    }
    for i in (chunks * 4)..n {
        out[i] = a[i] + b[i];
    }
}

/// SIMD element-wise subtract for f32: out[i] = a[i] - b[i].
#[inline]
pub fn sub_f32(a: &[f32], b: &[f32], out: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    let n = a.len();
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f32x4(a, i * 4);
        let vb = load_f32x4(b, i * 4);
        store_f32x4(out, i * 4, va - vb);
    }
    for i in (chunks * 4)..n {
        out[i] = a[i] - b[i];
    }
}

/// SIMD sum of squared differences for f32: sum_i (a[i] - b[i])^2.
#[inline]
pub fn squared_diff_sum_f32(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    let n = a.len();
    let mut sum = 0.0_f32;
    let chunks = n / 4;
    for i in 0..chunks {
        let va = load_f32x4(a, i * 4);
        let vb = load_f32x4(b, i * 4);
        let diff = va - vb;
        sum += (diff * diff).reduce_add();
    }
    for i in (chunks * 4)..n {
        let d = a[i] - b[i];
        sum += d * d;
    }
    sum
}

/// SIMD sum of squares for f32: sum_i x[i]².
#[inline]
pub fn squared_sum_f32(x: &[f32]) -> f32 {
    let n = x.len();
    let mut sum = 0.0_f32;
    let chunks = n / 4;
    for i in 0..chunks {
        let vx = load_f32x4(x, i * 4);
        sum += (vx * vx).reduce_add();
    }
    for i in (chunks * 4)..n {
        let v = x[i];
        sum += v * v;
    }
    sum
}

/// SIMD matrix-vector product for column-major f64: y = A * x.
/// A is m×n column-major (A(i,j) at j*m + i), x length n, y length m.
#[inline]
pub fn matvec_col_major_f64(m: usize, n: usize, a: &[f64], x: &[f64], y: &mut [f64]) {
    assert_eq!(a.len(), m * n);
    assert_eq!(x.len(), n);
    assert_eq!(y.len(), m);
    set_zero_f64(y);
    for j in 0..n {
        let xj = x[j];
        let vs = f64x4::splat(xj);
        let col_base = j * m;
        let row_chunks = m / 4;
        for i in 0..row_chunks {
            let idx = col_base + i * 4;
            let va = load_f64x4(a, idx);
            let vy = load_f64x4(y, i * 4);
            store_f64x4(y, i * 4, vy + va * vs);
        }
        for i in (row_chunks * 4)..m {
            let idx = col_base + i;
            y[i] += a[idx] * xj;
        }
    }
}

/// SIMD matrix-vector product for column-major f32: y = A * x.
/// A is m×n column-major (A(i,j) at j*m + i), x length n, y length m.
#[inline]
pub fn matvec_col_major_f32(m: usize, n: usize, a: &[f32], x: &[f32], y: &mut [f32]) {
    assert_eq!(a.len(), m * n);
    assert_eq!(x.len(), n);
    assert_eq!(y.len(), m);
    set_zero_f32(y);
    for j in 0..n {
        let xj = x[j];
        let vs = f32x4::splat(xj);
        let col_base = j * m;
        let row_chunks = m / 4;
        for i in 0..row_chunks {
            let idx = col_base + i * 4;
            let va = load_f32x4(a, idx);
            let vy = load_f32x4(y, i * 4);
            store_f32x4(y, i * 4, vy + va * vs);
        }
        for i in (row_chunks * 4)..m {
            let idx = col_base + i;
            y[i] += a[idx] * xj;
        }
    }
}
