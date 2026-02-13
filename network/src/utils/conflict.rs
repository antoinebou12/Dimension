//! Conflict resolution helpers: last-writer-wins and optional version vectors.

/// Last-writer-wins metadata: (timestamp_ms, source_id) for a single value.
///
/// Use for shared keys so apps can decide whether to apply an update without reimplementing LWW.
#[derive(Debug, Clone, Default)]
pub struct LastWriterWins {
    /// Timestamp in milliseconds (e.g. sim time or wall time).
    pub timestamp_ms: u64,
    /// Opaque source id (e.g. client or server id).
    pub source_id: String,
}

impl LastWriterWins {
    /// Create LWW metadata.
    #[must_use]
    pub fn new(timestamp_ms: u64, source_id: impl Into<String>) -> Self {
        Self {
            timestamp_ms,
            source_id: source_id.into(),
        }
    }

    /// Returns true if `new` should be applied over `current`: new is strictly newer,
    /// or same timestamp but new has a greater source_id (tie-break).
    #[must_use]
    pub fn should_apply(new: &LastWriterWins, current: Option<&LastWriterWins>) -> bool {
        match current {
            None => true,
            Some(c) => {
                new.timestamp_ms > c.timestamp_ms
                    || (new.timestamp_ms == c.timestamp_ms && new.source_id > c.source_id)
            }
        }
    }
}

/// Version vector: one counter per source for multi-key or multi-region state.
#[derive(Debug, Clone, Default)]
pub struct VersionVector {
    /// source_id -> counter
    pub counters: std::collections::HashMap<String, u64>,
}

impl VersionVector {
    /// Create an empty version vector.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Increment the counter for `source_id` and return the new value.
    pub fn increment(&mut self, source_id: impl Into<String>) -> u64 {
        let id = source_id.into();
        let v = self
            .counters
            .get(&id)
            .copied()
            .unwrap_or(0)
            .saturating_add(1);
        self.counters.insert(id, v);
        v
    }

    /// Get the counter for `source_id`.
    #[must_use]
    pub fn get(&self, source_id: &str) -> u64 {
        self.counters.get(source_id).copied().unwrap_or(0)
    }

    /// Merge another version vector: take the maximum counter per source.
    pub fn merge(&mut self, other: &VersionVector) {
        for (id, &c) in &other.counters {
            let current = self.counters.get(id).copied().unwrap_or(0);
            if c > current {
                self.counters.insert(id.clone(), c);
            }
        }
    }

    /// Returns true if `this` is strictly after `other` (at least one counter greater, rest >=).
    #[must_use]
    pub fn after(this: &VersionVector, other: &VersionVector) -> bool {
        let mut has_greater = false;
        for (id, &c_other) in &other.counters {
            let c_this = this.counters.get(id).copied().unwrap_or(0);
            if c_this < c_other {
                return false;
            }
            if c_this > c_other {
                has_greater = true;
            }
        }
        for (id, &c_this) in &this.counters {
            if !other.counters.contains_key(id) && c_this > 0 {
                has_greater = true;
            }
        }
        has_greater
    }
}
