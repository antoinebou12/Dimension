//! Executor abstraction for basic linear algebra: CPU-only, GPU-only, or automatic (threshold-based).
//!
//! The GPU path assumes contiguous column-major storage. Thresholds can be tuned from
//! benchmarks; see `cargo bench --bench gpu` and threshold guidance in the bench output.
//!
//! `CpuExecutor` uses the same CPU backends as the rest of the crate: when the `simd` or
//! `parallel` feature is enabled, `dot`, `add_vector`, `scale_vector`, `matvec`, and `axpy` benefit
//! from SIMD or parallel execution.

use crate::Storage;
use crate::matrix::Matrix;
use crate::vector::Vector;

/// Configurable thresholds for when to use GPU in [`AutoExecutor`].
///
/// Tune these from benchmarks (e.g. `cargo bench --bench gpu`). GPU is used only when
/// the operation size is at or above the corresponding threshold and the GPU is available.
#[derive(Clone, Debug)]
pub struct ExecutorThresholds {
    /// Minimum number of elements (M*K*N) for matmul to use GPU. Default 128^3 ≈ 2M.
    pub matmul_elements_min: usize,
    /// Minimum vector length for dot/norm to use GPU. Default `1_000_000`.
    pub dot_len_min: usize,
    /// Minimum rows*cols for matvec to use GPU. Default 256*256.
    pub matvec_elements_min: usize,
    /// Minimum length for elementwise (add, scale, axpy) to use GPU. Default `1_000_000`.
    pub elementwise_len_min: usize,
}

impl Default for ExecutorThresholds {
    fn default() -> Self {
        Self {
            matmul_elements_min: 128 * 128 * 128,
            dot_len_min: 1_000_000,
            matvec_elements_min: 256 * 256,
            elementwise_len_min: 1_000_000,
        }
    }
}

/// Executor for f32 linear algebra: always returns a result (CPU fallback when GPU fails).
pub trait Executor {
    /// Matrix multiply C = A * B.
    fn matmul(&self, a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32>;
    /// Matrix-vector product y = A * x.
    fn matvec(&self, a: &Matrix<f32>, x: &Vector<f32>) -> Vector<f32>;
    /// Dot product x · y.
    fn dot(&self, x: &Vector<f32>, y: &Vector<f32>) -> f32;
    /// Element-wise matrix add C = A + B.
    fn add_matrix(&self, a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32>;
    /// Element-wise vector add z = x + y.
    fn add_vector(&self, x: &Vector<f32>, y: &Vector<f32>) -> Vector<f32>;
    /// Scale matrix B = s * A.
    fn scale_matrix(&self, s: f32, a: &Matrix<f32>) -> Matrix<f32>;
    /// Scale vector z = s * x.
    fn scale_vector(&self, s: f32, x: &Vector<f32>) -> Vector<f32>;
    /// AXPY: z = alpha * x + y (writes into a new vector).
    fn axpy(&self, alpha: f32, x: &Vector<f32>, y: &Vector<f32>) -> Vector<f32>;
}

/// CPU-only executor. Always uses CPU; no GPU dependency.
#[derive(Clone, Copy, Debug, Default)]
pub struct CpuExecutor;

impl CpuExecutor {
    /// Creates a new CPU-only executor.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Executor for CpuExecutor {
    fn matmul(&self, a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
        assert_eq!(a.cols(), b.rows());
        let mut out = Matrix::with_storage(a.rows(), b.cols(), Storage::Column);
        out.set_zero();
        for i in 0..a.rows() {
            for j in 0..b.cols() {
                let mut sum = 0.0f32;
                for k in 0..a.cols() {
                    sum += a.get(i, k) * b.get(k, j);
                }
                out.set(i, j, sum);
            }
        }
        out
    }

    fn matvec(&self, a: &Matrix<f32>, x: &Vector<f32>) -> Vector<f32> {
        assert_eq!(a.cols(), x.rows());
        let m = a.rows();
        let n = a.cols();
        let mut out = Vector::with_capacity(m);
        if a.storage == Storage::Column {
            out.set_zero();
            crate::cpu::matvec_col_major_f32(m, n, a.data(), x.as_slice(), out.data_mut());
        } else {
            out.set_zero();
            for j in 0..n {
                for i in 0..m {
                    out.set(i, out.get(i) + a.get(i, j) * x.get(j));
                }
            }
        }
        out
    }

    fn dot(&self, x: &Vector<f32>, y: &Vector<f32>) -> f32 {
        assert_eq!(x.rows(), y.rows());
        crate::cpu::dot_f32(x.as_slice(), y.as_slice())
    }

    fn add_matrix(&self, a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
        assert_eq!(a.rows(), b.rows());
        assert_eq!(a.cols(), b.cols());
        let mut out = Matrix::with_storage(a.rows(), a.cols(), a.storage);
        for i in 0..a.rows() {
            for j in 0..a.cols() {
                out.set(i, j, a.get(i, j) + b.get(i, j));
            }
        }
        out
    }

    fn add_vector(&self, x: &Vector<f32>, y: &Vector<f32>) -> Vector<f32> {
        assert_eq!(x.rows(), y.rows());
        let mut out = Vector::with_capacity(x.rows());
        crate::cpu::add_f32(x.as_slice(), y.as_slice(), out.data_mut());
        out
    }

    fn scale_matrix(&self, s: f32, a: &Matrix<f32>) -> Matrix<f32> {
        let mut out = Matrix::with_storage(a.rows(), a.cols(), a.storage);
        for i in 0..a.rows() {
            for j in 0..a.cols() {
                out.set(i, j, s * a.get(i, j));
            }
        }
        out
    }

    fn scale_vector(&self, s: f32, x: &Vector<f32>) -> Vector<f32> {
        let mut out = Vector::with_capacity(x.rows());
        crate::cpu::scalar_mul_f32(s, x.as_slice(), out.data_mut());
        out
    }

    fn axpy(&self, alpha: f32, x: &Vector<f32>, y: &Vector<f32>) -> Vector<f32> {
        assert_eq!(x.rows(), y.rows());
        let mut scaled = Vector::with_capacity(x.rows());
        crate::cpu::scalar_mul_f32(alpha, x.as_slice(), scaled.data_mut());
        let mut out = Vector::with_capacity(x.rows());
        crate::cpu::add_f32(scaled.as_slice(), y.as_slice(), out.data_mut());
        out
    }
}

#[cfg(feature = "gpu")]
mod gpu_exec {
    use super::{CpuExecutor, Executor, ExecutorThresholds};
    use crate::gpu;
    use crate::matrix::Matrix;
    use crate::vector::Vector;
    use std::sync::atomic::{AtomicBool, Ordering};

    /// GPU-only executor: tries GPU, on failure falls back to CPU (with one-time log).
    #[derive(Clone, Copy, Debug, Default)]
    pub struct GpuExecutor;

    impl GpuExecutor {
        #[must_use]
        pub const fn new() -> Self {
            Self
        }
    }

    impl Executor for GpuExecutor {
        fn matmul(&self, a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
            if let Some(out) = gpu::try_matmul_f32(a, b) {
                return out;
            }
            static WARNED: AtomicBool = AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!("mathlib executor: GPU matmul failed, falling back to CPU");
            }
            CpuExecutor.matmul(a, b)
        }

        fn matvec(&self, a: &Matrix<f32>, x: &Vector<f32>) -> Vector<f32> {
            if let Some(out) = gpu::try_matvec_f32(a, x) {
                return out;
            }
            static WARNED: AtomicBool = AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!("mathlib executor: GPU matvec failed, falling back to CPU");
            }
            CpuExecutor.matvec(a, x)
        }

        fn dot(&self, x: &Vector<f32>, y: &Vector<f32>) -> f32 {
            if let Some(v) = gpu::try_dot_f32(x, y) {
                return v;
            }
            static WARNED: AtomicBool = AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!("mathlib executor: GPU dot failed, falling back to CPU");
            }
            CpuExecutor.dot(x, y)
        }

        fn add_matrix(&self, a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
            if let Some(data) = gpu::try_add_f32(a.data(), b.data()) {
                let mut out = Matrix::with_storage(a.rows(), a.cols(), a.storage);
                out.data_mut().copy_from_slice(&data);
                return out;
            }
            static WARNED: AtomicBool = AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!("mathlib executor: GPU add_matrix failed, falling back to CPU");
            }
            CpuExecutor.add_matrix(a, b)
        }

        fn add_vector(&self, x: &Vector<f32>, y: &Vector<f32>) -> Vector<f32> {
            if let Some(data) = gpu::try_add_f32(x.data(), y.data()) {
                let mut out = Vector::with_capacity(x.rows());
                out.data_mut().copy_from_slice(&data);
                return out;
            }
            static WARNED: AtomicBool = AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!("mathlib executor: GPU add_vector failed, falling back to CPU");
            }
            CpuExecutor.add_vector(x, y)
        }

        fn scale_matrix(&self, s: f32, a: &Matrix<f32>) -> Matrix<f32> {
            if let Some(data) = gpu::try_scale_f32(s, a.data()) {
                let mut out = Matrix::with_storage(a.rows(), a.cols(), a.storage);
                out.data_mut().copy_from_slice(&data);
                return out;
            }
            static WARNED: AtomicBool = AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!("mathlib executor: GPU scale_matrix failed, falling back to CPU");
            }
            CpuExecutor.scale_matrix(s, a)
        }

        fn scale_vector(&self, s: f32, x: &Vector<f32>) -> Vector<f32> {
            if let Some(data) = gpu::try_scale_f32(s, x.data()) {
                let mut out = Vector::with_capacity(x.rows());
                out.data_mut().copy_from_slice(&data);
                return out;
            }
            static WARNED: AtomicBool = AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!("mathlib executor: GPU scale_vector failed, falling back to CPU");
            }
            CpuExecutor.scale_vector(s, x)
        }

        fn axpy(&self, alpha: f32, x: &Vector<f32>, y: &Vector<f32>) -> Vector<f32> {
            if let Some(data) = gpu::try_axpy_f32(alpha, x.data(), y.data()) {
                let mut out = Vector::with_capacity(x.rows());
                out.data_mut().copy_from_slice(&data);
                return out;
            }
            static WARNED: AtomicBool = AtomicBool::new(false);
            if !WARNED.swap(true, Ordering::Relaxed) {
                tracing::warn!("mathlib executor: GPU axpy failed, falling back to CPU");
            }
            CpuExecutor.axpy(alpha, x, y)
        }
    }

    /// Executor that uses GPU when available and size is above threshold; otherwise CPU.
    #[derive(Clone, Debug)]
    pub struct AutoExecutor {
        pub thresholds: ExecutorThresholds,
    }

    impl Default for AutoExecutor {
        fn default() -> Self {
            Self {
                thresholds: ExecutorThresholds::default(),
            }
        }
    }

    impl AutoExecutor {
        #[must_use]
        pub fn new() -> Self {
            Self {
                thresholds: ExecutorThresholds::default(),
            }
        }

        #[must_use]
        pub fn with_thresholds(thresholds: ExecutorThresholds) -> Self {
            Self { thresholds }
        }

        fn use_gpu_matmul(&self, m: usize, k: usize, n: usize) -> bool {
            gpu::is_available() && (m * k * n) >= self.thresholds.matmul_elements_min
        }

        fn use_gpu_dot(&self, len: usize) -> bool {
            gpu::is_available() && len >= self.thresholds.dot_len_min
        }

        fn use_gpu_matvec(&self, rows: usize, cols: usize) -> bool {
            gpu::is_available() && (rows * cols) >= self.thresholds.matvec_elements_min
        }

        fn use_gpu_elementwise(&self, len: usize) -> bool {
            gpu::is_available() && len >= self.thresholds.elementwise_len_min
        }
    }

    impl Executor for AutoExecutor {
        fn matmul(&self, a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
            let m = a.rows();
            let k = a.cols();
            let n = b.cols();
            if self.use_gpu_matmul(m, k, n) {
                if let Some(out) = gpu::try_matmul_f32(a, b) {
                    return out;
                }
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    tracing::warn!("mathlib AutoExecutor: GPU matmul failed, falling back to CPU");
                }
            }
            CpuExecutor.matmul(a, b)
        }

        fn matvec(&self, a: &Matrix<f32>, x: &Vector<f32>) -> Vector<f32> {
            let rows = a.rows();
            let cols = a.cols();
            if self.use_gpu_matvec(rows, cols) {
                if let Some(out) = gpu::try_matvec_f32(a, x) {
                    return out;
                }
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    tracing::warn!("mathlib AutoExecutor: GPU matvec failed, falling back to CPU");
                }
            }
            CpuExecutor.matvec(a, x)
        }

        fn dot(&self, x: &Vector<f32>, y: &Vector<f32>) -> f32 {
            let len = x.rows();
            if self.use_gpu_dot(len) {
                if let Some(v) = gpu::try_dot_f32(x, y) {
                    return v;
                }
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    tracing::warn!("mathlib AutoExecutor: GPU dot failed, falling back to CPU");
                }
            }
            CpuExecutor.dot(x, y)
        }

        fn add_matrix(&self, a: &Matrix<f32>, b: &Matrix<f32>) -> Matrix<f32> {
            let len = a.rows() * a.cols();
            if self.use_gpu_elementwise(len) {
                if let Some(data) = gpu::try_add_f32(a.data(), b.data()) {
                    let mut out = Matrix::with_storage(a.rows(), a.cols(), a.storage);
                    out.data_mut().copy_from_slice(&data);
                    return out;
                }
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    tracing::warn!(
                        "mathlib AutoExecutor: GPU add_matrix failed, falling back to CPU"
                    );
                }
            }
            CpuExecutor.add_matrix(a, b)
        }

        fn add_vector(&self, x: &Vector<f32>, y: &Vector<f32>) -> Vector<f32> {
            let len = x.rows();
            if self.use_gpu_elementwise(len) {
                if let Some(data) = gpu::try_add_f32(x.data(), y.data()) {
                    let mut out = Vector::with_capacity(len);
                    out.data_mut().copy_from_slice(&data);
                    return out;
                }
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    tracing::warn!(
                        "mathlib AutoExecutor: GPU add_vector failed, falling back to CPU"
                    );
                }
            }
            CpuExecutor.add_vector(x, y)
        }

        fn scale_matrix(&self, s: f32, a: &Matrix<f32>) -> Matrix<f32> {
            let len = a.rows() * a.cols();
            if self.use_gpu_elementwise(len) {
                if let Some(data) = gpu::try_scale_f32(s, a.data()) {
                    let mut out = Matrix::with_storage(a.rows(), a.cols(), a.storage);
                    out.data_mut().copy_from_slice(&data);
                    return out;
                }
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    tracing::warn!(
                        "mathlib AutoExecutor: GPU scale_matrix failed, falling back to CPU"
                    );
                }
            }
            CpuExecutor.scale_matrix(s, a)
        }

        fn scale_vector(&self, s: f32, x: &Vector<f32>) -> Vector<f32> {
            let len = x.rows();
            if self.use_gpu_elementwise(len) {
                if let Some(data) = gpu::try_scale_f32(s, x.data()) {
                    let mut out = Vector::with_capacity(len);
                    out.data_mut().copy_from_slice(&data);
                    return out;
                }
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    tracing::warn!(
                        "mathlib AutoExecutor: GPU scale_vector failed, falling back to CPU"
                    );
                }
            }
            CpuExecutor.scale_vector(s, x)
        }

        fn axpy(&self, alpha: f32, x: &Vector<f32>, y: &Vector<f32>) -> Vector<f32> {
            let len = x.rows();
            if self.use_gpu_elementwise(len) {
                if let Some(data) = gpu::try_axpy_f32(alpha, x.data(), y.data()) {
                    let mut out = Vector::with_capacity(len);
                    out.data_mut().copy_from_slice(&data);
                    return out;
                }
                static WARNED: AtomicBool = AtomicBool::new(false);
                if !WARNED.swap(true, Ordering::Relaxed) {
                    tracing::warn!("mathlib AutoExecutor: GPU axpy failed, falling back to CPU");
                }
            }
            CpuExecutor.axpy(alpha, x, y)
        }
    }
}

#[cfg(feature = "gpu")]
pub use gpu_exec::{AutoExecutor, GpuExecutor};
