//! Integration tests: convex_hull_2d and point_in_polygon_2d.

use collision::{convex_hull_2d, point_in_polygon_2d};

#[test]
fn hull_then_point_in_poly() {
    let pts = [
        [-1.0, 1.0],
        [-0.5, -0.5],
        [0.0, 0.5],
        [0.5, -0.5],
        [1.0, 1.0],
    ];
    let hull = convex_hull_2d(&pts);
    assert!(hull.len() >= 3);
    for p in &pts {
        assert!(
            point_in_polygon_2d(p, &hull),
            "point {:?} should be inside hull {:?}",
            p,
            hull
        );
    }
    assert!(!point_in_polygon_2d(&[2.0, 2.0], &hull));
}
