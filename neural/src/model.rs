//! Neural IK model: MLP mapping target position (and optionally current joints) → joint angles.

use burn::{
    nn::{Linear, LinearConfig, Relu},
    prelude::*,
};

use crate::ChainConfig;

/// Configuration for the neural IK MLP.
#[derive(Clone, Debug)]
pub struct NeuralIkConfig {
    /// Chain config (input/output sizes and normalization).
    pub chain: ChainConfig,
    /// Hidden layer size.
    pub hidden_size: usize,
    /// Number of hidden layers (each hidden_size, then output dof).
    pub num_hidden_layers: usize,
}

impl NeuralIkConfig {
    /// Creates config from chain; hidden_size and num_hidden_layers use defaults.
    pub fn new(chain: ChainConfig) -> Self {
        Self {
            chain: chain.clone(),
            hidden_size: 256,
            num_hidden_layers: 2,
        }
    }

    /// Sets hidden layer size.
    #[must_use]
    pub fn with_hidden_size(mut self, hidden_size: usize) -> Self {
        self.hidden_size = hidden_size;
        self
    }

    /// Sets number of hidden layers.
    #[must_use]
    pub fn with_num_hidden_layers(mut self, num_hidden_layers: usize) -> Self {
        self.num_hidden_layers = num_hidden_layers;
        self
    }

    /// Input dimension (3 + optional dof).
    #[must_use]
    pub fn input_size(&self) -> usize {
        self.chain.input_size()
    }

    /// Output dimension (dof).
    #[must_use]
    pub fn output_size(&self) -> usize {
        self.chain.output_size()
    }

    /// Initialize the model on the given device.
    pub fn init<B: Backend>(&self, device: &B::Device) -> NeuralIkModel<B> {
        let input_size = self.input_size();
        let output_size = self.output_size();
        let h = self.hidden_size;
        let n = self.num_hidden_layers;

        let mut layers = Vec::with_capacity(n + 1);
        // First layer: input -> hidden
        layers.push(LinearConfig::new(input_size, h).init(device));
        for _ in 1..n {
            layers.push(LinearConfig::new(h, h).init(device));
        }
        layers.push(LinearConfig::new(h, output_size).init(device));

        NeuralIkModel {
            layers,
            activation: Relu::new(),
            _phantom: std::marker::PhantomData,
        }
    }
}

/// Neural IK model: MLP that predicts joint angles from target position (and optionally current state).
#[derive(Module, Debug)]
pub struct NeuralIkModel<B: Backend> {
    /// Linear layers (last one is output, no activation).
    layers: Vec<Linear<B>>,
    activation: Relu,
    _phantom: std::marker::PhantomData<B>,
}

impl<B: Backend> NeuralIkModel<B> {
    /// Forward pass. Input shape `[batch_size, input_size]`, output `[batch_size, dof]`.
    pub fn forward(&self, x: Tensor<B, 2>) -> Tensor<B, 2> {
        let n = self.layers.len();
        let mut x = self.layers[0].forward(x);
        x = self.activation.forward(x);
        for i in 1..n - 1 {
            x = self.layers[i].forward(x);
            x = self.activation.forward(x);
        }
        self.layers[n - 1].forward(x)
    }
}
