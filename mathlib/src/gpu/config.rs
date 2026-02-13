//! GPU adapter and device configuration.
//!
//! Use [`GpuConfig`] with [`super::init_blocking`] or [`super::init_async`] to customize
//! adapter selection and limits.

/// Adapter selection hint for GPU initialization.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PowerPreference {
    /// No hint; let the implementation choose.
    #[default]
    Default,
    /// Favor lower power consumption (e.g. integrated GPU).
    LowPower,
    /// Favor higher performance (e.g. discrete GPU on multi-GPU systems).
    HighPerformance,
}

impl PowerPreference {
    pub(crate) fn to_wgpu(self) -> wgpu::PowerPreference {
        match self {
            PowerPreference::Default => wgpu::PowerPreference::default(),
            PowerPreference::LowPower => wgpu::PowerPreference::LowPower,
            PowerPreference::HighPerformance => wgpu::PowerPreference::HighPerformance,
        }
    }
}

/// Configuration for GPU adapter and device creation.
#[derive(Clone, Debug)]
pub struct GpuConfig {
    /// Adapter selection hint. HighPerformance favors discrete GPU on multi-GPU systems.
    pub power_preference: PowerPreference,
    /// Use software renderer if no hardware adapter (debugging only).
    pub force_fallback_adapter: bool,
    /// Request relaxed limits when possible (native only). On wasm, adapter.limits() is always used.
    pub relaxed_limits: bool,
}

impl Default for GpuConfig {
    fn default() -> Self {
        Self {
            power_preference: PowerPreference::Default,
            force_fallback_adapter: false,
            relaxed_limits: false,
        }
    }
}
