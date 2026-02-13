//! Reconnection policy: exponential backoff and retry limits.

use std::time::Duration;

/// Policy for reconnection delays and retry limits.
#[derive(Debug, Clone)]
pub struct ReconnectPolicy {
    /// Initial delay after first failure.
    pub initial_delay: Duration,
    /// Maximum delay (cap for exponential backoff).
    pub max_delay: Duration,
    /// Multiplier for exponential backoff (e.g. 2.0 doubles each time).
    pub multiplier: f64,
    /// Maximum number of attempts (None = no limit).
    pub max_attempts: Option<u32>,
}

impl Default for ReconnectPolicy {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            max_attempts: None,
        }
    }
}

impl ReconnectPolicy {
    /// Create a new policy with the given parameters.
    #[must_use]
    pub fn new(
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        max_attempts: Option<u32>,
    ) -> Self {
        Self {
            initial_delay,
            max_delay,
            multiplier: multiplier.max(1.0),
            max_attempts,
        }
    }

    /// Delay to use before the next reconnect attempt (0-based attempt index).
    #[must_use]
    pub fn next_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return self.initial_delay;
        }
        let d = self.initial_delay.as_secs_f64() * self.multiplier.powi(attempt as i32);
        let d_ms = d as u64;
        let capped = Duration::from_millis(d_ms.min(self.max_delay.as_millis() as u64));
        capped.min(self.max_delay)
    }

    /// Returns true if the client should retry after `attempt` (0-based).
    #[must_use]
    pub fn should_retry(&self, attempt: u32) -> bool {
        match self.max_attempts {
            None => true,
            Some(max) => attempt < max,
        }
    }
}
