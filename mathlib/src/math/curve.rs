//! 3D curve evaluation: linear, cubic Bézier, cubic Hermite, and B-spline.
//!
//! All curves are parameterized by `t` in `[0, 1]` and return a point `[f32; 3]`.
//! Used by the render crate for line and curve primitives.

/// Linear segment: lerp between start and end.
///
/// # Examples
///
/// ```
/// use mathlib::math::curve::linear_curve;
/// let start = [0.0, 0.0, 0.0];
/// let end = [1.0, 2.0, 0.0];
/// assert_eq!(linear_curve(start, end, 0.0), start);
/// assert_eq!(linear_curve(start, end, 1.0), end);
/// let mid = linear_curve(start, end, 0.5);
/// assert!((mid[0] - 0.5).abs() < 1e-6 && (mid[1] - 1.0).abs() < 1e-6);
/// ```
#[must_use]
#[inline]
pub fn linear_curve(p0: [f32; 3], p1: [f32; 3], t: f32) -> [f32; 3] {
    [
        p0[0] + t * (p1[0] - p0[0]),
        p0[1] + t * (p1[1] - p0[1]),
        p0[2] + t * (p1[2] - p0[2]),
    ]
}

/// Cubic Bézier curve (4 control points) in Bernstein form.
///
/// At `t = 0` the point is `p0`; at `t = 1` the point is `p3`.
///
/// # Examples
///
/// ```
/// use mathlib::math::curve::bezier_curve;
/// let p0 = [0.0, 0.0, 0.0];
/// let p1 = [0.33, 0.0, 0.0];
/// let p2 = [0.66, 0.0, 0.0];
/// let p3 = [1.0, 0.0, 0.0];
/// assert_eq!(bezier_curve(p0, p1, p2, p3, 0.0), p0);
/// assert_eq!(bezier_curve(p0, p1, p2, p3, 1.0), p3);
/// ```
#[must_use]
pub fn bezier_curve(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
    let u = 1.0 - t;
    let u2 = u * u;
    let u3 = u2 * u;
    let t2 = t * t;
    let t3 = t2 * t;
    let b0 = u3;
    let b1 = 3.0 * u2 * t;
    let b2 = 3.0 * u * t2;
    let b3 = t3;
    [
        b0 * p0[0] + b1 * p1[0] + b2 * p2[0] + b3 * p3[0],
        b0 * p0[1] + b1 * p1[1] + b2 * p2[1] + b3 * p3[1],
        b0 * p0[2] + b1 * p1[2] + b2 * p2[2] + b3 * p3[2],
    ]
}

/// Cubic Hermite curve: two points and two tangent vectors.
///
/// `p0`, `p1` are positions at `t = 0` and `t = 1`; `m0`, `m1` are tangents at those points.
///
/// # Examples
///
/// ```
/// use mathlib::math::curve::hermite_curve;
/// let p0 = [0.0, 0.0, 0.0];
/// let p1 = [1.0, 0.0, 0.0];
/// let m0 = [0.0, 0.0, 0.0];
/// let m1 = [0.0, 0.0, 0.0];
/// assert_eq!(hermite_curve(p0, p1, m0, m1, 0.0), p0);
/// assert_eq!(hermite_curve(p0, p1, m0, m1, 1.0), p1);
/// let mid = hermite_curve(p0, p1, m0, m1, 0.5);
/// assert!((mid[0] - 0.5).abs() < 1e-5);
/// ```
#[must_use]
pub fn hermite_curve(p0: [f32; 3], p1: [f32; 3], m0: [f32; 3], m1: [f32; 3], t: f32) -> [f32; 3] {
    let t2 = t * t;
    let t3 = t2 * t;
    let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
    let h10 = t3 - 2.0 * t2 + t;
    let h01 = -2.0 * t3 + 3.0 * t2;
    let h11 = t3 - t2;
    [
        h00 * p0[0] + h10 * m0[0] + h01 * p1[0] + h11 * m1[0],
        h00 * p0[1] + h10 * m0[1] + h01 * p1[1] + h11 * m1[1],
        h00 * p0[2] + h10 * m0[2] + h01 * p1[2] + h11 * m1[2],
    ]
}

/// Cubic B-spline segment: single segment with 4 control points, `t` in [0, 1].
///
/// The curve does not generally pass through the first or last control point.
///
/// # Examples
///
/// ```
/// use mathlib::math::curve::bspline_curve;
/// let p0 = [0.0, 0.0, 0.0];
/// let p1 = [0.33, 0.0, 0.0];
/// let p2 = [0.66, 0.0, 0.0];
/// let p3 = [1.0, 0.0, 0.0];
/// let _ = bspline_curve(p0, p1, p2, p3, 0.5);
/// ```
#[must_use]
pub fn bspline_curve(p0: [f32; 3], p1: [f32; 3], p2: [f32; 3], p3: [f32; 3], t: f32) -> [f32; 3] {
    let u = t;
    let u2 = u * u;
    let u3 = u2 * u;
    let one_minus_u = 1.0 - u;
    let b0 = one_minus_u.powi(3) / 6.0;
    let b1 = (3.0 * u3 - 6.0 * u2 + 4.0) / 6.0;
    let b2 = (-3.0 * u3 + 3.0 * u2 + 3.0 * u + 1.0) / 6.0;
    let b3 = u3 / 6.0;
    [
        b0 * p0[0] + b1 * p1[0] + b2 * p2[0] + b3 * p3[0],
        b0 * p0[1] + b1 * p1[1] + b2 * p2[1] + b3 * p3[1],
        b0 * p0[2] + b1 * p1[2] + b2 * p2[2] + b3 * p3[2],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-5;

    fn assert_near(a: [f32; 3], b: [f32; 3], eps: f32) {
        assert!((a[0] - b[0]).abs() < eps, "x: {} vs {}", a[0], b[0]);
        assert!((a[1] - b[1]).abs() < eps, "y: {} vs {}", a[1], b[1]);
        assert!((a[2] - b[2]).abs() < eps, "z: {} vs {}", a[2], b[2]);
    }

    #[test]
    fn linear_at_boundaries_and_mid() {
        let start = [0.0, 1.0, 2.0];
        let end = [3.0, 4.0, 5.0];
        assert_near(linear_curve(start, end, 0.0), start, EPS);
        assert_near(linear_curve(start, end, 1.0), end, EPS);
        let mid = linear_curve(start, end, 0.5);
        assert_near(mid, [1.5, 2.5, 3.5], EPS);
    }

    #[test]
    fn bezier_at_boundaries() {
        let p0 = [0.0, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [1.0, 1.0, 0.0];
        let p3 = [1.0, 1.0, 1.0];
        assert_near(bezier_curve(p0, p1, p2, p3, 0.0), p0, EPS);
        assert_near(bezier_curve(p0, p1, p2, p3, 1.0), p3, EPS);
    }

    #[test]
    fn hermite_zero_tangents_equals_linear() {
        let p0 = [0.0, 0.0, 0.0];
        let p1 = [2.0, 4.0, 6.0];
        let m0 = [0.0, 0.0, 0.0];
        let m1 = [0.0, 0.0, 0.0];
        assert_near(hermite_curve(p0, p1, m0, m1, 0.0), p0, EPS);
        assert_near(hermite_curve(p0, p1, m0, m1, 1.0), p1, EPS);
        assert_near(
            hermite_curve(p0, p1, m0, m1, 0.5),
            linear_curve(p0, p1, 0.5),
            EPS,
        );
    }

    #[test]
    fn bspline_mid_in_reasonable_range() {
        let p0 = [0.0, 0.0, 0.0];
        let p1 = [1.0, 0.0, 0.0];
        let p2 = [2.0, 0.0, 0.0];
        let p3 = [3.0, 0.0, 0.0];
        let mid = bspline_curve(p0, p1, p2, p3, 0.5);
        assert!(mid[0] >= 0.0 && mid[0] <= 3.0);
    }
}
