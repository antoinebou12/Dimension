//! Training loop for neural IK using Burn.

use burn::data::dataloader::DataLoaderBuilder;
use burn::optim::{AdamConfig, GradientsParams, Optimizer};
use burn::tensor::backend::AutodiffBackend;

use crate::dataset::{IkBatcher, IkDataset};
use crate::model::{NeuralIkConfig, NeuralIkModel};

/// Runs training for the neural IK model.
///
/// Uses MSE loss between predicted and target joint angles. Requires `train` feature.
#[allow(clippy::module_name_repetitions)]
pub fn train_neural_ik<B: AutodiffBackend>(
    dataset: IkDataset,
    model_config: NeuralIkConfig,
    num_epochs: usize,
    batch_size: usize,
    learning_rate: f64,
    device: B::Device,
) -> NeuralIkModel<B> {
    let mut model = model_config.init::<B>(&device);
    let mut optim = AdamConfig::new().init();

    let batcher = IkBatcher;
    let dataloader = DataLoaderBuilder::new(batcher)
        .batch_size(batch_size)
        .shuffle(42)
        .num_workers(0)
        .build(dataset);

    for epoch in 1..=num_epochs {
        let mut num_batches = 0_usize;
        for batch in dataloader.iter() {
            let (inputs, targets) = batch.to_tensors::<B>(&device);
            let output = model.forward(inputs);
            let diff = output - targets;
            let loss = diff.powf_scalar(2.0).mean();
            let loss_val = loss.clone();
            let grads = loss.backward();
            let grads = GradientsParams::from_grads(grads, &model);
            model = optim.step(learning_rate.into(), model, grads);
            if num_batches == 0 {
                println!(
                    "Epoch {epoch}/{num_epochs} loss = {:?}",
                    loss_val.into_scalar()
                );
            }
            num_batches += 1;
        }
    }
    model
}
