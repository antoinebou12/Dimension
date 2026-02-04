//! Parallel execution backend using par-iter with chili. Enabled with the `parallel` feature (not available on wasm32).

use par_iter::prelude::*;

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

#[cfg(all(test, feature = "parallel", not(target_arch = "wasm32")))]
mod tests {
    use super::*;

    #[test]
    fn test_par_dot_f64_length_one() {
        let a = [3.0];
        let b = [4.0];
        assert!((par_dot_f64(&a, &b) - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_par_dot_f64_length_five() {
        let a = [1.0, 2.0, 3.0, 4.0, 5.0];
        let b = [2.0, 3.0, 4.0, 5.0, 6.0];
        let expected: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        assert!((par_dot_f64(&a, &b) - expected).abs() < 1e-10);
    }

    #[test]
    fn test_par_set_zero_f64() {
        let mut slice = [1.0, 2.0, 3.0];
        par_set_zero_f64(&mut slice);
        assert!(slice.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_par_scalar_mul_f64() {
        let x = [1.0, 2.0, 3.0];
        let mut out = [0.0; 3];
        par_scalar_mul_f64(2.0, &x, &mut out);
        assert!((out[0] - 2.0).abs() < 1e-10);
        assert!((out[1] - 4.0).abs() < 1e-10);
        assert!((out[2] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_par_add_f64() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [0.5, 1.0, 1.5, 2.0];
        let mut out = [0.0; 4];
        par_add_f64(&a, &b, &mut out);
        assert!((out[0] - 1.5).abs() < 1e-10);
        assert!((out[1] - 3.0).abs() < 1e-10);
        assert!((out[2] - 4.5).abs() < 1e-10);
        assert!((out[3] - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_par_sub_f64() {
        let a = [5.0, 4.0, 3.0, 2.0];
        let b = [1.0, 1.0, 1.0, 1.0];
        let mut out = [0.0; 4];
        par_sub_f64(&a, &b, &mut out);
        assert!((out[0] - 4.0).abs() < 1e-10);
        assert!((out[1] - 3.0).abs() < 1e-10);
        assert!((out[2] - 2.0).abs() < 1e-10);
        assert!((out[3] - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_par_squared_diff_sum_f64() {
        let a = [1.0, 2.0, 3.0];
        let b = [2.0, 1.0, 4.0];
        let expected: f64 = a
            .iter()
            .zip(b.iter())
            .map(|(&x, &y)| {
                let d: f64 = x - y;
                d * d
            })
            .sum();
        assert!((par_squared_diff_sum_f64(&a, &b) - expected).abs() < 1e-10);
        assert!((par_squared_diff_sum_f64(&a, &b) - 3.0).abs() < 1e-10); // (1-2)²+(2-1)²+(3-4)² = 3
    }
}
