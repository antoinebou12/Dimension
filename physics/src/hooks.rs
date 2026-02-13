//! Pre-substep and post-substep callbacks for the simulation loop.

/// Callbacks invoked by the solver at defined points in each substep.
///
/// All callbacks are optional; `None` means no-op.
pub struct SubstepHooks {
    /// Called at the start of each substep.
    pub pre_substep: Option<Box<dyn FnMut(u32, f32) + Send + Sync>>,
    /// Called at the end of each substep.
    pub post_substep: Option<Box<dyn FnMut(u32, f32) + Send + Sync>>,
}

impl SubstepHooks {
    /// Creates hooks with no callbacks.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pre_substep: None,
            post_substep: None,
        }
    }

    /// Invoke pre-substep hook if set.
    pub fn on_pre_substep(&mut self, substep: u32, dtau: f32) {
        if let Some(ref mut f) = self.pre_substep {
            f(substep, dtau);
        }
    }

    /// Invoke post-substep hook if set.
    pub fn on_post_substep(&mut self, substep: u32, dtau: f32) {
        if let Some(ref mut f) = self.post_substep {
            f(substep, dtau);
        }
    }
}

impl Default for SubstepHooks {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for SubstepHooks {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SubstepHooks")
            .field("pre_substep", &self.pre_substep.as_ref().map(|_| ".."))
            .field("post_substep", &self.post_substep.as_ref().map(|_| ".."))
            .finish()
    }
}
