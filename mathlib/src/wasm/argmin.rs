//! WasmPsoResult for particle swarm optimization and line search from JavaScript.
//!
//! Cost function is passed as a JS callback: `(position: Float64Array) => number`.
//! Uses a sequential PSO implementation (no Sync bound) for wasm.

use js_sys::Float64Array;
use wasm_bindgen::prelude::*;

use crate::argmin::PsoOptions;
use crate::argmin::linesearch::{self, LineSearchOptions};

/// Result of PSO: best position, cost, and iterations.
#[wasm_bindgen]
pub struct WasmPsoResult {
    best_position: Vec<f64>,
    best_cost: f64,
    iterations: u32,
}

#[wasm_bindgen]
impl WasmPsoResult {
    /// Best position found.
    #[wasm_bindgen(js_name = getBestPosition)]
    pub fn get_best_position(&self) -> Vec<f64> {
        self.best_position.clone()
    }

    /// Cost at best position.
    #[wasm_bindgen(js_name = getBestCost)]
    pub fn get_best_cost(&self) -> f64 {
        self.best_cost
    }

    /// Number of iterations performed.
    #[wasm_bindgen(js_name = getIterations)]
    pub fn get_iterations(&self) -> u32 {
        self.iterations
    }
}

/// Result of PSO with per-iteration global best (for trajectory visualization).
#[wasm_bindgen]
pub struct WasmPsoResultWithHistory {
    best_position: Vec<f64>,
    best_cost: f64,
    iterations: u32,
    /// Flattened: iteration 0 position, then iteration 1 position, ... (length = iterations * dim).
    history_positions: Vec<f64>,
    history_costs: Vec<f64>,
}

#[wasm_bindgen]
impl WasmPsoResultWithHistory {
    #[wasm_bindgen(js_name = getBestPosition)]
    pub fn get_best_position(&self) -> Vec<f64> {
        self.best_position.clone()
    }

    #[wasm_bindgen(js_name = getBestCost)]
    pub fn get_best_cost(&self) -> f64 {
        self.best_cost
    }

    #[wasm_bindgen(js_name = getIterations)]
    pub fn get_iterations(&self) -> u32 {
        self.iterations
    }

    /// Global best position at each iteration, row-major: [iter0_x0, iter0_x1, ..., iter1_x0, ...].
    #[wasm_bindgen(js_name = getHistoryPositions)]
    pub fn get_history_positions(&self) -> Vec<f64> {
        self.history_positions.clone()
    }

    /// Global best cost at each iteration.
    #[wasm_bindgen(js_name = getHistoryCosts)]
    pub fn get_history_costs(&self) -> Vec<f64> {
        self.history_costs.clone()
    }
}

/// Deterministic RNG (xorshift64) for reproducible PSO.
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

    fn uniform_in_bounds(&mut self, low: &[f64], high: &[f64], out: &mut [f64]) {
        for i in 0..out.len() {
            let u = self.uniform01();
            out[i] = low[i] + u * (high[i] - low[i]);
        }
    }
}

fn seed_from_bounds(low: &[f64], high: &[f64], num_particles: usize) -> u64 {
    let mut h = num_particles as u64;
    for (i, (&a, &b)) in low.iter().zip(high.iter()).enumerate().take(8) {
        h = h.wrapping_add((i as u64).wrapping_mul(a.to_bits()));
        h = h.wrapping_add(b.to_bits());
    }
    if h == 0 { 1 } else { h }
}

/// Run PSO to minimize a cost function over a box.
///
/// `cost_fn` is a JS function `(position: Float64Array) => number`.
/// `lower` and `upper` define the search bounds per dimension.
/// Optional `seed` (number or undefined in JS): when provided, used for the RNG for reproducible or fresh runs; when omitted, the seed is derived from the bounds.
#[wasm_bindgen(js_name = psoMinimize)]
pub fn pso_minimize(
    lower: Vec<f64>,
    upper: Vec<f64>,
    num_particles: usize,
    max_iters: u32,
    cost_fn: &js_sys::Function,
    seed: Option<u32>,
) -> Result<WasmPsoResult, JsError> {
    if lower.len() != upper.len() {
        return Err(JsError::new(
            "lower and upper bounds must have the same length",
        ));
    }
    if lower.is_empty() {
        return Err(JsError::new("bounds must have at least one dimension"));
    }
    if num_particles < 1 {
        return Err(JsError::new("num_particles must be at least 1"));
    }

    let dim = lower.len();
    let opts = PsoOptions::default();

    let eval_cost = |x: &[f64]| -> f64 {
        let arr = Float64Array::new_with_length(x.len() as u32);
        arr.copy_from(x);
        cost_fn
            .call1(&JsValue::NULL, &arr)
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(f64::INFINITY)
    };

    let seed_u64 = seed
        .map(u64::from)
        .unwrap_or_else(|| seed_from_bounds(&lower, &upper, num_particles));
    let mut rng = XorShift64::new(seed_u64);

    let mut delta = vec![0.0_f64; dim];
    for i in 0..dim {
        delta[i] = upper[i] - lower[i];
    }
    let delta_neg: Vec<f64> = delta.iter().map(|&d| -d).collect();

    // Initialize particles
    let mut positions: Vec<Vec<f64>> = Vec::with_capacity(num_particles);
    let mut velocities: Vec<Vec<f64>> = Vec::with_capacity(num_particles);
    let mut best_positions: Vec<Vec<f64>> = Vec::with_capacity(num_particles);
    let mut costs: Vec<f64> = Vec::with_capacity(num_particles);
    let mut best_costs: Vec<f64> = Vec::with_capacity(num_particles);

    for _ in 0..num_particles {
        let mut pos = vec![0.0_f64; dim];
        let mut vel = vec![0.0_f64; dim];
        rng.uniform_in_bounds(&lower, &upper, &mut pos);
        rng.uniform_in_bounds(&delta_neg, &delta, &mut vel);
        let c = eval_cost(&pos);
        positions.push(pos.clone());
        velocities.push(vel);
        best_positions.push(pos);
        costs.push(c);
        best_costs.push(c);
    }

    let mut best_idx = 0;
    for i in 1..num_particles {
        if costs[i] < costs[best_idx] {
            best_idx = i;
        }
    }
    let mut best_position = best_positions[best_idx].clone();
    let mut best_cost = costs[best_idx];

    let mut diff = vec![0.0_f64; dim];
    let mut scaled = vec![0.0_f64; dim];
    let mut new_vel = vec![0.0_f64; dim];
    let mut new_pos = vec![0.0_f64; dim];

    for _ in 0..max_iters {
        for p in 0..num_particles {
            // v_new = w*v + c1*r1*(pbest - x) + c2*r2*(gbest - x)
            for i in 0..dim {
                new_vel[i] = opts.inertia * velocities[p][i];
            }
            for i in 0..dim {
                diff[i] = (best_positions[p][i] - positions[p][i]) * rng.uniform01();
            }
            for i in 0..dim {
                new_vel[i] += opts.cognitive * diff[i];
            }
            for i in 0..dim {
                scaled[i] = (best_position[i] - positions[p][i]) * rng.uniform01();
            }
            for i in 0..dim {
                new_vel[i] += opts.social * scaled[i];
            }
            velocities[p].copy_from_slice(&new_vel);
            for i in 0..dim {
                new_pos[i] = (positions[p][i] + velocities[p][i]).clamp(lower[i], upper[i]);
            }
            positions[p].copy_from_slice(&new_pos);
        }

        for p in 0..num_particles {
            costs[p] = eval_cost(&positions[p]);
            if costs[p] < best_costs[p] {
                best_positions[p].copy_from_slice(&positions[p]);
                best_costs[p] = costs[p];
                if costs[p] < best_cost {
                    best_cost = costs[p];
                    best_position.copy_from_slice(&positions[p]);
                }
            }
        }
    }

    Ok(WasmPsoResult {
        best_position,
        best_cost,
        iterations: max_iters,
    })
}

/// Run PSO and return per-iteration global best (for trajectory viz).
///
/// Same as `psoMinimize` but also returns `getHistoryPositions()` and `getHistoryCosts()`.
/// Optional `seed` (number or undefined in JS): when provided, used for the RNG; when omitted, the seed is derived from the bounds.
#[must_use]
#[wasm_bindgen(js_name = psoMinimizeWithHistory)]
pub fn pso_minimize_with_history(
    lower: Vec<f64>,
    upper: Vec<f64>,
    num_particles: usize,
    max_iters: u32,
    cost_fn: &js_sys::Function,
    seed: Option<u32>,
) -> Result<WasmPsoResultWithHistory, JsError> {
    if lower.len() != upper.len() {
        return Err(JsError::new(
            "lower and upper bounds must have the same length",
        ));
    }
    if lower.is_empty() {
        return Err(JsError::new("bounds must have at least one dimension"));
    }
    if num_particles < 1 {
        return Err(JsError::new("num_particles must be at least 1"));
    }

    let dim = lower.len();
    let opts = PsoOptions::default();

    let eval_cost = |x: &[f64]| -> f64 {
        let arr = Float64Array::new_with_length(x.len() as u32);
        arr.copy_from(x);
        cost_fn
            .call1(&JsValue::NULL, &arr)
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(f64::INFINITY)
    };

    let seed_u64 = seed
        .map(u64::from)
        .unwrap_or_else(|| seed_from_bounds(&lower, &upper, num_particles));
    let mut rng = XorShift64::new(seed_u64);

    let mut delta = vec![0.0_f64; dim];
    for i in 0..dim {
        delta[i] = upper[i] - lower[i];
    }
    let delta_neg: Vec<f64> = delta.iter().map(|&d| -d).collect();

    let mut positions: Vec<Vec<f64>> = Vec::with_capacity(num_particles);
    let mut velocities: Vec<Vec<f64>> = Vec::with_capacity(num_particles);
    let mut best_positions: Vec<Vec<f64>> = Vec::with_capacity(num_particles);
    let mut costs: Vec<f64> = Vec::with_capacity(num_particles);
    let mut best_costs: Vec<f64> = Vec::with_capacity(num_particles);

    for _ in 0..num_particles {
        let mut pos = vec![0.0_f64; dim];
        let mut vel = vec![0.0_f64; dim];
        rng.uniform_in_bounds(&lower, &upper, &mut pos);
        rng.uniform_in_bounds(&delta_neg, &delta, &mut vel);
        let c = eval_cost(&pos);
        positions.push(pos.clone());
        velocities.push(vel);
        best_positions.push(pos);
        costs.push(c);
        best_costs.push(c);
    }

    let mut best_idx = 0;
    for i in 1..num_particles {
        if costs[i] < costs[best_idx] {
            best_idx = i;
        }
    }
    let mut best_position = best_positions[best_idx].clone();
    let mut best_cost = costs[best_idx];

    let mut history_positions = Vec::with_capacity(max_iters as usize * dim);
    let mut history_costs = Vec::with_capacity(max_iters as usize);

    let mut diff = vec![0.0_f64; dim];
    let mut scaled = vec![0.0_f64; dim];
    let mut new_vel = vec![0.0_f64; dim];
    let mut new_pos = vec![0.0_f64; dim];

    for _ in 0..max_iters {
        history_positions.extend(&best_position);
        history_costs.push(best_cost);

        for p in 0..num_particles {
            for i in 0..dim {
                new_vel[i] = opts.inertia * velocities[p][i];
            }
            for i in 0..dim {
                diff[i] = (best_positions[p][i] - positions[p][i]) * rng.uniform01();
            }
            for i in 0..dim {
                new_vel[i] += opts.cognitive * diff[i];
            }
            for i in 0..dim {
                scaled[i] = (best_position[i] - positions[p][i]) * rng.uniform01();
            }
            for i in 0..dim {
                new_vel[i] += opts.social * scaled[i];
            }
            velocities[p].copy_from_slice(&new_vel);
            for i in 0..dim {
                new_pos[i] = (positions[p][i] + velocities[p][i]).clamp(lower[i], upper[i]);
            }
            positions[p].copy_from_slice(&new_pos);
        }

        for p in 0..num_particles {
            costs[p] = eval_cost(&positions[p]);
            if costs[p] < best_costs[p] {
                best_positions[p].copy_from_slice(&positions[p]);
                best_costs[p] = costs[p];
                if costs[p] < best_cost {
                    best_cost = costs[p];
                    best_position.copy_from_slice(&positions[p]);
                }
            }
        }
    }

    Ok(WasmPsoResultWithHistory {
        best_position,
        best_cost,
        iterations: max_iters,
        history_positions,
        history_costs,
    })
}

/// Backtracking line search: find step length α so that Armijo holds.
///
/// `x` and `d` are the current point and search direction. `f` is the cost at `x`,
/// `g_dot_d` is the gradient at `x` dotted with `d`. `cost_fn` is a JS function
/// `(point: Float64Array) => number` that evaluates the cost at a point.
#[wasm_bindgen(js_name = lineSearchBacktracking)]
pub fn line_search_backtracking(
    x: Vec<f64>,
    d: Vec<f64>,
    f: f64,
    g_dot_d: f64,
    cost_fn: &js_sys::Function,
) -> Result<f64, JsError> {
    if x.len() != d.len() {
        return Err(JsError::new("x and d must have the same length"));
    }
    if x.is_empty() {
        return Err(JsError::new("x and d must not be empty"));
    }
    let mut x_plus_alpha_d = x.clone();
    let eval_cost = |pt: &[f64]| -> f64 {
        let arr = Float64Array::new_with_length(pt.len() as u32);
        arr.copy_from(pt);
        cost_fn
            .call1(&JsValue::NULL, &arr)
            .ok()
            .and_then(|v| v.as_f64())
            .unwrap_or(f64::INFINITY)
    };
    let options = LineSearchOptions::default();
    let alpha = linesearch::backtracking(
        &x,
        &d,
        f,
        g_dot_d,
        &eval_cost,
        &options,
        &mut x_plus_alpha_d,
    );
    Ok(alpha)
}
