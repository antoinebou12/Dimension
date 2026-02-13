//! Rate limiter: at most N messages per time window.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Strategy when over the rate limit.
#[derive(Debug, Clone, Copy)]
pub enum ThrottleStrategy {
    /// Drop new messages when over limit.
    Drop,
    /// Coalesce: keep only the latest message (overwrites pending).
    CoalesceLatest,
}

/// Throttle: at most `max_per_window` messages per `window` duration.
///
/// When over limit, either drops new messages or coalesces to the latest (see [`ThrottleStrategy`]).
#[derive(Debug)]
pub struct Throttle {
    max_per_window: usize,
    window: Duration,
    strategy: ThrottleStrategy,
    timestamps: VecDeque<Instant>,
    /// When using CoalesceLatest, we store one coalesced item (as "has pending").
    coalesced: bool,
}

impl Throttle {
    /// Create a throttle: at most `max_per_window` events per `window`.
    #[must_use]
    pub fn new(max_per_window: usize, window: Duration) -> Self {
        Self {
            max_per_window,
            window,
            strategy: ThrottleStrategy::Drop,
            timestamps: VecDeque::new(),
            coalesced: false,
        }
    }

    /// Use drop strategy when over limit.
    #[must_use]
    pub fn with_drop(mut self) -> Self {
        self.strategy = ThrottleStrategy::Drop;
        self
    }

    /// Use coalesce strategy: when over limit, keep only the latest (one slot).
    #[must_use]
    pub fn with_coalesce(mut self) -> Self {
        self.strategy = ThrottleStrategy::CoalesceLatest;
        self
    }

    /// Evict timestamps outside the current window.
    fn evict_old(&mut self, now: Instant) {
        let cutoff = now.checked_sub(self.window).unwrap_or(now);
        while let Some(&t) = self.timestamps.front() {
            if t < cutoff {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
    }

    /// Returns true if a new message is allowed (under the limit).
    #[must_use]
    pub fn allow(&mut self, now: Instant) -> bool {
        self.evict_old(now);
        if self.timestamps.len() < self.max_per_window {
            self.timestamps.push_back(now);
            self.coalesced = false;
            true
        } else {
            match self.strategy {
                ThrottleStrategy::Drop => false,
                ThrottleStrategy::CoalesceLatest => {
                    self.coalesced = true;
                    false
                }
            }
        }
    }

    /// Call from the same clock used for `allow` (e.g. `Instant::now()`). Returns true if allowed.
    pub fn allow_now(&mut self) -> bool {
        self.allow(Instant::now())
    }

    /// When using CoalesceLatest, returns true if there is a coalesced (latest) update to emit.
    #[must_use]
    pub fn take_coalesced(&mut self) -> bool {
        let v = self.coalesced;
        self.coalesced = false;
        v
    }
}
