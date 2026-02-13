//! Client connection manager.

use std::time::Duration;

use crate::utils::ReconnectPolicy;

/// Connection configuration.
#[derive(Debug, Clone)]
pub struct ConnectionConfig {
    /// Server base URL (e.g. "http://127.0.0.1:3000").
    pub base_url: String,
    /// Reconnect delay on disconnect (used when `reconnect_policy` is None).
    pub reconnect_delay: Duration,
    /// Optional reconnect policy for exponential backoff and retry limits.
    pub reconnect_policy: Option<ReconnectPolicy>,
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            base_url: "http://127.0.0.1:3000".to_string(),
            reconnect_delay: Duration::from_secs(2),
            reconnect_policy: None,
        }
    }
}

impl ConnectionConfig {
    /// Delay before the next reconnect attempt (0-based attempt index).
    /// Uses `reconnect_policy` if set, otherwise fixed `reconnect_delay`.
    #[must_use]
    pub fn next_reconnect_delay(&self, attempt: u32) -> Duration {
        self.reconnect_policy
            .as_ref()
            .map(|p| p.next_delay(attempt))
            .unwrap_or(self.reconnect_delay)
    }

    /// Whether to retry after the given attempt. Uses `reconnect_policy` if set, otherwise always true.
    #[must_use]
    pub fn should_retry(&self, attempt: u32) -> bool {
        self.reconnect_policy
            .as_ref()
            .map(|p| p.should_retry(attempt))
            .unwrap_or(true)
    }
}
