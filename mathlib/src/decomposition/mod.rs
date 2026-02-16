//! Matrix decompositions: SVD, PCA (PCA via SVD), t-SNE, UMAP, Procrustes.

pub mod pca;
pub mod procrustes;
pub mod svd;
pub mod tsne;
pub mod umap;

pub use pca::{Pca, pca};
pub use procrustes::procrustes_orthogonal;
pub use svd::{Svd, SvdEcon, pinv, svd_econ};
pub use tsne::{TsneOptions, tsne};
pub use umap::{UmapOptions, umap};
