//! Generate a 3D point cloud from the same waves heightmap (PLY format).
//!
//! Pipeline: noise (wave_2d) → height [0,1] → colormap (elevation) → (x,y,z) + RGB → PLY.
//!
//! Run: `cargo run --example waves_heightmap_pointcloud`
//! Output: `waves_heightmap.ply`. Open with MeshLab, CloudCompare, or any PLY viewer.

const GRID: usize = 64;
const SCALE_XY: f64 = 2.0;
const SCALE_Z: f64 = 0.5;
const OUTPUT_PATH: &str = "waves_heightmap.ply";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use mathlib::{height_to_rgb, wave_2d};
    use std::fs::File;
    use std::io::BufWriter;
    use std::io::Write;
    use std::path::Path;

    let mut points: Vec<(f64, f64, f64, u8, u8, u8)> = Vec::new();

    for i in 0..=GRID {
        for j in 0..=GRID {
            let u = (i as f64) / (GRID as f64);
            let v = (j as f64) / (GRID as f64);
            let h = wave_2d(u, v);
            let [r, g, b] = height_to_rgb(h);
            let x = (u - 0.5) * SCALE_XY;
            let y = (v - 0.5) * SCALE_XY;
            let z = (h - 0.5) * SCALE_Z;
            points.push((x, y, z, r, g, b));
        }
    }

    let path = Path::new(OUTPUT_PATH);
    let f = File::create(path)?;
    let mut w = BufWriter::new(f);

    writeln!(w, "ply")?;
    writeln!(w, "format ascii 1.0")?;
    writeln!(w, "element vertex {}", points.len())?;
    writeln!(w, "property float x")?;
    writeln!(w, "property float y")?;
    writeln!(w, "property float z")?;
    writeln!(w, "property uchar red")?;
    writeln!(w, "property uchar green")?;
    writeln!(w, "property uchar blue")?;
    writeln!(w, "end_header")?;

    for (x, y, z, r, g, b) in &points {
        writeln!(w, "{} {} {} {} {} {}", x, y, z, r, g, b)?;
    }

    w.flush()?;
    println!(
        "Saved {} points (waves heightmap 3D) to {:?}",
        points.len(),
        path
    );
    println!("Open with MeshLab, CloudCompare, or any PLY viewer.");

    Ok(())
}
