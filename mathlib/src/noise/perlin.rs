//! 2D Perlin gradient noise (deterministic, fixed permutation table).

/// Permutation table (256 entries, doubled for wrapping). Fixed seed for reproducibility.
const PERM: [u8; 256] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,
];

#[inline]
fn perm(i: i32) -> u8 {
    #[allow(clippy::cast_sign_loss)]
    PERM[(i & 0xff) as usize]
}

/// Four gradient directions (unit vectors): (1,0), (-1,0), (0,1), (0,-1).
#[inline]
fn grad2(h: u8, x: f64, y: f64) -> f64 {
    match h & 3 {
        0 => x,
        1 => -x,
        2 => y,
        _ => -y,
    }
}

/// Smoothstep 5t^4 - 10t^3 + 10t^2 - 5t + t (quintic for Perlin).
#[inline]
fn fade(t: f64) -> f64 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// 2D Perlin noise at (x, y). Output is in approximately [-1, 1]; rescale for [0, 1] if needed.
#[inline]
#[allow(clippy::cast_possible_truncation)]
pub fn perlin_2d(x: f64, y: f64) -> f64 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;
    let xf = x - f64::from(xi);
    let yf = y - f64::from(yi);

    let u = fade(xf);
    let v = fade(yf);

    let aa = i32::from(perm(xi)) + yi;
    let ab = i32::from(perm(xi)) + yi + 1;
    let ba = i32::from(perm(xi + 1)) + yi;
    let bb = i32::from(perm(xi + 1)) + yi + 1;

    let x0 = xf;
    let x1 = xf - 1.0;
    let y0 = yf;
    let y1 = yf - 1.0;

    let g00 = grad2(perm(aa), x0, y0);
    let g10 = grad2(perm(ba), x1, y0);
    let g01 = grad2(perm(ab), x0, y1);
    let g11 = grad2(perm(bb), x1, y1);

    let nx0 = (1.0 - u) * g00 + u * g10;
    let nx1 = (1.0 - u) * g01 + u * g11;
    (1.0 - v) * nx0 + v * nx1
}
