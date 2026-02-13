//! Batch buffer: collect items and flush after max count or max interval.

use std::time::{Duration, Instant};

/// Collects items and flushes when either max count or max interval is reached.
///
/// Push items; call `should_flush()` to decide when to flush, then `take_pending()` to get and clear the batch.
#[derive(Debug)]
pub struct BatchBuffer<T> {
    pending: Vec<T>,
    max_count: usize,
    max_interval: Duration,
    last_flush: Instant,
}

impl<T> BatchBuffer<T> {
    /// Create a buffer that flushes when `max_count` items are queued or `max_interval` has passed.
    #[must_use]
    pub fn new(max_count: usize, max_interval: Duration) -> Self {
        Self {
            pending: Vec::new(),
            max_count,
            max_interval,
            last_flush: Instant::now(),
        }
    }

    /// Push an item. Does not flush automatically.
    pub fn push(&mut self, item: T) {
        self.pending.push(item);
    }

    /// Returns true if the buffer should be flushed (max count reached or interval elapsed).
    #[must_use]
    pub fn should_flush(&self) -> bool {
        self.pending.len() >= self.max_count || self.last_flush.elapsed() >= self.max_interval
    }

    /// Take all pending items and reset the interval timer. Returns an empty vec if nothing pending.
    #[must_use]
    pub fn take_pending(&mut self) -> Vec<T> {
        self.last_flush = Instant::now();
        std::mem::take(&mut self.pending)
    }

    /// Number of items currently pending.
    #[must_use]
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    /// Returns true if there are no pending items.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }
}
