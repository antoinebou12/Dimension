//! 2D convex hull and point-in-polygon.

/// 2D cross product (determinant) of (p1 - p0) and (p2 - p0).
/// Positive = counter-clockwise turn, negative = clockwise, zero = collinear.
#[inline]
fn cross2d(p0: &[f32; 2], p1: &[f32; 2], p2: &[f32; 2]) -> f32 {
    (p1[0] - p0[0]) * (p2[1] - p0[1]) - (p1[1] - p0[1]) * (p2[0] - p0[0])
}

/// Convex hull of 2D points (Graham scan). Returns hull vertices in counter-clockwise order.
///
/// Input can be any slice of `[f32; 2]`. Duplicates and collinear points on the hull
/// may be omitted. Returns empty vec if `points.len() < 3`.
///
/// # Examples
///
/// ```
/// use collision::convex_hull_2d;
///
/// let pts = [[0.0, 0.0], [1.0, 0.0], [0.5, 0.5], [1.0, 1.0], [0.0, 1.0]];
/// let hull = convex_hull_2d(&pts);
/// assert!(hull.len() >= 3);
/// ```
#[must_use]
pub fn convex_hull_2d(points: &[[f32; 2]]) -> Vec<[f32; 2]> {
    if points.len() < 3 {
        return Vec::new();
    }
    let mut pts: Vec<[f32; 2]> = points.to_vec();
    let mut min_idx = 0;
    for (i, p) in pts.iter().enumerate().skip(1) {
        if p[1] < pts[min_idx][1] || (p[1] == pts[min_idx][1] && p[0] < pts[min_idx][0]) {
            min_idx = i;
        }
    }
    pts.swap(0, min_idx);
    let origin = pts[0];
    pts[1..].sort_by(|a, b| {
        let c = cross2d(&origin, a, b);
        if c.abs() < 1e-10 {
            let da =
                (a[0] - origin[0]) * (a[0] - origin[0]) + (a[1] - origin[1]) * (a[1] - origin[1]);
            let db =
                (b[0] - origin[0]) * (b[0] - origin[0]) + (b[1] - origin[1]) * (b[1] - origin[1]);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        } else if c > 0.0 {
            std::cmp::Ordering::Less
        } else {
            std::cmp::Ordering::Greater
        }
    });
    let mut hull = Vec::with_capacity(pts.len().min(64));
    hull.push(pts[0]);
    hull.push(pts[1]);
    for p in pts.iter().skip(2) {
        while hull.len() >= 2 && cross2d(&hull[hull.len() - 2], hull.last().unwrap(), p) <= 0.0 {
            hull.pop();
        }
        hull.push(*p);
    }
    hull
}

/// Point-in-polygon test (ray-cast: horizontal ray, count crossings).
///
/// Returns `true` if `point` is inside or on the boundary of `polygon`.
/// Polygon is treated as closed (segment from last vertex to first is implied).
/// Vertices should be in counter-clockwise or clockwise order.
///
/// # Panics
///
/// Does not panic; returns false if polygon has fewer than 3 vertices.
#[must_use]
pub fn point_in_polygon_2d(point: &[f32; 2], polygon: &[[f32; 2]]) -> bool {
    if polygon.len() < 3 {
        return false;
    }
    let [px, py] = *point;
    let n = polygon.len();
    const EPS: f32 = 1e-6;
    let mut j = n - 1;
    for i in 0..n {
        let [xi, yi] = polygon[i];
        let [xj, yj] = polygon[j];
        if (px - xi).abs() < EPS && (py - yi).abs() < EPS {
            return true;
        }
        let dy = yj - yi;
        if dy.abs() > EPS {
            let t = (py - yi) / dy;
            if (0.0..=1.0).contains(&t) {
                let x_intersect = xi + t * (xj - xi);
                if (px - x_intersect).abs() < EPS {
                    return true;
                }
            }
        }
        j = i;
    }
    let mut inside = false;
    j = n - 1;
    for i in 0..n {
        let [xi, yi] = polygon[i];
        let [xj, yj] = polygon[j];
        let dy = yj - yi;
        if dy.abs() > EPS && ((yi > py) != (yj > py)) {
            let t = (py - yi) / dy;
            let x_intersect = xi + t * (xj - xi);
            if px < x_intersect {
                inside = !inside;
            }
        }
        j = i;
    }
    inside
}

/// Ray–convex-polygon intersection in 2D.
///
/// Returns the smallest non-negative `t` such that `origin + t * dir` lies on or inside
/// the polygon boundary, or `None` if the ray does not hit. Polygon is treated as closed.
/// For convex polygons only; behavior for non-convex is unspecified.
#[must_use]
pub fn ray_polygon_2d(origin: &[f32; 2], dir: &[f32; 2], polygon: &[[f32; 2]]) -> Option<f32> {
    if polygon.len() < 3 {
        return None;
    }
    const EPS: f32 = 1e-8;
    let mut t_min: Option<f32> = None;
    let n = polygon.len();
    for i in 0..n {
        let a = polygon[i];
        let b = polygon[(i + 1) % n];
        let ex = b[0] - a[0];
        let ey = b[1] - a[1];
        let det = ex * (-dir[1]) - (-dir[0]) * ey;
        if det.abs() < EPS {
            continue;
        }
        let ox_ax = origin[0] - a[0];
        let oy_ay = origin[1] - a[1];
        let s = (-ox_ax * dir[1] + dir[0] * oy_ay) / det;
        let t = (ex * oy_ay - ox_ax * ey) / det;
        if t >= 0.0 && (0.0..=1.0).contains(&s) {
            t_min = Some(match t_min {
                None => t,
                Some(tm) => tm.min(t),
            });
        }
    }
    t_min
}

#[cfg(test)]
mod tests {
    use super::{convex_hull_2d, point_in_polygon_2d, ray_polygon_2d};

    #[test]
    fn convex_hull_square() {
        let pts = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0], [0.5, 0.5]];
        let hull = convex_hull_2d(&pts);
        assert_eq!(hull.len(), 4);
    }

    #[test]
    fn convex_hull_few_points() {
        assert!(convex_hull_2d(&[]).is_empty());
        assert!(convex_hull_2d(&[[0.0, 0.0]]).is_empty());
        assert!(convex_hull_2d(&[[0.0, 0.0], [1.0, 0.0]]).is_empty());
    }

    #[test]
    fn point_in_polygon_triangle() {
        let tri = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        assert!(point_in_polygon_2d(&[0.5, 0.3], &tri));
        assert!(!point_in_polygon_2d(&[2.0, 2.0], &tri));
    }

    #[test]
    fn point_in_polygon_square() {
        let sq = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        assert!(point_in_polygon_2d(&[0.5, 0.5], &sq));
        assert!(!point_in_polygon_2d(&[-0.1, 0.5], &sq));
    }

    #[test]
    fn point_in_polygon_few_vertices() {
        assert!(!point_in_polygon_2d(&[0.0, 0.0], &[]));
        assert!(!point_in_polygon_2d(&[0.0, 0.0], &[[0.0, 0.0], [1.0, 0.0]]));
    }

    #[test]
    fn ray_polygon_2d_hit() {
        let tri = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let t = ray_polygon_2d(&[0.5, -1.0], &[0.0, 1.0], &tri);
        assert!(t.is_some());
        assert!(t.unwrap() > 0.0);
    }

    #[test]
    fn ray_polygon_2d_miss() {
        let tri = [[0.0, 0.0], [1.0, 0.0], [0.5, 1.0]];
        let t = ray_polygon_2d(&[2.0, 2.0], &[1.0, 0.0], &tri);
        assert!(t.is_none());
    }
}
