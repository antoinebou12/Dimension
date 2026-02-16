//! Optimization: line search, gradient descent, conjugate gradient, Gauss-Newton, Muon, PSO.
//!
//! Set `RUST_LOG=mathlib=debug` to see iteration logs (cost, gradient norm, step size, backtracks).

pub mod conjugate_gradient;
pub mod gauss_newton;
pub mod gradient_descent;
pub mod lbfgsb;
pub mod levenberg_marquardt;
pub mod linesearch;
pub mod muon;
pub mod pso;

pub use conjugate_gradient::{
    CgError, NonlinearCgOptions, NonlinearCgResult, nonlinear_cg, solve_cg, solve_cg_sparse,
};
pub use gauss_newton::{GaussNewtonOptions, GaussNewtonResult, gauss_newton};
pub use gradient_descent::{
    GradientDescentOptions, GradientDescentResult, LineSearchVariant, gradient_descent,
};
pub use lbfgsb::{LbfgsbOptions, LbfgsbResult, lbfgsb};
pub use levenberg_marquardt::{
    LevenbergMarquardtOptions, LevenbergMarquardtResult, levenberg_marquardt,
};
pub use linesearch::{LineSearchOptions, armijo, backtracking, wolfe};
pub use muon::muon_step;
pub use pso::{PsoOptions, PsoResult, pso, pso_sequential};
