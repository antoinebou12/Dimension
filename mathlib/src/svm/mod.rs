//! Support Vector Machine (SVM) for binary classification.
//!
//! Data layout: rows = samples, cols = features (same as [`crate::clustering`]).
//! Labels are ±1. Uses the dual formulation and Sequential Minimal Optimization (SMO).
//! Linear kernel and RBF (Radial Basis Function) kernel are supported.

pub mod linear;
pub mod rbf;

pub use linear::{SvmError, SvmOptions, SvmResult, svm};
pub use rbf::{SvmRbfResult, svm_rbf};
