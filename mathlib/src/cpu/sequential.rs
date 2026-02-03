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
