//! Monte Carlo π visualization: scatter plot of points inside/outside unit circle.
//!
//! Run: `cargo run --example monte_carlo_pi_scatter`
//! Output: `monte_carlo_pi_scatter.png` (512×512). Prints estimate and n to console.

const WIDTH: u32 = 512;
const HEIGHT: u32 = 512;
const N_SAMPLES: u64 = 50_000;
const SEED: u64 = 12345;
const OUTPUT_PATH: &str = "monte_carlo_pi_scatter.png";

/// Deterministic RNG (XorShift64) matching mathlib's monte_carlo for reproducible viz.
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let x = self.state;
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        x
    }

    #[allow(clippy::cast_precision_loss)]
    fn uniform01(&mut self) -> f64 {
        const INV_2_53: f64 = 1.0 / 9_007_199_254_740_992.0;
        (self.next_u64() >> 11) as f64 * INV_2_53
    }

    fn uniform_in_range(&mut self, low: f64, high: f64) -> f64 {
        low + self.uniform01() * (high - low)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use mathlib::estimate_pi;

    let pi_est = estimate_pi(SEED, N_SAMPLES);
    println!(
        "Monte Carlo π (seed={}, n={}): estimate = {:.6}",
        SEED, N_SAMPLES, pi_est
    );

    let mut rng = XorShift64::new(SEED);
    let mut img = image::RgbaImage::from_pixel(WIDTH, HEIGHT, image::Rgba([255, 255, 255, 255]));

    for _ in 0..N_SAMPLES {
        let x = rng.uniform_in_range(-1.0, 1.0);
        let y = rng.uniform_in_range(-1.0, 1.0);
        let inside = x * x + y * y <= 1.0;
        // Map [-1,1]² to pixel coords (y flipped for image)
        let px = ((x + 1.0) * 0.5 * (WIDTH - 1) as f64).round() as u32;
        let py = ((1.0 - y) * 0.5 * (HEIGHT - 1) as f64).round() as u32;
        let px = px.min(WIDTH.saturating_sub(1));
        let py = py.min(HEIGHT.saturating_sub(1));
        let color = if inside {
            image::Rgba([30, 90, 200, 255]) // blue inside
        } else {
            image::Rgba([200, 60, 60, 255]) // red outside
        };
        img.put_pixel(px, py, color);
    }

    let path = std::path::Path::new(OUTPUT_PATH);
    img.save(path)?;
    println!("Saved {}x{} scatter to {:?}", WIDTH, HEIGHT, path);

    Ok(())
}
