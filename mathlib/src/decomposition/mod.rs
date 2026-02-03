//! Matrix decompositions: SVD and PCA (PCA via SVD).

pub mod pca;
pub mod svd;
pub use pca::{Pca, pca};
pub use svd::{Svd, SvdEcon, svd_econ};
