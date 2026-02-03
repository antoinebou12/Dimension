//! `Hash`, `PartialEq`, and `Eq` implementations for Matrix and Vector.
//!
//! For types that implement `Hash` and `Eq` (e.g. `i32`, `u32`), generic
//! implementations are provided via the [`HashableElement`] marker trait
//! (floats are excluded so we can use `to_bits()` for f32/f64).
//!
//! For float types (f32, f64), elements are hashed via `to_bits()` since floats
//! do not implement `Hash` in Rust. NaN bit patterns may have inconsistent
//! behavior across platforms but are deterministic for non-NaN values.

use crate::matrix::Matrix;
use crate::vector::Vector;
use std::hash::{Hash, Hasher};

/// Marker for element types that implement `Hash` + `Eq`. Used to provide
/// generic `Hash`/`PartialEq`/`Eq` for `Matrix<T>` and `Vector<T>` without
/// overlapping with the float-specific impls (f32/f64 use `to_bits()`).
pub trait HashableElement: Hash + PartialEq + Eq {}

impl HashableElement for i8 {}
impl HashableElement for i16 {}
impl HashableElement for i32 {}
impl HashableElement for i64 {}
impl HashableElement for u8 {}
impl HashableElement for u16 {}
impl HashableElement for u32 {}
impl HashableElement for u64 {}
impl HashableElement for bool {}

// --- Generic Matrix<T> and Vector<T> where T: HashableElement ---

impl<T: HashableElement> Hash for Matrix<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rows().hash(state);
        self.cols().hash(state);
        (self.storage as u8).hash(state);
        for x in self.data() {
            x.hash(state);
        }
    }
}

impl<T: HashableElement> PartialEq for Matrix<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows() == other.rows()
            && self.cols() == other.cols()
            && self.storage == other.storage
            && self.data() == other.data()
    }
}

impl<T: HashableElement> Eq for Matrix<T> {}

impl<T: HashableElement> Hash for Vector<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rows().hash(state);
        for x in self.data() {
            x.hash(state);
        }
    }
}

impl<T: HashableElement> PartialEq for Vector<T> {
    fn eq(&self, other: &Self) -> bool {
        self.rows() == other.rows() && self.data() == other.data()
    }
}

impl<T: HashableElement> Eq for Vector<T> {}

// --- Matrix<f64> and Matrix<f32>: use to_bits for Hash/Eq ---

impl Hash for Matrix<f64> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rows().hash(state);
        self.cols().hash(state);
        (self.storage as u8).hash(state);
        let (r, c) = (self.rows(), self.cols());
        for i in 0..r {
            for j in 0..c {
                self.get(i, j).to_bits().hash(state);
            }
        }
    }
}

impl PartialEq for Matrix<f64> {
    fn eq(&self, other: &Self) -> bool {
        if self.rows() != other.rows()
            || self.cols() != other.cols()
            || self.storage != other.storage
        {
            return false;
        }
        let (r, c) = (self.rows(), self.cols());
        for i in 0..r {
            for j in 0..c {
                if self.get(i, j).to_bits() != other.get(i, j).to_bits() {
                    return false;
                }
            }
        }
        true
    }
}

impl Eq for Matrix<f64> {}

impl Hash for Matrix<f32> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rows().hash(state);
        self.cols().hash(state);
        (self.storage as u8).hash(state);
        let (r, c) = (self.rows(), self.cols());
        for i in 0..r {
            for j in 0..c {
                self.get(i, j).to_bits().hash(state);
            }
        }
    }
}

impl PartialEq for Matrix<f32> {
    fn eq(&self, other: &Self) -> bool {
        if self.rows() != other.rows()
            || self.cols() != other.cols()
            || self.storage != other.storage
        {
            return false;
        }
        let (r, c) = (self.rows(), self.cols());
        for i in 0..r {
            for j in 0..c {
                if self.get(i, j).to_bits() != other.get(i, j).to_bits() {
                    return false;
                }
            }
        }
        true
    }
}

impl Eq for Matrix<f32> {}

// --- Vector<f64> and Vector<f32>: use to_bits for Hash/Eq ---

impl Hash for Vector<f64> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rows().hash(state);
        for i in 0..self.rows() {
            self.get(i).to_bits().hash(state);
        }
    }
}

impl PartialEq for Vector<f64> {
    fn eq(&self, other: &Self) -> bool {
        if self.rows() != other.rows() {
            return false;
        }
        for i in 0..self.rows() {
            if self.get(i).to_bits() != other.get(i).to_bits() {
                return false;
            }
        }
        true
    }
}

impl Eq for Vector<f64> {}

impl Hash for Vector<f32> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.rows().hash(state);
        for i in 0..self.rows() {
            self.get(i).to_bits().hash(state);
        }
    }
}

impl PartialEq for Vector<f32> {
    fn eq(&self, other: &Self) -> bool {
        if self.rows() != other.rows() {
            return false;
        }
        for i in 0..self.rows() {
            if self.get(i).to_bits() != other.get(i).to_bits() {
                return false;
            }
        }
        true
    }
}

impl Eq for Vector<f32> {}
