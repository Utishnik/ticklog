//! Experimental SPSC ring backed by the `ringbuffer` crate, selected with the
//! `backend-ringbuffer` feature.
//!
//! The crate stores typed elements (here `u8`) and keeps its internal storage
//! private, so records are staged by the producer and pushed as a byte stream.
//! The FIFO is guarded by a [`std::sync::Mutex`] because the crate's read/write
//! API requires `&mut self`; the custom ring's lock-free design exists precisely
//! to avoid this cost.
//!
//! Records are framed exactly like the custom backend (version, type,
//! total_size, …), but nothing is slot-aligned and no `END_OF_BUFFER`
//! fillers are written: a logical FIFO never wraps.
//!
//! # Limitations
//!
//! - Hot-path cost: every producer commit locks the mutex and pushes one byte
//!   at a time. This is intentionally the upper-bound "what does a crate with
//!   an opaque byte-FIFO cost us" baseline for comparison.
//! - Drain locking: the drain holds the mutex while it pops all available
//!   records, so the producer blocks during drain I/O. Acceptable for a bench
//!   backend; never use this for production latency-sensitive logging.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use ringbuffer::RingBuffer as _;

use crate::builder::Backpressure;
use crate::ring::Reservation;

/// A byte FIFO ring backed by [`ringbuffer::AllocRingBuffer`].
pub(crate) struct RingBuffer {
    /// The backing FIFO. Shared between producer (`commit`) and drain via a
    /// mutex: `ringbuffer` requires `&mut self` for both `push` and `pop`.
    pub(crate) fifo: Mutex<ringbuffer::AllocRingBuffer<u8>>,
    /// Set to `false` by the producer on thread exit. The drain reads with
    /// Acquire to detect dead rings whose remaining bytes have been consumed.
    pub(crate) live: AtomicBool,
    /// Declared capacity (must be a power of two and at least one slot).
    capacity: usize,
    /// Stable thread id of this ring's producer, set at registration (see
    /// [`RingBuffer::set_thread_info`]). Records carry no thread bytes.
    thread_id: AtomicU64,
    /// Producer thread name, set at registration.
    thread_name: Mutex<String>,
}

// SAFETY: `ringbuffer::AllocRingBuffer<u8>` is Send; the Mutex makes it Sync;
// `AtomicBool` is Send+Sync. The live flag follows the same SPSC acquire/release
// contract as the custom backend.
unsafe impl Sync for RingBuffer {}

impl RingBuffer {
    /// Creates a new ring buffer of the default
    /// [`DEFAULT_RING_SIZE`](crate::ring::DEFAULT_RING_SIZE).
    #[allow(dead_code)] // used only by tests; `with_capacity` is the hot path
    pub(crate) fn new() -> Self {
        Self::with_capacity(crate::ring::DEFAULT_RING_SIZE)
    }

    /// Creates a new FIFO ring of `capacity` bytes.
    ///
    /// # Panics
    ///
    /// Panics if `capacity` is not a power of two or is smaller than
    /// [`SLOT_SIZE`](crate::ring::SLOT_SIZE).
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        assert!(
            capacity.is_power_of_two() && capacity >= crate::ring::SLOT_SIZE,
            "invariant: ring capacity must be a power of two >= SLOT_SIZE, got {capacity}"
        );
        Self {
            fifo: Mutex::new(ringbuffer::AllocRingBuffer::new(capacity)),
            live: AtomicBool::new(true),
            thread_id: AtomicU64::new(0),
            thread_name: Mutex::new(String::new()),
            capacity,
        }
    }

    /// Records this ring's producer identity. See [`crate::ring`]'s custom
    /// backend for the rationale; the FIFO backends mirror it so
    /// [`crate::thread_buf`] and the drain share one interface.
    pub(crate) fn set_thread_info(&self, thread_id: u64, thread_name: &str) {
        self.thread_id.store(thread_id, Ordering::Relaxed);
        let mut guard = self.thread_name.lock().unwrap_or_else(|e| e.into_inner());
        *guard = thread_name.to_string();
    }

    /// Identity used by the drain when formatting this ring's records.
    pub(crate) fn thread_id(&self) -> u64 {
        self.thread_id.load(Ordering::Relaxed)
    }

    /// Identity used by the drain when formatting this ring's records.
    pub(crate) fn thread_name(&self) -> String {
        self.thread_name
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }

    /// Reserves space for a record of `total_size` bytes.
    ///
    /// Returns a dummy [`Reservation`] when space is available. The actual byte
    /// write happens in [`commit`](Self::commit).
    ///
    /// Under [`Backpressure::Block`], spins until the drain frees space; under
    /// [`Backpressure::Drop`], returns `None` immediately when there is not
    /// enough room.
    pub(crate) fn reserve(&self, total_size: usize, policy: Backpressure) -> Option<Reservation> {
        let mut backoff = crate::backoff::Backoff::new();
        loop {
            // Lock, check capacity, release before potentially blocking.
            let len = self.fifo.lock().unwrap_or_else(|e| e.into_inner()).len();
            // Leave one byte of slack: some ring buffer implementations mark the
            // FIFO full when `len == capacity`, overwriting the oldest element;
            // we must never let that happen.
            if len.saturating_add(total_size) < self.capacity {
                return Some(Reservation {
                    ptr: std::ptr::null_mut(),
                    head: 0,
                });
            }
            match policy {
                Backpressure::Drop => return None,
                // Segmented policies require the crate-local ring and are
                // degraded to Block by `configure!` under a FIFO backend.
                Backpressure::NanoLog | Backpressure::Quill | Backpressure::Block => {
                    if !self.live.load(Ordering::Relaxed) {
                        return None;
                    }
                    backoff.wait();
                }
            }
        }
    }

    /// Publish is a no-op for the FIFO backend: the real write happens inside
    /// [`commit`](Self::commit).
    #[allow(dead_code)] // part of the shared RingBuffer interface; custom-only
    #[inline(always)]
    pub(crate) fn publish(&self, _r: Reservation) {}

    /// Pushes the fully assembled record bytes into the FIFO.
    ///
    /// Must be called after [`reserve`](Self::reserve) succeeds and the record
    /// has been written into `bytes`. The whole record is committed under one
    /// lock acquisition, so the drain (which locks the same mutex to pop) can
    /// never observe a partial record.
    pub(crate) fn commit(&self, bytes: &[u8]) {
        let mut fifo = self.fifo.lock().unwrap_or_else(|e| e.into_inner());
        for &b in bytes {
            fifo.enqueue(b);
        }
    }

    /// Moves every buffered byte into `out`, returning how many were moved.
    /// Called by the drain; the producer never invokes this method.
    pub(crate) fn pop_available(&self, out: &mut Vec<u8>) -> usize {
        let mut fifo = self.fifo.lock().unwrap_or_else(|e| e.into_inner());
        let mut moved = 0;
        while let Some(b) = fifo.dequeue() {
            out.push(b);
            moved += 1;
        }
        moved
    }

    /// Whether the FIFO currently holds no bytes.
    #[allow(dead_code)] // used only by tests
    pub(crate) fn is_empty(&self) -> bool {
        self.fifo
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CAP: usize = crate::ring::SLOT_SIZE * 256; // large enough for a few records

    #[test]
    fn new_default_capacity() {
        let rb = RingBuffer::new();
        assert_eq!(rb.capacity, crate::ring::DEFAULT_RING_SIZE);
    }

    #[test]
    fn reserve_and_commit_fills_fifo() {
        let rb = RingBuffer::with_capacity(CAP);
        let payload = vec![0xABu8; 100];
        let slot = rb.reserve(payload.len(), Backpressure::Drop).unwrap();
        assert!(slot.ptr.is_null()); // dummy for FIFO
        rb.commit(&payload);
        assert!(rb.live.load(Ordering::Relaxed));
    }

    #[test]
    fn reserve_drops_when_full() {
        // Tiny ring; a record that fills (capacity - 1) bytes leaves only the
        // 1-byte slack, so a second reserve must fail under the Drop policy.
        let rb = RingBuffer::with_capacity(crate::ring::SLOT_SIZE);
        let payload = vec![0u8; crate::ring::SLOT_SIZE - 1];
        let _slot = rb.reserve(payload.len(), Backpressure::Drop).unwrap();
        rb.commit(&payload);
        // Next reserve must fail under Drop policy.
        assert!(rb.reserve(1, Backpressure::Drop).is_none());
    }

    #[test]
    fn publish_is_noop() {
        let rb = RingBuffer::with_capacity(CAP);
        let slot = rb.reserve(1, Backpressure::Drop).unwrap();
        rb.publish(slot); // must not panic or corrupt state
        assert_eq!(rb.fifo.lock().unwrap().len(), 0);
    }

    #[test]
    fn pop_available_moves_all_bytes_in_order() {
        let rb = RingBuffer::with_capacity(CAP);
        rb.commit(b"abc");
        rb.commit(b"def");
        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), 6);
        assert_eq!(out, b"abcdef");
        assert!(rb.is_empty());
    }
}
