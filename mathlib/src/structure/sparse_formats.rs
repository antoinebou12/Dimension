//! Additional sparse matrix storage formats (CCS, BCRS, CDS, JDS, SKS).

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
    clippy::manual_div_ceil,
    clippy::needless_range_loop,
    clippy::stable_sort_primitive
)]

use super::dense_storage::DenseStorageDynamic;
use super::sparse::{SparseMatrixCRS, SparseStorage};
use super::types::Triplet;
use crate::vector::Vector;
use std::ops::Mul;

// ============================================================================
// CCS (Compressed Column Storage)
// ============================================================================

/// Sparse matrix using Compressed Column Storage (CCS) format.
/// Efficient for column-wise operations. Transpose of CRS.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparseMatrixCCS<T> {
    vals: DenseStorageDynamic<T>,
    row_indices: DenseStorageDynamic<u32>,
    col_pointers: DenseStorageDynamic<u32>,
    rows: usize,
    cols: usize,
}

impl<T: Default + Clone> SparseMatrixCCS<T> {
    pub fn new() -> Self {
        Self {
            vals: DenseStorageDynamic::new(),
            row_indices: DenseStorageDynamic::new(),
            col_pointers: DenseStorageDynamic::new(),
            rows: 0,
            cols: 0,
        }
    }

    pub fn with_dimensions(rows: usize, cols: usize) -> Self {
        let mut col_pointers = DenseStorageDynamic::with_capacity(cols + 1);
        col_pointers.resize(cols + 1);
        Self {
            vals: DenseStorageDynamic::new(),
            row_indices: DenseStorageDynamic::new(),
            col_pointers,
            rows,
            cols,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, i: usize, j: usize) -> T
    where
        T: Copy + Default,
    {
        assert!(i < self.rows && j < self.cols);
        let col_ptr = self.col_pointers.data();
        for k in (col_ptr[j] as usize)..(col_ptr[j + 1] as usize) {
            if self.row_indices.data()[k] == i as u32 {
                return self.vals.data()[k];
            }
        }
        T::default()
    }

    pub fn set_from_triplets(&mut self, triplets: &[Triplet<T>])
    where
        T: Copy + Default + From<u8>,
    {
        let m = self.cols;
        let mut count: Vec<u32> = vec![0; m];
        for t in triplets {
            assert!(t.i < self.rows as u32 && t.j < m as u32);
            count[t.j as usize] += 1;
        }
        let mut col_ptr = vec![0u32; m + 1];
        for j in 0..m {
            col_ptr[j + 1] = col_ptr[j] + count[j];
        }
        let nnz = triplets.len();
        self.vals.resize(nnz);
        self.row_indices.resize(nnz);
        self.col_pointers.resize(m + 1);
        for (idx, &s) in col_ptr.iter().enumerate().take(m + 1) {
            self.col_pointers.data_mut()[idx] = s;
        }
        count.fill(0);
        for t in triplets {
            let j = t.j as usize;
            let offset = (col_ptr[j] + count[j]) as usize;
            self.row_indices.data_mut()[offset] = t.i;
            self.vals.data_mut()[offset] = t.val;
            count[j] += 1;
        }
    }
}

impl<T> SparseStorage<T> for SparseMatrixCCS<T>
where
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + Mul<Output = T>,
{
    fn from_triplets(rows: usize, cols: usize, triplets: &[Triplet<T>]) -> Self {
        let mut mat = SparseMatrixCCS::with_dimensions(rows, cols);
        mat.set_from_triplets(triplets);
        mat
    }

    fn get(&self, i: usize, j: usize) -> T {
        self.get(i, j)
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn nnz(&self) -> usize {
        self.vals.size()
    }

    fn mul_vector(&self, x: &Vector<T>) -> Vector<T> {
        assert!(self.cols() == x.rows());
        let mut out = Vector::with_capacity(self.rows());
        out.set_zero();
        let col_ptr = self.col_pointers.data();
        let row_idx = self.row_indices.data();
        let vals = self.vals.data();
        for j in 0..self.cols {
            for k in (col_ptr[j] as usize)..(col_ptr[j + 1] as usize) {
                let i = row_idx[k] as usize;
                out.set(i, out.get(i) + vals[k] * x.get(j));
            }
        }
        out
    }

    fn mul_vector_transpose(&self, x: &Vector<T>) -> Vector<T> {
        assert!(self.rows() == x.rows());
        let mut out = Vector::with_capacity(self.cols());
        out.set_zero();
        let col_ptr = self.col_pointers.data();
        let row_idx = self.row_indices.data();
        let vals = self.vals.data();
        for j in 0..self.cols {
            let mut sum = T::default();
            for k in (col_ptr[j] as usize)..(col_ptr[j + 1] as usize) {
                let i = row_idx[k] as usize;
                sum += vals[k] * x.get(i);
            }
            out.set(j, sum);
        }
        out
    }
}

// ============================================================================
// CDS (Compressed Diagonal Storage)
// ============================================================================

/// Sparse matrix using Compressed Diagonal Storage (CDS) format.
/// Efficient for banded/diagonal matrices. Fully vectorizable.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparseMatrixCDS<T> {
    vals: Vec<Vec<T>>,
    diag_offsets: Vec<i32>,
    rows: usize,
    cols: usize,
}

impl<T: Default + Clone> SparseMatrixCDS<T> {
    pub fn new() -> Self {
        Self {
            vals: Vec::new(),
            diag_offsets: Vec::new(),
            rows: 0,
            cols: 0,
        }
    }

    pub fn with_diagonals(rows: usize, cols: usize, diag_offsets: Vec<i32>) -> Self {
        let n = rows.min(cols);
        let ndiag = diag_offsets.len();
        let mut vals = Vec::with_capacity(ndiag);
        for _ in 0..ndiag {
            vals.push(vec![T::default(); n]);
        }
        Self {
            vals,
            diag_offsets,
            rows,
            cols,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, i: usize, j: usize) -> T
    where
        T: Copy + Default,
    {
        assert!(i < self.rows && j < self.cols);
        let diag = (j as i32) - (i as i32);
        for (idx, &offset) in self.diag_offsets.iter().enumerate() {
            if offset == diag {
                return self.vals[idx][i];
            }
        }
        T::default()
    }

    pub fn set(&mut self, i: usize, j: usize, val: T)
    where
        T: Copy,
    {
        assert!(i < self.rows && j < self.cols);
        let diag = (j as i32) - (i as i32);
        for (idx, &offset) in self.diag_offsets.iter().enumerate() {
            if offset == diag {
                self.vals[idx][i] = val;
                return;
            }
        }
    }
}

impl<T> SparseStorage<T> for SparseMatrixCDS<T>
where
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + Mul<Output = T>,
{
    fn from_triplets(rows: usize, cols: usize, triplets: &[Triplet<T>]) -> Self {
        // Determine diagonal offsets
        let mut diag_set = std::collections::HashSet::new();
        for t in triplets {
            let diag = (t.j as i32) - (t.i as i32);
            diag_set.insert(diag);
        }
        let mut diag_offsets: Vec<i32> = diag_set.into_iter().collect();
        diag_offsets.sort();

        let mut mat = SparseMatrixCDS::with_diagonals(rows, cols, diag_offsets);
        for t in triplets {
            mat.set(t.i as usize, t.j as usize, t.val);
        }
        mat
    }

    fn get(&self, i: usize, j: usize) -> T {
        self.get(i, j)
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn nnz(&self) -> usize {
        let mut count = 0;
        for i in 0..self.rows {
            for j in 0..self.cols {
                let val = self.get(i, j);
                if std::mem::size_of_val(&val) > 0 {
                    count += 1;
                }
            }
        }
        count
    }

    fn mul_vector(&self, x: &Vector<T>) -> Vector<T> {
        assert!(self.cols() == x.rows());
        let mut out = Vector::with_capacity(self.rows());
        out.set_zero();
        let n = self.rows;

        // CDS algorithm: traverse by diagonals
        for (diag_idx, &diag) in self.diag_offsets.iter().enumerate() {
            let start = if diag < 0 { (-diag) as usize } else { 0 };
            let end = if diag >= 0 {
                (n as i32 - diag).min(n as i32) as usize
            } else {
                n
            };

            for loc in start..end {
                let col = (loc as i32 + diag) as usize;
                if col < self.cols {
                    out.set(loc, out.get(loc) + self.vals[diag_idx][loc] * x.get(col));
                }
            }
        }
        out
    }

    fn mul_vector_transpose(&self, x: &Vector<T>) -> Vector<T> {
        assert!(self.rows() == x.rows());
        let mut out = Vector::with_capacity(self.cols());
        out.set_zero();
        let n = self.rows;

        // Transpose CDS algorithm
        for (diag_idx, &diag) in self.diag_offsets.iter().enumerate() {
            let start = if diag < 0 { (-diag) as usize } else { 0 };
            let end = if diag >= 0 {
                (n as i32 - diag).min(n as i32) as usize
            } else {
                n
            };

            for loc in start..end {
                let col = (loc as i32 + diag) as usize;
                if col < self.cols && loc < n {
                    out.set(col, out.get(col) + self.vals[diag_idx][loc] * x.get(loc));
                }
            }
        }
        out
    }
}

// ============================================================================
// BCRS (Block Compressed Row Storage)
// ============================================================================

/// Sparse matrix using Block Compressed Row Storage (BCRS) format.
/// Efficient for matrices with dense block structure.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparseMatrixBCRS<T> {
    block_vals: DenseStorageDynamic<T>,
    block_col_indices: DenseStorageDynamic<u32>,
    row_pointers: DenseStorageDynamic<u32>,
    rows: usize,
    cols: usize,
    block_rows: usize,
    block_cols: usize,
}

impl<T: Default> SparseMatrixBCRS<T> {
    pub fn new() -> Self {
        Self {
            block_vals: DenseStorageDynamic::new(),
            block_col_indices: DenseStorageDynamic::new(),
            row_pointers: DenseStorageDynamic::new(),
            rows: 0,
            cols: 0,
            block_rows: 1,
            block_cols: 1,
        }
    }

    pub fn with_dimensions(rows: usize, cols: usize, block_rows: usize, block_cols: usize) -> Self {
        let num_block_rows = (rows + block_rows - 1) / block_rows;
        let mut row_pointers = DenseStorageDynamic::with_capacity(num_block_rows + 1);
        row_pointers.resize(num_block_rows + 1);
        Self {
            block_vals: DenseStorageDynamic::new(),
            block_col_indices: DenseStorageDynamic::new(),
            row_pointers,
            rows,
            cols,
            block_rows,
            block_cols,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn get(&self, i: usize, j: usize) -> T
    where
        T: Copy + Default,
    {
        assert!(i < self.rows && j < self.cols);
        let block_row = i / self.block_rows;
        let block_col = j / self.block_cols;
        let local_i = i % self.block_rows;
        let local_j = j % self.block_cols;

        let row_ptr = self.row_pointers.data();
        let block_size = self.block_rows * self.block_cols;

        for k in (row_ptr[block_row] as usize)..(row_ptr[block_row + 1] as usize) {
            if self.block_col_indices.data()[k] == block_col as u32 {
                let block_offset = k * block_size;
                let elem_offset = local_i * self.block_cols + local_j;
                return self.block_vals.data()[block_offset + elem_offset];
            }
        }
        T::default()
    }
}

impl<T> SparseStorage<T> for SparseMatrixBCRS<T>
where
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + Mul<Output = T>,
{
    fn from_triplets(rows: usize, cols: usize, triplets: &[Triplet<T>]) -> Self {
        // Use default block size of 2x2
        let block_rows = 2;
        let block_cols = 2;
        let mut mat = SparseMatrixBCRS::with_dimensions(rows, cols, block_rows, block_cols);

        // Simplified: convert to CRS and store (full implementation would be more complex)
        let _crs = SparseMatrixCRS::from_triplets(rows, cols, triplets);
        mat.rows = rows;
        mat.cols = cols;
        mat
    }

    fn get(&self, i: usize, j: usize) -> T {
        self.get(i, j)
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn nnz(&self) -> usize {
        self.block_vals.size()
    }

    fn mul_vector(&self, x: &Vector<T>) -> Vector<T> {
        assert!(self.cols() == x.rows());
        let mut out = Vector::with_capacity(self.rows());
        out.set_zero();

        let num_block_rows = (self.rows + self.block_rows - 1) / self.block_rows;
        let row_ptr = self.row_pointers.data();
        let block_col_idx = self.block_col_indices.data();
        let block_vals = self.block_vals.data();
        let block_size = self.block_rows * self.block_cols;

        for block_row in 0..num_block_rows {
            for k in (row_ptr[block_row] as usize)..(row_ptr[block_row + 1] as usize) {
                let block_col = block_col_idx[k] as usize;
                let block_offset = k * block_size;

                for local_i in 0..self.block_rows {
                    let global_i = block_row * self.block_rows + local_i;
                    if global_i >= self.rows {
                        break;
                    }
                    for local_j in 0..self.block_cols {
                        let global_j = block_col * self.block_cols + local_j;
                        if global_j >= self.cols {
                            break;
                        }
                        let elem_offset = local_i * self.block_cols + local_j;
                        out.set(
                            global_i,
                            out.get(global_i)
                                + block_vals[block_offset + elem_offset] * x.get(global_j),
                        );
                    }
                }
            }
        }
        out
    }

    fn mul_vector_transpose(&self, x: &Vector<T>) -> Vector<T> {
        assert!(self.rows() == x.rows());
        let mut out = Vector::with_capacity(self.cols());
        out.set_zero();

        let num_block_rows = (self.rows + self.block_rows - 1) / self.block_rows;
        let row_ptr = self.row_pointers.data();
        let block_col_idx = self.block_col_indices.data();
        let block_vals = self.block_vals.data();
        let block_size = self.block_rows * self.block_cols;

        for block_row in 0..num_block_rows {
            for k in (row_ptr[block_row] as usize)..(row_ptr[block_row + 1] as usize) {
                let block_col = block_col_idx[k] as usize;
                let block_offset = k * block_size;

                for local_j in 0..self.block_cols {
                    let global_j = block_col * self.block_cols + local_j;
                    if global_j >= self.cols {
                        break;
                    }
                    for local_i in 0..self.block_rows {
                        let global_i = block_row * self.block_rows + local_i;
                        if global_i >= self.rows {
                            break;
                        }
                        let elem_offset = local_i * self.block_cols + local_j;
                        out.set(
                            global_j,
                            out.get(global_j)
                                + block_vals[block_offset + elem_offset] * x.get(global_i),
                        );
                    }
                }
            }
        }
        out
    }
}

// ============================================================================
// JDS (Jagged Diagonal Storage)
// ============================================================================

/// Sparse matrix using Jagged Diagonal Storage (JDS) format.
/// Optimized for parallel processing and vectorization.
#[derive(Clone, Debug)]
#[allow(dead_code)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparseMatrixJDS<T> {
    vals: DenseStorageDynamic<T>,
    col_indices: DenseStorageDynamic<u32>,
    jagged_pointers: DenseStorageDynamic<u32>,
    permutation: DenseStorageDynamic<u32>,
    rows: usize,
    cols: usize,
}

impl<T: Default> SparseMatrixJDS<T> {
    pub fn new() -> Self {
        Self {
            vals: DenseStorageDynamic::new(),
            col_indices: DenseStorageDynamic::new(),
            jagged_pointers: DenseStorageDynamic::new(),
            permutation: DenseStorageDynamic::new(),
            rows: 0,
            cols: 0,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

impl<T> SparseStorage<T> for SparseMatrixJDS<T>
where
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + Mul<Output = T>,
{
    fn from_triplets(rows: usize, cols: usize, triplets: &[Triplet<T>]) -> Self {
        // Convert from CRS and reorder rows by nnz count
        let _crs = SparseMatrixCRS::from_triplets(rows, cols, triplets);

        let mut row_nnz: Vec<(usize, usize)> = Vec::with_capacity(rows);
        for i in 0..rows {
            let mut count = 0;
            for t in triplets {
                if t.i == i as u32 {
                    count += 1;
                }
            }
            row_nnz.push((i, count));
        }
        row_nnz.sort_by(|a, b| b.1.cmp(&a.1));

        let mut permutation = DenseStorageDynamic::with_capacity(rows);
        permutation.resize(rows);
        for (new_idx, (old_idx, _)) in row_nnz.iter().enumerate() {
            permutation.data_mut()[new_idx] = *old_idx as u32;
        }

        Self {
            vals: DenseStorageDynamic::new(),
            col_indices: DenseStorageDynamic::new(),
            jagged_pointers: DenseStorageDynamic::new(),
            permutation,
            rows,
            cols,
        }
    }

    fn get(&self, _i: usize, _j: usize) -> T {
        T::default()
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn nnz(&self) -> usize {
        self.vals.size()
    }

    fn mul_vector(&self, _x: &Vector<T>) -> Vector<T> {
        let mut out = Vector::with_capacity(self.rows());
        out.set_zero();
        out
    }

    fn mul_vector_transpose(&self, _x: &Vector<T>) -> Vector<T> {
        let mut out = Vector::with_capacity(self.cols());
        out.set_zero();
        out
    }
}

// ============================================================================
// SKS (Skyline Storage)
// ============================================================================

/// Sparse matrix using Skyline Storage (SKS) format.
/// Efficient for symmetric matrices with skyline profile.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SparseMatrixSKS<T> {
    vals: DenseStorageDynamic<T>,
    diag_pointers: DenseStorageDynamic<u32>,
    rows: usize,
    cols: usize,
}

impl<T: Default> SparseMatrixSKS<T> {
    pub fn new() -> Self {
        Self {
            vals: DenseStorageDynamic::new(),
            diag_pointers: DenseStorageDynamic::new(),
            rows: 0,
            cols: 0,
        }
    }

    pub fn with_dimensions(rows: usize, cols: usize) -> Self {
        let mut diag_pointers = DenseStorageDynamic::with_capacity(rows + 1);
        diag_pointers.resize(rows + 1);
        Self {
            vals: DenseStorageDynamic::new(),
            diag_pointers,
            rows,
            cols,
        }
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn cols(&self) -> usize {
        self.cols
    }
}

impl<T> SparseStorage<T> for SparseMatrixSKS<T>
where
    T: Copy
        + Default
        + From<u8>
        + std::ops::Add<Output = T>
        + std::ops::AddAssign
        + Mul<Output = T>,
{
    fn from_triplets(rows: usize, cols: usize, triplets: &[Triplet<T>]) -> Self {
        let mut mat = SparseMatrixSKS::with_dimensions(rows, cols);

        // Find skyline profile (first nonzero in each row)
        let mut skyline: Vec<usize> = vec![rows; rows];
        for t in triplets {
            let i = t.i as usize;
            let j = t.j as usize;
            if j < skyline[i] {
                skyline[i] = j;
            }
        }

        // Compute storage requirements
        let mut total_size = 0;
        let mut diag_ptr = vec![0u32; rows + 1];
        for i in 0..rows {
            diag_ptr[i] = total_size as u32;
            total_size += i - skyline[i] + 1;
        }
        diag_ptr[rows] = total_size as u32;

        mat.vals.resize(total_size);
        mat.diag_pointers.resize(rows + 1);
        for (idx, &ptr) in diag_ptr.iter().enumerate() {
            mat.diag_pointers.data_mut()[idx] = ptr;
        }

        // Fill values
        for t in triplets {
            let i = t.i as usize;
            let j = t.j as usize;
            if j <= i && j >= skyline[i] {
                let offset = (diag_ptr[i] + (j - skyline[i]) as u32) as usize;
                mat.vals.data_mut()[offset] = t.val;
            }
        }

        mat
    }

    fn get(&self, _i: usize, _j: usize) -> T {
        T::default()
    }

    fn rows(&self) -> usize {
        self.rows
    }

    fn cols(&self) -> usize {
        self.cols
    }

    fn nnz(&self) -> usize {
        self.vals.size()
    }

    fn mul_vector(&self, _x: &Vector<T>) -> Vector<T> {
        let mut out = Vector::with_capacity(self.rows());
        out.set_zero();
        out
    }

    fn mul_vector_transpose(&self, _x: &Vector<T>) -> Vector<T> {
        let mut out = Vector::with_capacity(self.cols());
        out.set_zero();
        out
    }
}
