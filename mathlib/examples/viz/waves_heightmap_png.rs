//! Generate an RGBA waves heightmap image and save it as PNG.
//!
//! Pipeline: noise (wave_2d) → height [0,1] → colormap (elevation) → RGBA image → PNG.
//!
//! Run: `cargo run --example waves_heightmap_png`
//! Output: `waves_heightmap.png` (512×512 RGBA).

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const OUTPUT_PATH: &str = "waves_heightmap.png";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use mathlib::{height_to_rgba, wave_2d};

    let img = image::RgbaImage::from_fn(WIDTH, HEIGHT, |x, y| {
        let u = (x as f64) / (WIDTH - 1) as f64;
        let v = (y as f64) / (HEIGHT - 1) as f64;
        let h = wave_2d(u, v);
        let [r, g, b, a] = height_to_rgba(h);
        image::Rgba([r, g, b, a])
    });

    let path = std::path::Path::new(OUTPUT_PATH);
    img.save(path)?;
    println!(
        "Saved {}x{} RGBA waves heightmap to {:?}",
        WIDTH, HEIGHT, path
    );

    Ok(())
}
