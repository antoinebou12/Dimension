//! Primitive shape types: 2D and 3D, kept separate at the type level.

use std::hash::{Hash, Hasher};

/// Point used in curve primitives; implements Eq/Hash for use as cache key (bitwise comparison).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CurvePoint(pub [f32; 3]);

impl PartialEq for CurvePoint {
    fn eq(&self, other: &Self) -> bool {
        self.0[0].to_bits() == other.0[0].to_bits()
            && self.0[1].to_bits() == other.0[1].to_bits()
            && self.0[2].to_bits() == other.0[2].to_bits()
    }
}
impl Eq for CurvePoint {}
impl Hash for CurvePoint {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0[0].to_bits().hash(state);
        self.0[1].to_bits().hash(state);
        self.0[2].to_bits().hash(state);
    }
}

/// 2D primitive shapes. All are generated with z = 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Primitive2D {
    /// Axis-aligned fullscreen quad (-1,-1) to (1,1).
    Quad,
    /// Unit square centered at origin, from -0.5 to 0.5 on x and y.
    Square,
    /// Circle centered at origin; resolution from mesh constants.
    Circle,
    /// Ellipse centered at origin (rx, ry); resolution from mesh constants.
    Ellipse,
    /// Triangle (e.g. for testing).
    Triangle,
}

/// 3D primitive shapes. All use full 3D positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Primitive3D {
    /// Axis-aligned quad in the xy-plane.
    Quad,
    /// Single triangle.
    Triangle,
    /// Unit cube centered at origin.
    Cube,
    /// Regular tetrahedron.
    Tetrahedron,
    /// Cylinder along z; resolution from mesh constants.
    Cylinder,
    /// Unit sphere centered at origin; resolution from mesh constants.
    Sphere,
    /// Cone along z (base at -z, tip at +z); resolution from mesh constants.
    Cone,
    /// Capsule along z (cylinder with hemispherical caps); resolution from mesh constants.
    Capsule,
    /// Line segment between two points (drawn as line list).
    LineSegment {
        /// Start point.
        start: CurvePoint,
        /// End point.
        end: CurvePoint,
    },
    /// Cubic Bézier curve (4 control points).
    Bezier {
        /// Control points.
        control_points: [CurvePoint; 4],
    },
    /// Cubic Hermite curve (2 points, 2 tangents).
    Hermite {
        /// Position at t = 0.
        p0: CurvePoint,
        /// Position at t = 1.
        p1: CurvePoint,
        /// Tangent at t = 0.
        m0: CurvePoint,
        /// Tangent at t = 1.
        m1: CurvePoint,
    },
    /// Cubic B-spline segment (4 control points).
    BSpline {
        /// Control points.
        control_points: [CurvePoint; 4],
    },
}

/// Top-level primitive: either a 2D or 3D shape.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Primitive {
    /// A 2D shape (z = 0).
    TwoD(Primitive2D),
    /// A 3D shape.
    ThreeD(Primitive3D),
}

impl Primitive {
    /// Returns `true` if this primitive is drawn as a line list (curve/line segment).
    #[must_use]
    pub fn is_line_list(&self) -> bool {
        matches!(
            self,
            Primitive::ThreeD(
                Primitive3D::LineSegment { .. }
                    | Primitive3D::Bezier { .. }
                    | Primitive3D::Hermite { .. }
                    | Primitive3D::BSpline { .. }
            )
        )
    }
}
