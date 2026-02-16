//! WASM bindings for roots, quadrature, differentiation, ODE.

use js_sys::Float64Array;
use wasm_bindgen::prelude::*;

use crate::rootfinding::{bisection, brent, newton_1d, secant};
use crate::{differentiation, ode, quadrature};

/// Result of root-finding: x, fx, iterations, converged.
#[wasm_bindgen]
pub struct WasmRootResult {
    x: f64,
    fx: f64,
    iterations: u32,
    converged: bool,
}

#[wasm_bindgen]
impl WasmRootResult {
    #[wasm_bindgen(js_name = getX)]
    pub fn get_x(&self) -> f64 {
        self.x
    }
    #[wasm_bindgen(js_name = getFx)]
    pub fn get_fx(&self) -> f64 {
        self.fx
    }
    #[wasm_bindgen(js_name = getIterations)]
    pub fn get_iterations(&self) -> u32 {
        self.iterations
    }
    #[wasm_bindgen(js_name = getConverged)]
    pub fn get_converged(&self) -> bool {
        self.converged
    }
}

fn eval_f(f: &js_sys::Function, x: f64) -> f64 {
    f.call1(&JsValue::NULL, &JsValue::from_f64(x))
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(f64::NAN)
}

fn eval_df(df: &js_sys::Function, x: f64) -> f64 {
    df.call1(&JsValue::NULL, &JsValue::from_f64(x))
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(f64::NAN)
}

/// Bisection: f(a)*f(b) < 0. Returns RootResult; converged=false if invalid bracket.
#[wasm_bindgen(js_name = bisectionRoot)]
pub fn bisection_wasm(f: &js_sys::Function, a: f64, b: f64, tol: f64) -> WasmRootResult {
    let f_clone = |x: f64| eval_f(f, x);
    match bisection(f_clone, a, b, tol) {
        Some(x) => WasmRootResult {
            x,
            fx: eval_f(f, x),
            iterations: 0, // bisection doesn't return iter count
            converged: true,
        },
        None => WasmRootResult {
            x: a,
            fx: eval_f(f, a),
            iterations: 0,
            converged: false,
        },
    }
}

/// Bisection with iteration tracking. Uses internal loop.
#[wasm_bindgen(js_name = bisectionRootResult)]
pub fn bisection_result_wasm(
    f: &js_sys::Function,
    a: f64,
    b: f64,
    tol: f64,
    max_iter: u32,
) -> WasmRootResult {
    let f_clone = |x: f64| eval_f(f, x);
    let mut lo = a;
    let mut hi = b;
    let fa = f_clone(lo);
    let fb = f_clone(hi);
    if fa * fb >= 0.0 {
        return WasmRootResult {
            x: lo,
            fx: fa,
            iterations: 0,
            converged: false,
        };
    }
    for iter in 0..max_iter {
        let mid = 0.5 * (lo + hi);
        let fm = f_clone(mid);
        if fm.abs() <= tol || (hi - lo) <= tol {
            return WasmRootResult {
                x: mid,
                fx: fm,
                iterations: iter + 1,
                converged: true,
            };
        }
        if fa * fm < 0.0 {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    let mid = 0.5 * (lo + hi);
    WasmRootResult {
        x: mid,
        fx: f_clone(mid),
        iterations: max_iter,
        converged: false,
    }
}

/// Newton's method. f: x => f(x), df: x => f'(x).
#[wasm_bindgen(js_name = newtonRoot)]
pub fn newton_wasm(
    f: &js_sys::Function,
    df: &js_sys::Function,
    x0: f64,
    tol: f64,
    max_iter: u32,
) -> WasmRootResult {
    let f_clone = |x: f64| eval_f(f, x);
    let df_clone = |x: f64| eval_df(df, x);
    match newton_1d(f_clone, df_clone, x0, tol, max_iter as usize) {
        Some(x) => WasmRootResult {
            x,
            fx: eval_f(f, x),
            iterations: 0,
            converged: true,
        },
        None => WasmRootResult {
            x: x0,
            fx: eval_f(f, x0),
            iterations: max_iter,
            converged: false,
        },
    }
}

/// Secant method. f: x => f(x).
#[wasm_bindgen(js_name = secantRoot)]
pub fn secant_wasm(
    f: &js_sys::Function,
    x0: f64,
    x1: f64,
    tol: f64,
    max_iter: u32,
) -> WasmRootResult {
    let f_clone = |x: f64| eval_f(f, x);
    let r = secant(f_clone, x0, x1, tol, max_iter as usize);
    WasmRootResult {
        x: r.x,
        fx: r.fx,
        iterations: r.iterations,
        converged: r.converged,
    }
}

/// Brent's method. f(a)*f(b) < 0.
#[wasm_bindgen(js_name = brentRoot)]
pub fn brent_wasm(f: &js_sys::Function, a: f64, b: f64, tol: f64, max_iter: u32) -> WasmRootResult {
    let f_clone = |x: f64| eval_f(f, x);
    let r = brent(f_clone, a, b, tol, max_iter as usize);
    WasmRootResult {
        x: r.x,
        fx: r.fx,
        iterations: r.iterations,
        converged: r.converged,
    }
}

/// Trapezoidal quadrature. f: x => f(x).
#[wasm_bindgen(js_name = trapezoidalQuad)]
pub fn trapezoidal_wasm(f: &js_sys::Function, a: f64, b: f64, n: u32) -> f64 {
    let f_clone = |x: f64| eval_f(f, x);
    quadrature::trapezoidal(f_clone, a, b, n as usize)
}

/// Simpson quadrature. n must be even.
#[wasm_bindgen(js_name = simpsonQuad)]
pub fn simpson_wasm(f: &js_sys::Function, a: f64, b: f64, n: u32) -> f64 {
    let f_clone = |x: f64| eval_f(f, x);
    quadrature::simpson(f_clone, a, b, n as usize)
}

/// Gauss-Legendre quadrature.
#[wasm_bindgen(js_name = gaussLegendreQuad)]
pub fn gauss_legendre_wasm(f: &js_sys::Function, a: f64, b: f64, n: u32) -> f64 {
    let f_clone = |x: f64| eval_f(f, x);
    quadrature::gauss_legendre(f_clone, a, b, n as usize)
}

/// Central difference derivative at x with step h.
#[wasm_bindgen(js_name = diffCentral)]
pub fn diff_central_wasm(f: &js_sys::Function, x: f64, h: f64) -> f64 {
    let f_clone = |t: f64| eval_f(f, t);
    differentiation::diff_central(f_clone, x, h)
}

/// ODE result: t and y arrays.
#[wasm_bindgen]
pub struct WasmOdeResult {
    t: Vec<f64>,
    y: Vec<Vec<f64>>,
}

#[wasm_bindgen]
impl WasmOdeResult {
    #[wasm_bindgen(js_name = getT)]
    pub fn get_t(&self) -> Vec<f64> {
        self.t.clone()
    }
    #[wasm_bindgen(js_name = getY)]
    pub fn get_y(&self) -> Vec<f64> {
        self.y.iter().flatten().cloned().collect()
    }
    #[wasm_bindgen(js_name = getYAt)]
    pub fn get_y_at(&self, i: usize) -> Option<Vec<f64>> {
        self.y.get(i).cloned()
    }
}

/// Euler ODE integration. dydt: (t, y) => dy/dt. y0 as Float64Array.
#[wasm_bindgen(js_name = eulerOde)]
pub fn euler_wasm(
    dydt: &js_sys::Function,
    y0: &Float64Array,
    t0: f64,
    dt: f64,
    n: u32,
) -> WasmOdeResult {
    let y0_vec: Vec<f64> = y0.to_vec();
    let f = |t: f64, y: &[f64]| {
        let arr = Float64Array::new_with_length(y.len() as u32);
        arr.copy_from(y);
        let out = dydt.call2(&JsValue::NULL, &JsValue::from_f64(t), &arr).ok();
        if let Some(v) = out {
            if let Some(arr) = v.dyn_ref::<Float64Array>() {
                return arr.to_vec();
            }
        }
        vec![0.0; y.len()]
    };
    let result = ode::euler(&f, &y0_vec, t0, dt, n as usize);
    WasmOdeResult {
        t: result.t,
        y: result.y,
    }
}

/// RK4 ODE integration.
#[wasm_bindgen(js_name = rk4Ode)]
pub fn rk4_wasm(
    dydt: &js_sys::Function,
    y0: &Float64Array,
    t0: f64,
    dt: f64,
    n: u32,
) -> WasmOdeResult {
    let y0_vec: Vec<f64> = y0.to_vec();
    let f = |t: f64, y: &[f64]| {
        let arr = Float64Array::new_with_length(y.len() as u32);
        arr.copy_from(y);
        let out = dydt.call2(&JsValue::NULL, &JsValue::from_f64(t), &arr).ok();
        if let Some(v) = out {
            if let Some(arr) = v.dyn_ref::<Float64Array>() {
                return arr.to_vec();
            }
        }
        vec![0.0; y.len()]
    };
    let result = ode::rk4(&f, &y0_vec, t0, dt, n as usize);
    WasmOdeResult {
        t: result.t,
        y: result.y,
    }
}
