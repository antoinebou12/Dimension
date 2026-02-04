//! Simplex method for linear programming.
//!
//! Solves LPs in standard form: minimize c'x subject to Ax = b, x >= 0.
//! Uses a two-phase tableau method with optional SIMD-accelerated hot paths.

mod tableau;

use std::fmt;

use crate::vector::Vector;

pub use tableau::simplex_solve;

/// Status of the simplex solver.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SimplexStatus {
    /// An optimal solution was found.
    Optimal,
    /// The problem is unbounded (objective can go to -infinity).
    Unbounded,
    /// No feasible solution exists (Phase I did not reach zero).
    Infeasible,
}

/// Result of a successful simplex solve.
#[derive(Clone, Debug)]
pub struct SimplexResult {
    /// Primal solution vector (length n, the number of decision variables).
    pub x: Vector<f64>,
    /// Optimal objective value (c'x).
    pub objective: f64,
    /// Solver status.
    pub status: SimplexStatus,
}

/// Error from the simplex solver.
#[derive(Clone, Debug, PartialEq)]
pub enum SimplexError {
    /// Dimensions inconsistent (e.g. A rows != b length, A cols != c length).
    InconsistentDimensions,
    /// Problem has no feasible solution (Phase I failed).
    Infeasible,
    /// Problem is unbounded.
    Unbounded,
    /// Maximum iteration count exceeded (numerical or cycling).
    MaxIterationsExceeded,
}

impl std::error::Error for SimplexError {}

impl fmt::Display for SimplexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimplexError::InconsistentDimensions => write!(f, "inconsistent dimensions"),
            SimplexError::Infeasible => write!(f, "problem is infeasible"),
            SimplexError::Unbounded => write!(f, "problem is unbounded"),
            SimplexError::MaxIterationsExceeded => write!(f, "max iterations exceeded"),
        }
    }
}
