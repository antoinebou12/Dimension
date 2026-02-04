//! Example: MVP (model-view-projection), rotation matrix, inverse, and transpose.
//!
//! Builds model (translation + rotation), view (look-at), and projection;
//! computes MVP = projection * view * model; demonstrates inverse and transpose.

use mathlib::{
    Perspective3, from_euler_angles, look_at_rh, matrix4f_inverse, model_view_projection,
    new_perspective, new_translation, vector3,
};

fn main() {
    // Model: translate then rotate (Euler angles in radians)
    let t = vector3(1.0, 0.0, 0.0);
    let model_t = new_translation(&t);
    let model_r = from_euler_angles(0.0, 0.1, 0.2);
    let model = &model_t * &model_r;

    // View: camera at eye looking at target, up vector
    let eye = vector3(0.0, 0.0, 5.0);
    let target = vector3(0.0, 0.0, 0.0);
    let up = vector3(0.0, 1.0, 0.0);
    let view = look_at_rh(&eye, &target, &up);

    // Projection: perspective (aspect, fov_y rad, near, far)
    let aspect = 16.0 / 9.0;
    let fov_y = std::f32::consts::FRAC_PI_4;
    let near = 0.1;
    let far = 100.0;
    let projection = new_perspective(aspect, fov_y, near, far);

    // MVP = projection * view * model (column-major)
    let mvp = model_view_projection(&model, &view, &projection);

    println!("MVP (model-view-projection):");
    println!("  Model (translation * rotation): 4x4");
    println!("  View (look_at_rh): 4x4");
    println!("  Projection (perspective): 4x4");
    println!("  MVP = projection * view * model");
    println!(
        "  MVP[0,0] = {}, MVP[3,3] = {}",
        mvp.get(0, 0),
        mvp.get(3, 3)
    );

    // Rotation matrix (4x4): transpose = inverse for orthonormal rotation
    let rot = from_euler_angles(0.1, 0.2, 0.3);
    let rot_transpose = rot.transpose();
    let rot_inv = matrix4f_inverse(&rot);
    let rt_times_r = &rot_transpose * &rot;
    println!("\nRotation matrix (from_euler_angles):");
    println!(
        "  R^T * R (should be identity): [{}, {}], [{}, {}]",
        rt_times_r.get(0, 0),
        rt_times_r.get(0, 1),
        rt_times_r.get(1, 0),
        rt_times_r.get(1, 1)
    );
    println!(
        "  R^(-1) equals R^T for rotation: {}",
        (rot_inv.get(0, 0) - rot_transpose.get(0, 0)).abs() < 1e-5
    );

    // Inverse of view matrix (e.g. for transforming from view to world)
    let view_inv = matrix4f_inverse(&view);
    let view_times_inv = &view * &view_inv;
    println!("\nView inverse:");
    println!(
        "  view * view^(-1) [0,0] = {} (expect 1)",
        view_times_inv.get(0, 0)
    );
    println!(
        "  view * view^(-1) [3,3] = {} (expect 1)",
        view_times_inv.get(3, 3)
    );

    // Perspective inverse (used for unproject)
    let proj = Perspective3::new(aspect, fov_y, near, far);
    let proj_inv = proj.inverse_matrix();
    let proj_mat = proj.as_matrix();
    let proj_times_inv = &proj_mat * &proj_inv;
    println!("\nProjection inverse (Perspective3::inverse_matrix):");
    println!(
        "  proj * proj^(-1) [0,0] = {} (expect 1)",
        proj_times_inv.get(0, 0)
    );
}
