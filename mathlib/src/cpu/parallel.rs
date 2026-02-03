//! Parallel execution backend using rayon. Enabled with the `parallel` feature (not available on wasm32).

use rayon::prelude::*;

/// Parallel dot product (chunked then reduced).
#[inline]
pub fn par_dot_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.par_iter().zip(b.par_iter()).map(|(x, y)| x * y).sum()
}

/// Parallel set slice to zero.
#[inline]
pub fn par_set_zero_f64(slice: &mut [f64]) {
    slice.par_iter_mut().for_each(|x| *x = 0.0);
}

/// Parallel scalar multiply: out[i] = s * x[i].
#[inline]
pub fn par_scalar_mul_f64(s: f64, x: &[f64], out: &mut [f64]) {
    assert_eq!(x.len(), out.len());
    out.par_iter_mut()
        .zip(x.par_iter())
        .for_each(|(o, &v)| *o = s * v);
}

/// Parallel element-wise add: out[i] = a[i] + b[i].
#[inline]
pub fn par_add_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    out.par_iter_mut()
        .zip(a.par_iter())
        .zip(b.par_iter())
        .for_each(|((o, &x), &y)| *o = x + y);
}

/// Parallel element-wise subtract: out[i] = a[i] - b[i].
#[inline]
pub fn par_sub_f64(a: &[f64], b: &[f64], out: &mut [f64]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), out.len());
    out.par_iter_mut()
        .zip(a.par_iter())
        .zip(b.par_iter())
        .for_each(|((o, &x), &y)| *o = x - y);
}

/// Parallel sum of squared differences: sum_i (a[i] - b[i])^2.
#[inline]
pub fn par_squared_diff_sum_f64(a: &[f64], b: &[f64]) -> f64 {
    assert_eq!(a.len(), b.len());
    a.par_iter()
        .zip(b.par_iter())
        .map(|(&x, &y)| {
            let d = x - y;
            d * d
        })
        .sum()
}
