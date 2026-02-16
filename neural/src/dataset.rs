//! IK dataset: generate (target position, joint angles) pairs using kinematics (Halley IK).

use burn::data::dataset::Dataset;
use burn::tensor::backend::Backend;
use burn::tensor::{Tensor, TensorData};

use mathlib::cg::{new_translation, vector3};

use crate::chain_config::ChainConfig;
use crate::utils::{normalize_joints, normalize_position};

#[cfg(not(target_arch = "wasm32"))]
use kinematics::{Armature, HalleyIk};
#[cfg(not(target_arch = "wasm32"))]
use rand::Rng;

/// A single IK sample: normalized input (target xyz [+ current state]) and normalized joint angles.
#[derive(Clone, Debug)]
pub struct IkItem {
    /// Input: [target_x, target_y, target_z] normalized, optionally followed by current joint state.
    pub input: Vec<f32>,
    /// Target joint angles (normalized) for the chain.
    pub target: Vec<f32>,
}

/// In-memory dataset of IK samples.
#[derive(Clone)]
pub struct IkDataset {
    items: Vec<IkItem>,
    chain: ChainConfig,
}

impl IkDataset {
    /// Creates a dataset from precomputed items and chain config.
    pub fn new(items: Vec<IkItem>, chain: ChainConfig) -> Self {
        Self { items, chain }
    }

    /// Generates samples using the kinematics crate: random targets in workspace, solve with Halley IK.
    ///
    /// Runs on the CPU (not WASM) and requires the `train` feature and `rand`.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn generate_with_halley(
        armature: &mut Armature,
        end_effector_idx: usize,
        chain: ChainConfig,
        num_samples: usize,
        seed: u64,
    ) -> Self {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
        let dof = armature.pack().len();
        assert_eq!(dof, chain.dof, "armature DOF must match chain.dof");

        let [mx, my, mz] = chain.workspace_min;
        let [max_x, max_y, max_z] = chain.workspace_max;
        let mut items = Vec::with_capacity(num_samples);
        let mut num_attempts = 0_usize;
        let max_attempts = num_samples * 4;

        while items.len() < num_samples && num_attempts < max_attempts {
            num_attempts += 1;
            let x = rng.gen_range(mx..=max_x);
            let y = rng.gen_range(my..=max_y);
            let z = rng.gen_range(mz..=max_z);
            let target_pos = [x, y, z];
            let v = vector3(x, y, z);
            let target_tf = new_translation(&v);

            let mut solver = HalleyIk::new(armature, end_effector_idx, target_tf)
                .with_max_iters(64)
                .with_tolerance(1e-3);
            let err = solver.solve();
            if err > 0.01 {
                continue;
            }

            let theta = armature.pack();
            let mut input = normalize_position(target_pos, &chain).to_vec();
            if chain.use_current_state {
                let mut current_norm = vec![0.0; dof];
                normalize_joints(&theta, &chain, &mut current_norm);
                input.extend(current_norm);
            }
            let mut target_norm = vec![0.0; dof];
            normalize_joints(&theta, &chain, &mut target_norm);

            items.push(IkItem {
                input,
                target: target_norm,
            });
        }

        Self { items, chain }
    }

    /// Chain config used by this dataset.
    #[must_use]
    pub fn chain(&self) -> &ChainConfig {
        &self.chain
    }
}

impl Dataset<IkItem> for IkDataset {
    fn get(&self, index: usize) -> Option<IkItem> {
        self.items.get(index).cloned()
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

/// Batch of IK samples (raw data; convert to tensors with [IkBatch::to_tensors]).
#[derive(Clone, Debug)]
pub struct IkBatch {
    /// Flattened input [batch_size * input_size].
    pub input_data: Vec<f32>,
    /// Flattened target [batch_size * output_size].
    pub target_data: Vec<f32>,
    /// Number of samples in the batch.
    pub batch_size: usize,
    /// Input feature dimension per sample.
    pub input_size: usize,
    /// Output (joint angle) dimension per sample.
    pub output_size: usize,
}

impl IkBatch {
    /// Convert to tensors on the given backend and device.
    pub fn to_tensors<B: Backend>(&self, device: &B::Device) -> (Tensor<B, 2>, Tensor<B, 2>) {
        if self.batch_size == 0 {
            let inputs = Tensor::zeros([0, 1], device);
            let targets = Tensor::zeros([0, 1], device);
            return (inputs, targets);
        }
        let inputs = Tensor::<B, 1>::from_data(
            TensorData::from(self.input_data.as_slice()).convert::<B::FloatElem>(),
            device,
        )
        .reshape([self.batch_size, self.input_size]);
        let targets = Tensor::<B, 1>::from_data(
            TensorData::from(self.target_data.as_slice()).convert::<B::FloatElem>(),
            device,
        )
        .reshape([self.batch_size, self.output_size]);
        (inputs, targets)
    }
}

/// Batcher that converts IK items to raw batch (no device needed).
#[derive(Clone, Default)]
pub struct IkBatcher;

impl burn::data::dataloader::batcher::Batcher<IkItem, IkBatch> for IkBatcher {
    fn batch(&self, items: Vec<IkItem>) -> IkBatch {
        if items.is_empty() {
            return IkBatch {
                input_data: vec![],
                target_data: vec![],
                batch_size: 0,
                input_size: 1,
                output_size: 1,
            };
        }
        let input_size = items[0].input.len();
        let output_size = items[0].target.len();
        let batch_size = items.len();
        let mut input_data = Vec::with_capacity(batch_size * input_size);
        let mut target_data = Vec::with_capacity(batch_size * output_size);
        for item in &items {
            input_data.extend_from_slice(&item.input);
            target_data.extend_from_slice(&item.target);
        }
        IkBatch {
            input_data,
            target_data,
            batch_size,
            input_size,
            output_size,
        }
    }
}
