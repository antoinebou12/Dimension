//! One-dimensional root-finding (Chapter 8).
//!
//! - **Bisection**: bracket [a, b] with f(a)*f(b) < 0, then halve until |b - a| < tol.
//! - **Newton (1D)**: iterate x_{n+1} = x_n - f(x_n)/f'(x_n) until |f(x)| < tol.
//! - **Secant**: two-point iteration without derivative.
//! - **Brent**: hybrid bisection/secant/inverse-quadratic interpolation.

/// Result of a root-finding algorithm.
#[derive(Clone, Debug)]
pub struct RootResult {
    /// Approximate root.
    pub x: f64,
    /// Function value at x (should be near zero).
    pub fx: f64,
    /// Number of iterations performed.
    pub iterations: u32,
    /// Whether the algorithm converged.
    pub converged: bool,
}

/// Bisection: find a root of `f` in `[a, b]` assuming f(a)*f(b) < 0.
///
/// Returns a point `x` in the bracket with `|b - a| <= tol`, or `None` if the bracket is invalid.
#[must_use]
pub fn bisection<F>(f: F, mut a: f64, mut b: f64, tol: f64) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    let mut fa = f(a);
    let fb = f(b);
    if fa * fb > 0.0 {
        return None;
    }
    while (b - a).abs() > tol {
        let c = (a + b) * 0.5;
        let fc = f(c);
        if fc == 0.0 {
            return Some(c);
        }
        if fa * fc < 0.0 {
            b = c;
        } else {
            a = c;
            fa = fc;
        }
    }
    Some((a + b) * 0.5)
}

/// Newton's method in one variable: solve f(x) = 0 given derivative `df`.
///
/// Returns `x` such that |f(x)| < tol, or `None` if the iteration does not converge (e.g. df = 0).
#[must_use]
pub fn newton_1d<F, D>(f: F, df: D, mut x0: f64, tol: f64, max_iters: usize) -> Option<f64>
where
    F: Fn(f64) -> f64,
    D: Fn(f64) -> f64,
{
    for _ in 0..max_iters {
        let fx = f(x0);
        if fx.abs() < tol {
            return Some(x0);
        }
        let d = df(x0);
        if d.abs() < 1e-15 {
            return None;
        }
        x0 -= fx / d;
    }
    None
}

/// Secant method: find root of f using two initial guesses x0, x1 (no derivative).
#[must_use]
pub fn secant<F>(f: F, x0: f64, x1: f64, tol: f64, max_iter: usize) -> RootResult
where
    F: Fn(f64) -> f64,
{
    let mut x_prev = x0;
    let mut x = x1;
    let mut f_prev = f(x_prev);
    let mut fx = f(x);
    for iter in 0..max_iter {
        if fx.abs() <= tol {
            return RootResult {
                x,
                fx,
                iterations: u32::try_from(iter + 1).unwrap_or(u32::MAX),
                converged: true,
            };
        }
        let denom = fx - f_prev;
        if denom.abs() < f64::EPSILON * (x - x_prev).abs().max(1e-15) {
            break;
        }
        let x_new = x - fx * (x - x_prev) / denom;
        x_prev = x;
        f_prev = fx;
        x = x_new;
        fx = f(x);
    }
    RootResult {
        x,
        fx,
        iterations: u32::try_from(max_iter).unwrap_or(u32::MAX),
        converged: false,
    }
}

/// Brent's method: robust hybrid combining bisection, secant, and inverse quadratic interpolation.
///
/// Finds a root of f in [a, b] where f(a)*f(b) < 0.
#[must_use]
pub fn brent<F>(f: F, mut a: f64, mut b: f64, tol: f64, max_iter: usize) -> RootResult
where
    F: Fn(f64) -> f64,
{
    let mut fa = f(a);
    let mut fb = f(b);
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }
    if fa * fb >= 0.0 {
        return RootResult {
            x: a,
            fx: fa,
            iterations: 0,
            converged: false,
        };
    }
    let mut c = a;
    let mut fc = fa;
    let mut d = b - a;
    for iter in 0..max_iter {
        if fb.abs() <= tol || (b - a).abs() <= tol {
            return RootResult {
                x: b,
                fx: fb,
                iterations: u32::try_from(iter + 1).unwrap_or(u32::MAX),
                converged: true,
            };
        }
        let (s, use_bisection) = if (fc - fa).abs() > f64::EPSILON && (fc - fb).abs() > f64::EPSILON
        {
            let s = a * fb * fc / ((fa - fb) * (fa - fc))
                + b * fa * fc / ((fb - fa) * (fb - fc))
                + c * fa * fb / ((fc - fa) * (fc - fb));
            let bisect = s < (3.0 * a + b) / 4.0
                || s > b
                || (a - s).abs() >= (b - a).abs() / 2.0
                || (c - s).abs() >= (c - d).abs() / 2.0;
            (s, bisect)
        } else {
            let s = b - fb * (b - a) / (fb - fa);
            let bisect = s <= a || s >= b;
            (s, bisect)
        };
        let s = if use_bisection { (a + b) / 2.0 } else { s };
        d = c;
        c = b;
        fc = fb;
        let fs = f(s);
        if fa * fs < 0.0 {
            b = s;
            fb = fs;
        } else {
            a = s;
            fa = fs;
        }
        if fa.abs() < fb.abs() {
            std::mem::swap(&mut a, &mut b);
            std::mem::swap(&mut fa, &mut fb);
        }
    }
    RootResult {
        x: b,
        fx: fb,
        iterations: u32::try_from(max_iter).unwrap_or(u32::MAX),
        converged: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bisection_sqrt2() {
        let f = |x: f64| x * x - 2.0;
        let r = bisection(f, 1.0, 2.0, 1e-10);
        assert!(r.is_some());
        let x = r.unwrap();
        assert!((x - 2.0_f64.sqrt()).abs() < 1e-6);
    }

    #[test]
    fn secant_cos_minus_x() {
        let f = |x: f64| x.cos() - x;
        let r = secant(f, 0.0, 1.0, 1e-10, 50);
        assert!(r.converged);
        let exact = 0.7390851332151607;
        assert!((r.x - exact).abs() < 1e-8);
    }

    #[test]
    fn brent_sqrt2() {
        let f = |x: f64| x * x - 2.0;
        let r = brent(f, 1.0, 2.0, 1e-10, 100);
        assert!(r.converged);
        assert!((r.x - 2.0_f64.sqrt()).abs() < 1e-6);
    }
}
