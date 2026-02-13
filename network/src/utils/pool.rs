//! Reusable byte buffer pool to reduce allocations.

use std::sync::{Arc, Mutex};

/// A simple pool of reusable `Vec<u8>` buffers.
#[derive(Clone, Default)]
pub struct BufferPool {
    inner: Arc<Mutex<Vec<Vec<u8>>>>,
}

impl BufferPool {
    /// Create a new buffer pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Acquire a buffer from the pool, or allocate a new one if empty.
    pub fn acquire(&self) -> Vec<u8> {
        self.inner
            .lock()
            .expect("pool mutex")
            .pop()
            .unwrap_or_default()
    }

    /// Return a buffer to the pool for reuse.
    pub fn release(&self, mut buf: Vec<u8>) {
        buf.clear();
        if let Ok(mut guard) = self.inner.lock() {
            guard.push(buf);
        }
    }
}
