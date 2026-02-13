//! Native demo: 3-joint arm with forward kinematics and Jacobian IK.
//!
//! Controls: Left drag = orbit, Right drag = IK target, Scroll = zoom.

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    kinematics_demo::run_native()
}
