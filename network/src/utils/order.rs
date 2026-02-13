//! Event ordering: sequence numbers and ordered envelopes for a canonical stream.

/// Monotonic sequence generator for assigning order to messages or events.
///
/// Server or authoritative node can hold one per stream/session so all
/// participants can order updates.
#[derive(Debug, Clone, Default)]
pub struct SequenceGenerator {
    next: u64,
}

impl SequenceGenerator {
    /// Create a new generator starting at 0.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a generator that starts after a given sequence (e.g. after replay).
    #[must_use]
    pub fn after(sequence: u64) -> Self {
        Self {
            next: sequence.wrapping_add(1),
        }
    }

    /// Allocate the next sequence number.
    pub fn next(&mut self) -> u64 {
        let s = self.next;
        self.next = self.next.wrapping_add(1);
        s
    }

    /// Current next value (without advancing).
    #[must_use]
    pub fn peek(&self) -> u64 {
        self.next
    }
}

/// Envelope that wraps a payload with ordering metadata so receivers can sort and apply in order.
#[derive(Debug, Clone)]
pub struct OrderedEnvelope<T> {
    /// Monotonic sequence number (e.g. from [SequenceGenerator]).
    pub sequence_number: u64,
    /// Optional timestamp in milliseconds (e.g. sim time or wall time).
    pub timestamp_ms: Option<u64>,
    /// Optional source id (e.g. client or server id).
    pub source_id: Option<String>,
    /// Payload.
    pub payload: T,
}

impl<T> OrderedEnvelope<T> {
    /// Create an envelope with sequence number and payload.
    #[must_use]
    pub fn new(sequence_number: u64, payload: T) -> Self {
        Self {
            sequence_number,
            timestamp_ms: None,
            source_id: None,
            payload,
        }
    }

    /// Set optional timestamp (milliseconds).
    #[must_use]
    pub fn with_timestamp(mut self, timestamp_ms: u64) -> Self {
        self.timestamp_ms = Some(timestamp_ms);
        self
    }

    /// Set optional source id.
    #[must_use]
    pub fn with_source_id(mut self, source_id: impl Into<String>) -> Self {
        self.source_id = Some(source_id.into());
        self
    }

    /// Ordering key: lower sequence number = earlier. Use for sorting.
    #[must_use]
    pub fn order_key(&self) -> u64 {
        self.sequence_number
    }
}

/// Compare two envelopes by sequence for ordering (earlier first).
#[must_use]
pub fn order_by_sequence<T>(a: &OrderedEnvelope<T>, b: &OrderedEnvelope<T>) -> std::cmp::Ordering {
    a.sequence_number.cmp(&b.sequence_number)
}
