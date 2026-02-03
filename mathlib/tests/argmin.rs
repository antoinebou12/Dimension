//! Integration tests for argmin (line search, gradient descent, CG, Gauss-Newton, Muon, PSO).

#[path = "argmin/conjugate_gradient.rs"]
mod conjugate_gradient;
#[path = "argmin/gauss_newton.rs"]
mod gauss_newton;
#[path = "argmin/gradient_descent.rs"]
mod gradient_descent;
#[path = "argmin/linesearch.rs"]
mod linesearch;
#[path = "argmin/muon.rs"]
mod muon;
#[path = "argmin/pso.rs"]
mod pso;
