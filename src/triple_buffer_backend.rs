//! Experimental single-thread (SPSC) backend backed by the `triple_buffer`
//! crate, selected with the `backend-triple-buffer` feature.
//!
//! `triple_buffer` is a publisher/subscriber scheme that keeps three buffers
//! and publishes one **entire value** per shot. ticklog uses it as a
//! single-slot exchange: the producer writes the completed record bytes into
//! the input buffer and publishes them; the drain updates once per poll and
//! copies the latest value out.
//!
//! # Semantics and limitations (read before benchmarking)
//!
//! - **Exactly one slot.** There is no ring to fill, so `reserve` never fails
//!   for lack of space. The only "full" state is a value the drain has not yet
//!   consumed, and publishing overwrites it, so a correct producer must always
//!   wait for the previous value to be consumed.
//! - **Always lossless and ordered at the macro level.** [`reserve`] spins
//!   with *both* backpressure policies until the drain consumed the previous
//!   value. `Backpressure::Drop` deliberately degrades to `Block`: a
//!   single-slot channel has no "free capacity" to drop into, and honoring Drop
//!   would drop nearly every call.
//! - **The only loss path bypasses [`reserve`].** Calling [`commit`] directly
//!   while a value is outstanding overwrites it (the classic
//!   latest-value-wins triple-buffer behaviour). The macro path never does
//!   this; the unit test `new_publish_overwrites_outstanding_value` and the
//!   drain test `triple_buffer_drain_delivers_latest_record_only` exercise it
//!   explicitly.
//! - **Endpoint cost.** Every producer `commit` and every drain poll touch the
//!   crate's shared atomic state, and the producer blocks until the drain
//!   consumes each value — a strict per-record producer↔drain handshake.
//!   Benchmark this backend as a "synchronous single-slot channel" baseline,
//!   not a buffered one.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use triple_buffer::{Input, Output};

use crate::builder::Backpressure;
use crate::ring::Reservation;

/// A single-value exchange backed by [`triple_buffer::Input`]/[`Output`].
///
/// The producer writes records into the [`Input`] half and the drain reads
/// them from the [`Output`] half. The two halves are guarded by independent
/// mutexes so producer and drain never contend on the hot path.
pub(crate) struct RingBuffer {
    /// Producer half. Only `commit`/`reserve` touch it.
    input: Mutex<Input<Vec<u8>>>,
    /// Consumer half. Only the drain touches it.
    output: Mutex<Output<Vec<u8>>>,
    /// Set to `false` by the producer on thread exit; the drain uses it to
    /// detect a dead ring whose outstanding records have been consumed.
    pub(crate) live: AtomicBool,
}

impl RingBuffer {
    /// Creates a new triple-buffer-slot exchange of the default
    /// [`DEFAULT_RING_SIZE`](crate::ring::DEFAULT_RING_SIZE).
    #[allow(dead_code)] // used only by tests; `with_capacity` is the hot path
    pub(crate) fn new() -> Self {
        Self::with_capacity(crate::ring::DEFAULT_RING_SIZE)
    }

    /// Creates a new single-value exchange. `capacity` is accepted for
    /// interface parity and must be a power of two, but has no effect: this
    /// backend has exactly one slot.
    ///
    /// The internal value buffers are preallocated to
    /// [`MAX_RECORD_SIZE`](crate::record::MAX_RECORD_SIZE) so `commit`'s
    /// `clear` + `extend_from_slice` never reallocates on the hot path. That
    /// allocation happens here (during `warm_up`), keeping the first post-warm
    /// log call allocation-free, matching the custom and FIFO backends.
    pub(crate) fn with_capacity(capacity: usize) -> Self {
        debug_assert!(capacity.is_power_of_two());
        // Seed the buffer with MAX_RECORD_SIZE bytes: `Vec::clone` reserves
        // `len()` (not `capacity()`), so an empty pre-sized Vec clones into
        // three *zero-capacity* buffers and the first `commit`'s extend would
        // reallocate on the hot path. With the length at the worst case, every
        // internal clone carries capacity MAX_RECORD_SIZE and `commit` never
        // allocates for any record that passes the size gate.
        let initial = vec![0u8; crate::record::MAX_RECORD_SIZE];
        let (input, output) = triple_buffer::triple_buffer(&initial);
        Self {
            input: Mutex::new(input),
            output: Mutex::new(output),
            live: AtomicBool::new(true),
        }
    }

    /// Reserves the single slot for a record of `total_size` bytes.
    ///
    /// Space never fails — the only "full" state is a value the drain has not
    /// consumed yet, and publishing would overwrite it. So this method spins
    /// until the previous value has been consumed, **for both backpressure
    /// policies**: with one slot there is no "free capacity" a dropped record
    /// could free up, so [`Backpressure::Drop`] deliberately degrades to
    /// [`Backpressure::Block`]. The exchange is therefore lossless and
    /// ordered under normal (macro) usage.
    ///
    /// Returns `None` only at shutdown (`live == false`, mirroring the custom
    /// backend's Block-exit behaviour) so a producer never spins forever
    /// against a dead drain.
    pub(crate) fn reserve(&self, _total_size: usize, _policy: Backpressure) -> Option<Reservation> {
        debug_assert!(_total_size <= crate::record::MAX_RECORD_SIZE);
        let mut backoff = crate::backoff::Backoff::new();
        loop {
            let input = self.input.lock().unwrap_or_else(|e| e.into_inner());
            if input.consumed() {
                drop(input);
                return Some(Reservation {
                    ptr: std::ptr::null_mut(),
                    head: 0,
                });
            }
            drop(input);
            if !self.live.load(Ordering::Relaxed) {
                return None;
            }
            backoff.wait();
        }
    }

    /// Publish is a no-op for this backend: the real write runs inside
    /// [`commit`](Self::commit).
    #[allow(dead_code)] // part of the shared RingBuffer interface; custom-only
    #[inline(always)]
    pub(crate) fn publish(&self, _r: Reservation) {}

    /// Writes `bytes` as the single outstanding value and publishes it.
    ///
    /// Must follow a successful [`reserve`](Self::reserve) so the previous
    /// value is already consumed; `publish` then flips the input buffer back
    /// to the producer without reallocating.
    pub(crate) fn commit(&self, bytes: &[u8]) {
        let mut input = self.input.lock().unwrap_or_else(|e| e.into_inner());
        let buf = input.input_buffer_mut();
        buf.clear();
        buf.extend_from_slice(bytes);
        input.publish();
    }

    /// Copies the latest published value (if any) into `out`, returning how
    /// many bytes were copied.
    ///
    /// Called by the drain. Each call advances to the newest published value,
    /// so older outstanding values are skipped (the lossy behaviour inherent
    /// to the triple-buffer scheme).
    pub(crate) fn pop_available(&self, out: &mut Vec<u8>) -> usize {
        let mut output = self.output.lock().unwrap_or_else(|e| e.into_inner());
        if output.update() {
            let data = output.output_buffer();
            let n = data.len();
            if n > 0 {
                out.extend_from_slice(data);
            }
            n
        } else {
            0
        }
    }

    /// Whether the exchange currently holds no value that the drain has not
    /// yet fetched.
    #[allow(dead_code)] // used only by tests
    pub(crate) fn is_empty(&self) -> bool {
        !self.output.lock().unwrap_or_else(|e| e.into_inner()).updated()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commit_and_pop_delivers_the_record() {
        let rb = RingBuffer::with_capacity(crate::ring::DEFAULT_RING_SIZE);
        let slot = rb.reserve(5, Backpressure::Drop).unwrap();
        assert!(slot.ptr.is_null());
        rb.commit(b"hello");
        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), 5);
        assert_eq!(out, b"hello");
        assert!(rb.is_empty());
    }

    #[test]
    fn reserve_waits_for_drain_to_consume() {
        let rb = std::sync::Arc::new(RingBuffer::with_capacity(
            crate::ring::DEFAULT_RING_SIZE,
        ));
        // Publish a value; the drain has not consumed it yet.
        rb.reserve(1, Backpressure::Drop).unwrap();
        rb.commit(b"x");

        // A second `reserve` must wait (both policies block until the drain
        // consumes) — the single-slot rendezvous.
        let rb2 = std::sync::Arc::clone(&rb);
        let handle = std::thread::spawn(move || {
            let slot = rb2.reserve(1, Backpressure::Block).unwrap();
            assert!(slot.ptr.is_null());
        });

        // Give the waiter a moment to block on the outstanding value.
        std::thread::sleep(std::time::Duration::from_millis(200));
        assert!(
            !handle.is_finished(),
            "reserve must block until the drain consumes the outstanding value"
        );

        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), 1);
        handle.join().unwrap();
    }

    #[test]
    fn reserve_returns_none_when_drain_shut_down() {
        let rb = RingBuffer::with_capacity(crate::ring::DEFAULT_RING_SIZE);
        // Publish a value and shut the producer side down while it is still
        // outstanding: the next reserve must bail out instead of spinning.
        rb.reserve(1, Backpressure::Block).unwrap();
        rb.commit(b"x");
        rb.live.store(false, Ordering::Relaxed);
        assert!(rb.reserve(1, Backpressure::Block).is_none());
    }

    #[test]
    fn direct_commit_overwrites_outstanding_value() {
        let rb = RingBuffer::with_capacity(crate::ring::DEFAULT_RING_SIZE);
        // Calling `commit` straight into an unconsumed slot (no reserve
        // rendezvous) is the one loss path: the older value is overwritten.
        rb.commit(b"first-older");
        rb.commit(b"second");
        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), 6);
        assert_eq!(out, b"second"); // only the latest survives
        assert!(rb.is_empty());
    }

    #[test]
    fn pop_returns_zero_when_nothing_published() {
        let rb = RingBuffer::with_capacity(crate::ring::DEFAULT_RING_SIZE);
        assert!(rb.is_empty());
        let mut out = Vec::new();
        assert_eq!(rb.pop_available(&mut out), 0);
    }
}