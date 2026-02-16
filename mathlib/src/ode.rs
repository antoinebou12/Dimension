//! Simple ODE time-stepping (Chapter 15).
//!
//! Single-step integrators for y' = f(t, y). State `y` is a slice; the stepper updates it in place.
//! Full integrators `euler` and `rk4` return trajectories (t, y).

/// Result of an ODE integration: time points and state at each step.
#[derive(Clone, Debug)]
pub struct OdeResult {
    /// Time points t[0], t[1], ...
    pub t: Vec<f64>,
    /// State at each time: y[i] is the state vector at t[i].
    pub y: Vec<Vec<f64>>,
}

/// Forward Euler: y_{n+1} = y_n + dt * f(t_n, y_n).
///
/// Updates `y` in place. `f` is (t, y) -> dy/dt; `y` must have length matching the ODE dimension.
pub fn euler_step<F>(f: F, t: f64, y: &mut [f64], dt: f64)
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let dydt = f(t, y);
    assert_eq!(dydt.len(), y.len());
    for (yi, &dy) in y.iter_mut().zip(dydt.iter()) {
        *yi += dt * dy;
    }
}

/// Trapezoidal (Heun) step: y_{n+1} = y_n + (dt/2) * (f(t_n, y_n) + f(t_n+dt, y_euler)).
///
/// Uses one Euler step for the predictor, then trapezoidal corrector.
pub fn trapezoidal_step<F>(f: F, t: f64, y: &mut [f64], dt: f64)
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let dydt0 = f(t, y);
    assert_eq!(dydt0.len(), y.len());
    let y_pred: Vec<f64> = y
        .iter()
        .zip(dydt0.iter())
        .map(|(a, &b)| a + dt * b)
        .collect();
    let dydt1 = f(t + dt, &y_pred);
    for (yi, (&k0, &k1)) in y.iter_mut().zip(dydt0.iter().zip(dydt1.iter())) {
        *yi += dt * 0.5 * (k0 + k1);
    }
}

/// RK4 (classic Runge-Kutta) step: y_{n+1} = y_n + (dt/6)(k1 + 2k2 + 2k3 + k4).
pub fn rk4_step<F>(f: F, t: f64, y: &mut [f64], dt: f64)
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let dim = y.len();
    let k1 = f(t, y);
    assert_eq!(k1.len(), dim);
    let y2: Vec<f64> = y
        .iter()
        .zip(k1.iter())
        .map(|(yi, &k)| yi + dt * 0.5 * k)
        .collect();
    let k2 = f(t + dt * 0.5, &y2);
    let y3: Vec<f64> = y
        .iter()
        .zip(k2.iter())
        .map(|(yi, &k)| yi + dt * 0.5 * k)
        .collect();
    let k3 = f(t + dt * 0.5, &y3);
    let y4: Vec<f64> = y
        .iter()
        .zip(k3.iter())
        .map(|(yi, &k)| yi + dt * k)
        .collect();
    let k4 = f(t + dt, &y4);
    for (yi, (k1i, (k2i, (k3i, k4i)))) in y
        .iter_mut()
        .zip(k1.iter().zip(k2.iter().zip(k3.iter().zip(k4.iter()))))
    {
        *yi += dt * (*k1i + 2.0 * *k2i + 2.0 * *k3i + *k4i) / 6.0;
    }
}

/// Forward Euler integration: compute y(t) for t = t0, t0+dt, ..., t0+n*dt.
#[must_use]
pub fn euler<F>(f: F, y0: &[f64], t0: f64, dt: f64, n: usize) -> OdeResult
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let mut t = Vec::with_capacity(n + 1);
    let mut y = Vec::with_capacity(n + 1);
    t.push(t0);
    y.push(y0.to_vec());
    let mut y_cur = y0.to_vec();
    for i in 0..n {
        euler_step(&f, t[i], &mut y_cur, dt);
        t.push(t0 + (i + 1) as f64 * dt);
        y.push(y_cur.clone());
    }
    OdeResult { t, y }
}

/// RK4 integration: compute y(t) for t = t0, t0+dt, ..., t0+n*dt.
#[must_use]
pub fn rk4<F>(f: F, y0: &[f64], t0: f64, dt: f64, n: usize) -> OdeResult
where
    F: Fn(f64, &[f64]) -> Vec<f64>,
{
    let mut t = Vec::with_capacity(n + 1);
    let mut y = Vec::with_capacity(n + 1);
    t.push(t0);
    y.push(y0.to_vec());
    let mut y_cur = y0.to_vec();
    for i in 0..n {
        rk4_step(&f, t[i], &mut y_cur, dt);
        t.push(t0 + (i + 1) as f64 * dt);
        y.push(y_cur.clone());
    }
    OdeResult { t, y }
}
