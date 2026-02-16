//! Math / CG: 3D types, camera, quaternion, dual quaternion, trig, easing, curve, RBF interpolation.

pub mod cg;
pub mod curve;
pub mod dual_quaternion;
pub mod easing;
/// 3D types and helpers: Point3, Vector3f/4f, Matrix3f/4f, homogeneous, inverse, rotation.
pub mod math3d;
/// Raw array-based 3D math (`[f32; 9]` matrices, `[f32; 3]` vectors) for physics and performance.
pub mod math3d_raw;
pub mod quantize;
pub mod quaternion;
pub mod rbf;
pub mod trig;
pub mod twist;
