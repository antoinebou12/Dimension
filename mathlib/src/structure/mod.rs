//! Data structures and storage layouts: types, dense/sparse storage, matrix base, cube, sparse formats.

pub mod cube_base;
pub mod cube_slice;
pub mod dense_storage;
pub mod matrix_base;
pub mod sparse;
pub mod sparse_formats;
pub mod submatrix;
pub mod types;

pub use cube_base::CubeBase;
pub use cube_slice::CubeSlice;
pub use dense_storage::{DenseStorage, DenseStorageDynamic};
pub use matrix_base::MatrixBase;
pub use sparse::{SparseMatrix, SparseMatrixBase, SparseMatrixCRS, SparseStorage};
pub use sparse_formats::{
    SparseMatrixBCRS, SparseMatrixCCS, SparseMatrixCDS, SparseMatrixJDS, SparseMatrixSKS,
};
pub use submatrix::SubMatrix;
pub use types::{COLUMN_STORAGE, Dynamic, Fill, ROW_STORAGE, Storage, Triplet};
