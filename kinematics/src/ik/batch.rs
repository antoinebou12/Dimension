//! Batch IK solve: run multiple armature–target pairs in parallel when the `parallel` feature
//! is enabled (native only). Falls back to sequential iteration on wasm or without the feature.

use mathlib::{Matrix4f, Vector3f};

use crate::armature::Armature;
use crate::ik::{HalleyIk, HessianIk, JacobianIk};

/// Solves Jacobian IK for each (armature, end_effector_idx, target) pair.
/// Returns each armature with updated joint state and the final error.
#[allow(clippy::module_name_repetitions)]
pub fn solve_batch_jacobian(batch: Vec<(Armature, usize, Vector3f)>) -> Vec<(Armature, f32)> {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use par_iter::prelude::*;
        batch
            .into_par_iter()
            .map(|(mut arm, ee_idx, target)| {
                let err = JacobianIk::new(&mut arm, ee_idx, target)
                    .with_max_iters(30)
                    .solve();
                (arm, err)
            })
            .collect()
    }

    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        batch
            .into_iter()
            .map(|(mut arm, ee_idx, target)| {
                let err = JacobianIk::new(&mut arm, ee_idx, target)
                    .with_max_iters(30)
                    .solve();
                (arm, err)
            })
            .collect()
    }
}

/// Solves Hessian IK for each (armature, end_effector_idx, target) pair.
/// Returns each armature with updated joint state and the final error.
#[allow(clippy::module_name_repetitions)]
pub fn solve_batch_hessian(batch: Vec<(Armature, usize, Vector3f)>) -> Vec<(Armature, f32)> {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use par_iter::prelude::*;
        batch
            .into_par_iter()
            .map(|(mut arm, ee_idx, target)| {
                let err = HessianIk::new(&mut arm, ee_idx, target)
                    .with_max_iters(32)
                    .solve();
                (arm, err)
            })
            .collect()
    }

    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        batch
            .into_iter()
            .map(|(mut arm, ee_idx, target)| {
                let err = HessianIk::new(&mut arm, ee_idx, target)
                    .with_max_iters(32)
                    .solve();
                (arm, err)
            })
            .collect()
    }
}

/// Solves Halley IK (6D pose) for each (armature, end_effector_idx, target) pair.
/// Returns each armature with updated joint state and the final error.
#[allow(clippy::module_name_repetitions)]
pub fn solve_batch_halley(batch: Vec<(Armature, usize, Matrix4f)>) -> Vec<(Armature, f32)> {
    #[cfg(all(feature = "parallel", not(target_arch = "wasm32")))]
    {
        use par_iter::prelude::*;
        batch
            .into_par_iter()
            .map(|(mut arm, ee_idx, target)| {
                let err = HalleyIk::new(&mut arm, ee_idx, target)
                    .with_max_iters(32)
                    .solve();
                (arm, err)
            })
            .collect()
    }

    #[cfg(not(all(feature = "parallel", not(target_arch = "wasm32"))))]
    {
        batch
            .into_iter()
            .map(|(mut arm, ee_idx, target)| {
                let err = HalleyIk::new(&mut arm, ee_idx, target)
                    .with_max_iters(32)
                    .solve();
                (arm, err)
            })
            .collect()
    }
}
