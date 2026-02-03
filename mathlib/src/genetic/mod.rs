//! Genetic and evolution-strategy algorithms: CMA-ES.
//!
//! Requires the `genetic` feature. Uses `simd` and `parallel` features when enabled
//! for weighted recombination and parallel fitness evaluation.

pub mod cmaes;

pub use cmaes::{CmaEs, CmaEsBuilder, CmaEsResult};
