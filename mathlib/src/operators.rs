use crate::cube::Cube;
use crate::matrix::Matrix;
use crate::types::Storage;
use crate::vector::Vector;
use std::ops::{Add, Mul, Sub};

impl<T: Copy + Default + Add<Output = T>> Add<&Matrix<T>> for &Matrix<T> {
    type Output = Matrix<T>;

    fn add(self, other: &Matrix<T>) -> Matrix<T> {
        assert!(self.rows() == other.rows() && self.cols() == other.cols());
        let mut out = Matrix::with_storage(self.rows(), self.cols(), self.storage);
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                out.set(i, j, self.get(i, j) + other.get(i, j));
            }
        }
        out
    }
}

impl<T: Copy + Default + Sub<Output = T>> Sub<&Matrix<T>> for &Matrix<T> {
    type Output = Matrix<T>;

    fn sub(self, other: &Matrix<T>) -> Matrix<T> {
        assert!(self.rows() == other.rows() && self.cols() == other.cols());
        let mut out = Matrix::with_storage(self.rows(), self.cols(), self.storage);
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                out.set(i, j, self.get(i, j) - other.get(i, j));
            }
        }
        out
    }
}

impl<T: Copy + Default + From<u8> + std::ops::AddAssign + Mul<Output = T>> Mul<&Matrix<T>>
    for &Matrix<T>
{
    type Output = Matrix<T>;

    fn mul(self, other: &Matrix<T>) -> Matrix<T> {
        assert!(self.cols() == other.rows());
        let mut out = Matrix::with_storage(self.rows(), other.cols(), Storage::Column);
        out.set_zero();
        for i in 0..self.rows() {
            for j in 0..other.cols() {
                let mut sum = T::from(0);
                for k in 0..self.cols() {
                    sum += self.get(i, k) * other.get(k, j);
                }
                out.set(i, j, sum);
            }
        }
        out
    }
}

// --- scalar * Matrix (concrete types to satisfy orphan rule) ---
impl Mul<&Matrix<f64>> for f64 {
    type Output = Matrix<f64>;

    fn mul(self, rhs: &Matrix<f64>) -> Matrix<f64> {
        let mut out = Matrix::with_storage(rhs.rows(), rhs.cols(), rhs.storage);
        for i in 0..rhs.rows() {
            for j in 0..rhs.cols() {
                out.set(i, j, self * rhs.get(i, j));
            }
        }
        out
    }
}

impl Mul<&Matrix<f32>> for f32 {
    type Output = Matrix<f32>;

    fn mul(self, rhs: &Matrix<f32>) -> Matrix<f32> {
        let mut out = Matrix::with_storage(rhs.rows(), rhs.cols(), rhs.storage);
        for i in 0..rhs.rows() {
            for j in 0..rhs.cols() {
                out.set(i, j, self * rhs.get(i, j));
            }
        }
        out
    }
}

// --- Matrix * Vector ---
impl<
    T: Copy + Default + From<u8> + std::ops::Add<Output = T> + std::ops::AddAssign + Mul<Output = T>,
> Mul<&Vector<T>> for &Matrix<T>
{
    type Output = Vector<T>;

    fn mul(self, v: &Vector<T>) -> Vector<T> {
        assert!(self.cols() == v.rows());
        let mut out = Vector::with_capacity(self.rows());
        out.set_zero();
        for j in 0..self.cols() {
            for i in 0..self.rows() {
                out.set(i, out.get(i) + self.get(i, j) * v.get(j));
            }
        }
        out
    }
}

// --- scalar * Vector (concrete types), f64 uses cpu backends ---
impl Mul<&Vector<f64>> for f64 {
    type Output = Vector<f64>;

    fn mul(self, rhs: &Vector<f64>) -> Vector<f64> {
        let mut out = Vector::with_capacity(rhs.rows());
        #[cfg(feature = "simd")]
        crate::cpu::simd::scalar_mul_f64(self, rhs.data(), out.data_mut());
        #[cfg(all(
            feature = "parallel",
            not(target_arch = "wasm32"),
            not(feature = "simd")
        ))]
        crate::cpu::parallel::par_scalar_mul_f64(self, rhs.data(), out.data_mut());
        #[cfg(not(any(
            feature = "simd",
            all(feature = "parallel", not(target_arch = "wasm32"))
        )))]
        crate::cpu::sequential::scalar_mul_f64(self, rhs.data(), out.data_mut());
        out
    }
}

impl Mul<&Vector<f32>> for f32 {
    type Output = Vector<f32>;

    fn mul(self, rhs: &Vector<f32>) -> Vector<f32> {
        let mut out = Vector::with_capacity(rhs.rows());
        for i in 0..rhs.rows() {
            out.set(i, self * rhs.get(i));
        }
        out
    }
}

impl<T: Copy + Default + Add<Output = T>> Add<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;

    fn add(self, other: &Vector<T>) -> Vector<T> {
        assert!(self.rows() == other.rows());
        let mut out = Vector::with_capacity(self.rows());
        for i in 0..self.rows() {
            out.set(i, self.get(i) + other.get(i));
        }
        out
    }
}

impl<T: Copy + Default + Sub<Output = T>> Sub<&Vector<T>> for &Vector<T> {
    type Output = Vector<T>;

    fn sub(self, other: &Vector<T>) -> Vector<T> {
        assert!(self.rows() == other.rows());
        let mut out = Vector::with_capacity(self.rows());
        for i in 0..self.rows() {
            out.set(i, self.get(i) - other.get(i));
        }
        out
    }
}

// --- Cube element-wise Add / Sub ---
impl<T: Copy + Default + Add<Output = T>> Add<&Cube<T>> for &Cube<T> {
    type Output = Cube<T>;

    fn add(self, other: &Cube<T>) -> Cube<T> {
        assert!(
            self.rows() == other.rows()
                && self.cols() == other.cols()
                && self.slices() == other.slices()
        );
        let mut out = Cube::with_dimensions(self.rows(), self.cols(), self.slices());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                for k in 0..self.slices() {
                    out.set(i, j, k, self.get(i, j, k) + other.get(i, j, k));
                }
            }
        }
        out
    }
}

impl<T: Copy + Default + Sub<Output = T>> Sub<&Cube<T>> for &Cube<T> {
    type Output = Cube<T>;

    fn sub(self, other: &Cube<T>) -> Cube<T> {
        assert!(
            self.rows() == other.rows()
                && self.cols() == other.cols()
                && self.slices() == other.slices()
        );
        let mut out = Cube::with_dimensions(self.rows(), self.cols(), self.slices());
        for i in 0..self.rows() {
            for j in 0..self.cols() {
                for k in 0..self.slices() {
                    out.set(i, j, k, self.get(i, j, k) - other.get(i, j, k));
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_vector(data: &[f64]) -> Vector<f64> {
        let mut v = Vector::with_capacity(data.len());
        for (i, &val) in data.iter().enumerate() {
            v.set(i, val);
        }
        v
    }

    fn make_matrix(rows: usize, cols: usize, data: &[f64]) -> Matrix<f64> {
        let mut m = Matrix::with_dimensions(rows, cols);
        for i in 0..rows {
            for j in 0..cols {
                m.set(i, j, data[i * cols + j]);
            }
        }
        m
    }

    #[test]
    fn test_matrix_add() {
        let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let b = make_matrix(2, 2, &[5.0, 6.0, 7.0, 8.0]);
        let c = &a + &b;
        assert!((c.get(0, 0) - 6.0).abs() < 1e-10);
        assert!((c.get(0, 1) - 8.0).abs() < 1e-10);
        assert!((c.get(1, 0) - 10.0).abs() < 1e-10);
        assert!((c.get(1, 1) - 12.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_sub() {
        let a = make_matrix(2, 2, &[5.0, 6.0, 7.0, 8.0]);
        let b = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let c = &a - &b;
        assert!((c.get(0, 0) - 4.0).abs() < 1e-10);
        assert!((c.get(1, 1) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_mul() {
        // [1 2] * [5 6] = [1*5+2*7  1*6+2*8] = [19 22]
        // [3 4]   [7 8]   [3*5+4*7  3*6+4*8]   [43 50]
        let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let b = make_matrix(2, 2, &[5.0, 6.0, 7.0, 8.0]);
        let c = &a * &b;
        assert!((c.get(0, 0) - 19.0).abs() < 1e-10);
        assert!((c.get(0, 1) - 22.0).abs() < 1e-10);
        assert!((c.get(1, 0) - 43.0).abs() < 1e-10);
        assert!((c.get(1, 1) - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_mul_identity() {
        let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let mut i = Matrix::with_dimensions(2, 2);
        i.set_identity();
        let c = &a * &i;
        assert!((c.get(0, 0) - 1.0).abs() < 1e-10);
        assert!((c.get(0, 1) - 2.0).abs() < 1e-10);
        assert!((c.get(1, 0) - 3.0).abs() < 1e-10);
        assert!((c.get(1, 1) - 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_mul_rectangular() {
        // [1 2 3] * [7 ]   [1*7+2*8+3*9]   [50]
        // [4 5 6]   [8 ] = [4*7+5*8+6*9] = [122]
        //           [9 ]
        let a = make_matrix(2, 3, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
        let b = make_matrix(3, 1, &[7.0, 8.0, 9.0]);
        let c = &a * &b;
        assert_eq!(c.rows(), 2);
        assert_eq!(c.cols(), 1);
        assert!((c.get(0, 0) - 50.0).abs() < 1e-10);
        assert!((c.get(1, 0) - 122.0).abs() < 1e-10);
    }

    #[test]
    fn test_scalar_mul_matrix_f64() {
        let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let c = 2.0_f64 * &a;
        assert!((c.get(0, 0) - 2.0).abs() < 1e-10);
        assert!((c.get(1, 1) - 8.0).abs() < 1e-10);
    }

    #[test]
    fn test_scalar_mul_matrix_f32() {
        let mut a: Matrix<f32> = Matrix::with_dimensions(2, 2);
        a.set(0, 0, 1.0);
        a.set(1, 1, 4.0);
        let c = 3.0_f32 * &a;
        assert!((c.get(0, 0) - 3.0).abs() < 1e-6);
        assert!((c.get(1, 1) - 12.0).abs() < 1e-6);
    }

    #[test]
    fn test_matrix_vector_mul() {
        // [1 2] * [3] = [1*3+2*4] = [11]
        // [3 4]   [4]   [3*3+4*4]   [25]
        let a = make_matrix(2, 2, &[1.0, 2.0, 3.0, 4.0]);
        let v = make_vector(&[3.0, 4.0]);
        let c = &a * &v;
        assert_eq!(c.rows(), 2);
        assert!((c.get(0) - 11.0).abs() < 1e-10);
        assert!((c.get(1) - 25.0).abs() < 1e-10);
    }

    #[test]
    fn test_scalar_mul_vector_f64() {
        let v = make_vector(&[1.0, 2.0, 3.0]);
        let c = 2.0_f64 * &v;
        assert!((c.get(0) - 2.0).abs() < 1e-10);
        assert!((c.get(1) - 4.0).abs() < 1e-10);
        assert!((c.get(2) - 6.0).abs() < 1e-10);
    }

    #[test]
    fn test_scalar_mul_vector_f32() {
        let mut v: Vector<f32> = Vector::with_capacity(2);
        v.set(0, 1.0);
        v.set(1, 2.0);
        let c = 5.0_f32 * &v;
        assert!((c.get(0) - 5.0).abs() < 1e-6);
        assert!((c.get(1) - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_vector_add() {
        let a = make_vector(&[1.0, 2.0, 3.0]);
        let b = make_vector(&[4.0, 5.0, 6.0]);
        let c = &a + &b;
        assert!((c.get(0) - 5.0).abs() < 1e-10);
        assert!((c.get(1) - 7.0).abs() < 1e-10);
        assert!((c.get(2) - 9.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector_sub() {
        let a = make_vector(&[4.0, 5.0, 6.0]);
        let b = make_vector(&[1.0, 2.0, 3.0]);
        let c = &a - &b;
        assert!((c.get(0) - 3.0).abs() < 1e-10);
        assert!((c.get(1) - 3.0).abs() < 1e-10);
        assert!((c.get(2) - 3.0).abs() < 1e-10);
    }

    fn make_cube(rows: usize, cols: usize, slices: usize, data: &[f64]) -> Cube<f64> {
        let mut c = Cube::with_dimensions(rows, cols, slices);
        for (idx, &val) in data.iter().enumerate() {
            let n = rows * cols;
            let k = idx / n;
            let r = idx % n;
            let j = r / rows;
            let i = r % rows;
            c.set(i, j, k, val);
        }
        c
    }

    #[test]
    fn test_cube_add() {
        let a = make_cube(2, 2, 2, &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
        let b = make_cube(2, 2, 2, &[10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0]);
        let c = &a + &b;
        assert!((c.get(0, 0, 0) - 11.0).abs() < 1e-10);
        assert!((c.get(1, 1, 1) - 88.0).abs() < 1e-10);
    }

    #[test]
    fn test_cube_sub() {
        let a = make_cube(2, 2, 1, &[5.0, 6.0, 7.0, 8.0]);
        let b = make_cube(2, 2, 1, &[1.0, 2.0, 3.0, 4.0]);
        let c = &a - &b;
        assert!((c.get(0, 0, 0) - 4.0).abs() < 1e-10);
        assert!((c.get(1, 1, 0) - 4.0).abs() < 1e-10);
    }
}
