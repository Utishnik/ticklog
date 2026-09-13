//! Experimental SPSC ring backed by the `ringbuf` crate, selected with the
//! `backend-ringbuf` feature.
//!
//! `ringbuf` is a lock-free SPSC FIFO that stores `u8` items and exposes the
//! shared storage through a split [`Producer`]/[`Consumer`] pair. Records are
//! staged by the producer and pushed as one byte slice per record; the drain
//! pops available bytes into its staging buffer.
//!
//! Unlike the `backend-ringbuffer` backend, the producer and the drain each
//! lock only their **own** half of the channel (`HeapProd` vs `HeapCons`), so
//! they never contend for a shared mutex on the hot path — the reverse of the
//! single-lock FIXO of `ringbuffer_backend`.
//!
//! # Limitations
//!
//! - Hot-path cost: every producer `commit` locks the producer mutex and
//!   pushes one slice. The atomic read/write index pair is shared with the
//!   drain, so the producer still touches a cross-core cache line on each
//!   publish (unlike the custom ring's per-thread cached tail).
//! - Capacity: one byte of slack is kept (same convention as
//!   `backend-ringbuffer`) so the buffer is never observed at the exact full
//!   bound, which some ring implementations treat as an overwrite condition.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use ringbuf::traits::{Consumer as _, Observer as _, Producer as _, Split};

use crate::builder::Backpressure;
use crate::ring::Reservation;

/// A byte FIFO ring backed by [`ringbuf::HeapRb`].
pub(crate) struct RingBuffer {
    /// Producer half of the split ring. Locked only by `commit`/`reserve`.
    prod: Mutex<ringbuf::HeapProd<u8>>,
    /// Consumer half of the split ring. Locked only by the drain.
    cons: Mutex<ringbuf::HeapCons<u8>>,
    /// Set to `false` by the producer on thread exit. The drain reads with
    /// Acquire to detect dead rings whose remaining bytes have been consumed.
    pub(crate) live: AtomicBool,
    /// Declared capacity (must be a power of two and at least one slot).
    capacity: usize,
}

// SAFETY: `HeapProd`/`HeapCons` are Send (they share the storage through an
// internal Arc) and the split protocol guarantees exactly one producer and one
// consumer, each confined to its own half. The mutexes never serialize the two
// sides on the hot path; the `AtomicBool` live flag follows the same
// acquire/release contract as the custom backend.
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
        let rb = ringbuf::HeapRb::<u8>::new(capacity);
        let (prod, cons) = rb.split();
        Self {
            prod: Mutex::new(prod),
            cons: Mutex::new(cons),
            live: AtomicBool::new(true),
            capacity,
        }
    }

    /// Reserves space for a record of `total_size` bytes.
    ///
    /// Returns a dummy [`Reservation`] when space is available. The actual
    /// byte write happens in [`commit`](Self::commit).
    ///
    /// Under [`Backpressure::Block`], spins until the drain frees space; under
    /// [`Backpressure::Drop`], returns `None` immediately when there is not
    /// enough room.
    pub(crate) fn reserve(&self, total_size: usize, policy: Backpressure) -> Option<Reservation> {
        loop {
            let prod = self.prod.lock().unwrap_or_else(|e| e.into_inner());
            // Leave one byte of slack, mirroring `backend-ringbuffer`, so the
            // FIFO is never observed at its exact full bound.
            if prod.occupied_len().saturating_add(total_size) < self.capacity {
                drop(prod);
                return Some(Reservation {
                    ptr: std::ptr::null_mut(),
                    head: 0,
                });
            }
            drop(prod);
            match policy {
                Backpressure::Drop => return None,
                Backpressure::Block => {
                    if !self.live.load(Ordering::Relaxed) {
                        return None;
                    }
                    std::hint::spin_loop();
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
    /// has been written into `bytes`. `reserve` guaranteed `total_size + 1`
    /// bytes of room and nothing else frees space for this producer, so
    /// `push_slice` must copy the entire slice in one call.
    pub(crate) fn commit(&self, bytes: &[u8]) {
        let mut prod = self.prod.lock().unwrap_or_else(|e| e.into_inner());
        let pushed = prod.push_slice(bytes);
        debug_assert_eq!(
            pushed,
            bytes.len(),
            "reserve guaranteed the whole record would fit"
        );
    }

    /// Moves every buffered byte into `out`, returning how many were moved.
    /// Called by the drain; the producer never invokes this method.
    pub(crate) fn pop_available(&self, out: &mut Vec<u8>) -> usize {
        let mut cons = self.cons.lock().unwrap_or_else(|e| e.into_inner());
        if cons.is_empty() {
            return 0;
        }
        let avail = cons.occupied_len();
        if avail == 0 {
            return 0;
        }
        let before = out.len();
        out.resize(before + avail, 0);
        let popped = cons.pop_slice(&mut out[before..]);
        out.truncate(before + popped);
        popped
    }

    /// Whether the FIFO currently holds no bytes.
    #[allow(dead_code)] // used only by tests
    pub(crate) fn is_empty(&self) -> bool {
        self.cons.lock().unwrap_or_else(|e| e.into_inner()).is_empty()
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
        assert!(!rb.is_empty());
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
        assert!(rb.is_empty());
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