//! Network timestamps, tick counter, sim time, and tick-based scheduling.

use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Returns a monotonic-style tick count (seconds since UNIX_EPOCH as f64).
/// Suitable for timestamps in protocol messages.
#[must_use]
pub fn network_timestamp() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

/// Tick counter for game loop / state updates.
#[derive(Debug, Clone, Copy, Default)]
pub struct TickCounter {
    tick: u64,
}

impl TickCounter {
    /// Create a new tick counter starting at 0.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance tick and return the new value.
    pub fn advance(&mut self) -> u64 {
        self.tick = self.tick.wrapping_add(1);
        self.tick
    }

    /// Current tick value.
    #[must_use]
    pub fn current(&self) -> u64 {
        self.tick
    }

    /// Elapsed ticks between two tick values (handles wrap-around).
    #[must_use]
    pub fn elapsed(from: u64, to: u64) -> u64 {
        to.wrapping_sub(from)
    }
}

/// Synchronized simulation time: advances from a base timestamp using ticks.
///
/// Same idea as "time since session start" driven by a canonical tick source
/// (server or agreed tick rate). All participants using the same base and
/// tick duration see identical sim time for a given tick.
#[derive(Debug, Clone)]
pub struct SimTime {
    /// Base time in milliseconds (e.g. session start).
    base_ms: u64,
    /// Milliseconds per tick (fixed tick duration).
    tick_duration_ms: f64,
    /// Current tick count.
    tick: u64,
}

impl SimTime {
    /// Create sim time with a base (ms) and tick duration (ms per tick).
    #[must_use]
    pub fn new(base_ms: u64, tick_duration_ms: f64) -> Self {
        Self {
            base_ms,
            tick_duration_ms: tick_duration_ms.max(0.0),
            tick: 0,
        }
    }

    /// Current sim time in milliseconds.
    #[must_use]
    pub fn now_ms(&self) -> u64 {
        self.base_ms
            .saturating_add((self.tick_duration_ms * self.tick as f64) as u64)
    }

    /// Current tick.
    #[must_use]
    pub fn tick(&self) -> u64 {
        self.tick
    }

    /// Advance by one tick and return the new tick.
    pub fn advance_tick(&mut self) -> u64 {
        self.tick = self.tick.wrapping_add(1);
        self.tick
    }

    /// Set tick (e.g. when syncing from server).
    pub fn set_tick(&mut self, tick: u64) {
        self.tick = tick;
    }

    /// Tick duration in milliseconds.
    #[must_use]
    pub fn tick_duration_ms(&self) -> f64 {
        self.tick_duration_ms
    }

    /// Base time in milliseconds.
    #[must_use]
    pub fn base_ms(&self) -> u64 {
        self.base_ms
    }
}

/// Tag for a scheduled action (opaque id chosen by the caller).
pub type ScheduledTag = u64;

/// Tick-based scheduler: run actions after N ticks.
///
/// Poll with the current tick to get due tags; the application decides
/// what to do for each tag (e.g. dispatch to a handler).
#[derive(Debug, Default)]
pub struct TickScheduler {
    pending: VecDeque<(u64, ScheduledTag)>,
    next_tag: ScheduledTag,
}

impl TickScheduler {
    /// Create a new empty scheduler.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Schedule an action to run at tick `run_at_tick`. Returns a tag that will
    /// be returned from `poll` when that tick is reached.
    pub fn schedule(&mut self, run_at_tick: u64, tag: ScheduledTag) {
        self.pending.push_back((run_at_tick, tag));
    }

    /// Schedule an action after `delay_ticks` from now; assigns a new tag and returns it.
    pub fn schedule_after(&mut self, current_tick: u64, delay_ticks: u64) -> ScheduledTag {
        let tag = self.next_tag;
        self.next_tag = self.next_tag.wrapping_add(1);
        self.schedule(current_tick.saturating_add(delay_ticks), tag);
        tag
    }

    /// Return all tags that are due (run_at_tick <= current_tick) and remove them.
    #[must_use]
    pub fn poll(&mut self, current_tick: u64) -> Vec<ScheduledTag> {
        let mut due = Vec::new();
        while let Some(&(run_at, tag)) = self.pending.front() {
            if run_at <= current_tick {
                self.pending.pop_front();
                due.push(tag);
            } else {
                break;
            }
        }
        due
    }

    /// Number of pending scheduled actions.
    #[must_use]
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}
