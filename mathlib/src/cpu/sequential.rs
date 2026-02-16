//! Sequential (scalar) execution backend. Default when no parallel or SIMD features are enabled.

/// Sequential dot product for two slices.
#[inline]
pub fn dot_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Sequential set slice to zero.
#[inline]
pub fn set_zero_f64(slice: &mut [f64]) {
    for x in slice.iter_mut() {
        *x = 0.0;
    }
}

/// Sequential scalar multiply: out[i] = s * x[i].
#[inline]
pub fn scalar_mul_f64(s: f64, x: &[f64], out: &mut [f64]) {
    assert_eq!(x.len(), out.len());
    for (o, &v) in out.iter_mut().zip(x.iter()) {
        *o = s * v;
    }
}

/// Sequential element-wise add: out[i] = a[i] + b[i].
#[inline]
pub fn add_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    for ((o, &x), &y) in out.iter_mut().zip(a.iter()).zip(b.iter()) {
        *o = x + y;
    }
}

/// Sequential element-wise subtract: out[i] = a[i] - b[i].
#[inline]
pub fn sub_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    for ((o, &x), &y) in out.iter_mut().zip(a.iter()).zip(b.iter()) {
        *o = x - y;
    }
}

/// Sequential sum of squared differences: `sum_i` (a[i] - b[i])^2.
#[inline]
pub fn squared_diff_sum_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| {
            let d = x - y;
            d * d
        })
        .sum()
}

/// Sequential sum of squares: `sum_i` x[i]².
#[inline]
pub fn squared_sum_f64(x: &[f64]) -> f64 {
    x.iter().map(|&v| v * v).sum()
}

/// Sequential sum of absolute differences: `sum_i` |a[i] - b[i]|.
#[inline]
pub fn abs_diff_sum_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(&x, &y)| (x - y).abs()).sum()
}

/// Sequential max of absolute differences: `max_i` |a[i] - b[i]|.
#[inline]
pub fn max_abs_diff_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| (x - y).abs())
        .fold(0.0_f64, f64::max)
}

/// Sequential dot product for f32 slices.
#[inline]
pub fn dot_f32(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Sequential scalar multiply for f32: out[i] = s * x[i].
#[inline]
pub fn scalar_mul_f32(s: f32, x: &[f32], out: &mut [f32]) {
    assert_eq!(x.len(), out.len());
    for (o, &v) in out.iter_mut().zip(x.iter()) {
        *o = s * v;
    }
}

/// Sequential sum of squared differences for f32: `sum_i` (a[i] - b[i])^2.
#[inline]
pub fn squared_diff_sum_f32(a: &[f32], b: &[f32]) -> f32 {
    assert_eq!(a.len(), b.len());
    a.iter()
        .zip(b.iter())
        .map(|(&x, &y)| {
            let d = x - y;
            d * d
        })
        .sum()
}

/// Sequential sum of squares for f32: `sum_i` x[i]².
#[inline]
pub fn squared_sum_f32(x: &[f32]) -> f32 {
    x.iter().map(|&v| v * v).sum()
}

/// Sequential element-wise add for f32: out[i] = a[i] + b[i].
#[inline]
pub fn add_f32(a: &[f32], b: &[f32], out: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    for ((o, &x), &y) in out.iter_mut().zip(a.iter()).zip(b.iter()) {
        *o = x + y;
    }
}

/// Sequential matrix-vector product for column-major f32: y = A * x.
#[inline]
pub fn matvec_col_major_f32(m: usize, n: usize, a: &[f32], x: &[f32], y: &mut [f32]) {
    assert_eq!(a.len(), m * n);
    assert_eq!(x.len(), n);
    assert_eq!(y.len(), m);
    y[..m].fill(0.0);
    for j in 0..n {
        let xj = x[j];
        for i in 0..m {
            y[i] += a[j * m + i] * xj;
        }
    }
}
