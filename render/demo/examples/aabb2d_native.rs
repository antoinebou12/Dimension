//! 2D collision demo: two circles (Lissajous + fixed), AABB outlines, merge, expand, intersection/inclusion colors.
//!
//! Run: `cargo run -p render-demo --example aabb2d_native`

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    render_demo::run_native_aabb2d()
}
