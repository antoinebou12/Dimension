//! One-dimensional quadrature (Chapter 14).
//!
//! - **Trapezoidal**: (b - a) / (n-1) * (f(a)/2 + f(x_1) + ... + f(x_{n-2}) + f(b)/2).
//! - **Simpson**: composite Simpson's rule with n subintervals (n even).
//! - **Gauss–Legendre**: n-point rule with precomputed nodes and weights on [-1, 1], mapped to [a, b].

/// Trapezoidal rule: approximate ∫_a^b f(x) dx with `n` points (n ≥ 2).
#[must_use]
pub fn trapezoidal<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    assert!(n >= 2);
    let h = (b - a) / (n - 1) as f64;
    let mut sum = 0.5 * (f(a) + f(b));
    for i in 1..(n - 1) {
        sum += f(a + i as f64 * h);
    }
    sum * h
}

/// Composite Simpson's rule: approximate ∫_a^b f(x) dx with `n` subintervals (n even, n ≥ 2).
#[must_use]
pub fn simpson<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    assert!(n >= 2 && n.is_multiple_of(2));
    let h = (b - a) / n as f64;
    let mut sum = f(a) + f(b);
    for i in 1..n {
        let x = a + i as f64 * h;
        let w = if i % 2 == 0 { 2.0 } else { 4.0 };
        sum += w * f(x);
    }
    sum * (h / 3.0)
}

/// Gauss–Legendre quadrature: approximate ∫_a^b f(x) dx with n-point rule.
///
/// Uses nodes and weights for [-1, 1] mapped linearly to [a, b].
/// Precomputed for n = 2, 3, 4, 5, 6, 8, 10; other n use 10-point rule.
#[must_use]
pub fn gauss_legendre<F>(f: F, a: f64, b: f64, n: usize) -> f64
where
    F: Fn(f64) -> f64,
{
    let (nodes, weights) = gauss_legendre_nodes_weights(n);
    let scale = (b - a) / 2.0;
    let shift = (a + b) / 2.0;
    let mut sum = 0.0;
    for (&xi, &wi) in nodes.iter().zip(weights.iter()) {
        let x = shift + scale * xi;
        sum += wi * f(x);
    }
    sum * scale
}

/// Returns (nodes, weights) for n-point Gauss–Legendre on [-1, 1].
#[allow(clippy::unreadable_literal, clippy::too_many_lines)]
fn gauss_legendre_nodes_weights(n: usize) -> (Vec<f64>, Vec<f64>) {
    match n {
        2 => (
            vec![-0.5773502691896257, 0.5773502691896257],
            vec![1.0, 1.0],
        ),
        3 => (
            vec![-0.7745966692414834, 0.0, 0.7745966692414834],
            vec![0.5555555555555556, 0.8888888888888888, 0.5555555555555556],
        ),
        4 => (
            vec![
                -0.8611363115940526,
                -0.3399810435848563,
                0.3399810435848563,
                0.8611363115940526,
            ],
            vec![
                0.3478548451374539,
                0.6521451548625461,
                0.6521451548625461,
                0.3478548451374539,
            ],
        ),
        5 => (
            vec![
                -0.9061798459386640,
                -0.5384693101056831,
                0.0,
                0.5384693101056831,
                0.9061798459386640,
            ],
            vec![
                0.2369268850561891,
                0.4786286704993665,
                0.5688888888888889,
                0.4786286704993665,
                0.2369268850561891,
            ],
        ),
        6 => (
            vec![
                -0.9324695142031521,
                -0.6612093864662645,
                -0.2386191860831969,
                0.2386191860831969,
                0.6612093864662645,
                0.9324695142031521,
            ],
            vec![
                0.1713244923791704,
                0.3607615730481386,
                0.4679139345726910,
                0.4679139345726910,
                0.3607615730481386,
                0.1713244923791704,
            ],
        ),
        8 => (
            vec![
                -0.9602898564975363,
                -0.7966664774136267,
                -0.5255324099163290,
                -0.1834346424956498,
                0.1834346424956498,
                0.5255324099163290,
                0.7966664774136267,
                0.9602898564975363,
            ],
            vec![
                0.1012285362903763,
                0.2223810344533745,
                0.3137066458778873,
                0.3626837833783620,
                0.3626837833783620,
                0.3137066458778873,
                0.2223810344533745,
                0.1012285362903763,
            ],
        ),
        10 => (
            vec![
                -0.9739065285171717,
                -0.8650633666889845,
                -0.6794095682990244,
                -0.4333953941292472,
                -0.1488743389816312,
                0.1488743389816312,
                0.4333953941292472,
                0.6794095682990244,
                0.8650633666889845,
                0.9739065285171717,
            ],
            vec![
                0.0666713443086881,
                0.1494513491505806,
                0.2190863625159820,
                0.2692667193099963,
                0.2955242247147529,
                0.2955242247147529,
                0.2692667193099963,
                0.2190863625159820,
                0.1494513491505806,
                0.0666713443086881,
            ],
        ),
        _ => gauss_legendre_nodes_weights(10.min(n.max(2))),
    }
}
