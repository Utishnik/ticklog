//! The drain thread: consumes encoded records from every registered ring
//! buffer and writes formatted log lines to the sink.
//!
//! This is the only module that performs raw-pointer arithmetic. All byte
//! access goes through [`Cursor`], the sole audit point for unsafe reads.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::encode::{FIXED_SIZES, TAG_COUNT, TAG_STR};
use crate::format::{self, Field, FormatSpec, Segment, Template};
use crate::level::Level;
use crate::record::{
    END_OF_BUFFER, FLAG_COMPLEX, FLAG_PROCESS, FLAG_SITE, FLAG_SOURCE, FLAG_THREAD, HEADER_SIZE,
    LOG_RECORD, VERSION,
};
use crate::ring::RingBuffer;
#[cfg(not(feature = "fifo-backend"))]
use crate::ring::{SLOT_SIZE, align_up};
use crate::sink::LogSink;
use crate::thread_buf::REGISTRY;
use crate::timestamp::{Calibration, format_iso8601, ticks_to_ns};

/// Spin batch used when the drain is freshly idle or has just done work.
const SPIN_MIN: u32 = 8;
/// Upper bound on the spin batch. Caps worst-case wakeup latency.
const SPIN_CAP: u32 = 256;
// Re-scan the ring registry once every this many poll iterations. Must be a
// power of two so the free-running sync counter stays aligned across its u32
// wrap. Bounds how stale the drain's local ring list can become without
// locking the registry mutex on every pass.
const SYNC_EVERY: u32 = 1024;

// Fail the build if SYNC_EVERY is ever set to a non-power-of-two, which would
// misalign the derived sync mask across the counter's u32 wrap.
const _: () = assert!(SYNC_EVERY.is_power_of_two());

/// A bounded raw-pointer byte parser over one record slice. Every read is
/// clamped to `[base, base + len)`, so a corrupt in-record length can never read
/// past the slice: a fixed-width read that would overrun yields `0`, and
/// `read_bytes` returns a short slice. This is defense-in-depth on a cooperative
/// single-process channel, and runs off the hot path.
struct Cursor {
    ptr: *const u8,
    end: *const u8,
}

impl Cursor {
    /// Creates a cursor over the `len`-byte record slice starting at `base`.
    ///
    /// # Safety
    ///
    /// `base` must point at the start of an initialized region of at least
    /// `len` bytes; reads never cross `base + len`.
    #[inline(always)]
    unsafe fn new(base: *const u8, len: usize) -> Self {
        Self {
            ptr: base,
            // SAFETY: the caller's contract guarantees `base` starts an
            // initialized region of at least `len` bytes, so `base + len` is at
            // most one-past-the-end and is valid to form (it is never dereferenced,
            // only compared against `ptr`).
            end: unsafe { base.add(len) },
        }
    }

    /// Bytes remaining before the end of the record slice.
    #[inline(always)]
    fn remaining(&self) -> usize {
        self.end as usize - self.ptr as usize
    }

    /// Reads the next `N` bytes as an array, advancing past them; returns
    /// `[0; N]` if fewer than `N` remain. `N` is inferred from the caller's
    /// `from_le_bytes` target type, so the width is never a literal.
    #[inline(always)]
    unsafe fn read_array<const N: usize>(&mut self) -> [u8; N] {
        if self.remaining() < N {
            return [0; N];
        }
        // SAFETY: N readable bytes remain; `[u8; N]` has alignment 1, so the
        // cast-and-copy is a valid unaligned load. The pointer advances past
        // exactly the N bytes consumed.
        let v = unsafe { *(self.ptr as *const [u8; N]) };
        self.ptr = unsafe { self.ptr.add(N) };
        v
    }

    #[inline(always)]
    unsafe fn read_u8(&mut self) -> u8 {
        // SAFETY: `read_array` is bounds-clamped (it yields `[0; N]` when fewer
        // than N bytes remain), so it never reads past `end`; here N = 1.
        u8::from_le_bytes(unsafe { self.read_array() })
    }

    #[inline(always)]
    unsafe fn read_u16(&mut self) -> u16 {
        // SAFETY: `read_array` is bounds-clamped and never reads past `end`; N = 2.
        u16::from_le_bytes(unsafe { self.read_array() })
    }

    #[inline(always)]
    unsafe fn read_u32(&mut self) -> u32 {
        // SAFETY: `read_array` is bounds-clamped and never reads past `end`; N = 4.
        u32::from_le_bytes(unsafe { self.read_array() })
    }

    #[inline(always)]
    unsafe fn read_u64(&mut self) -> u64 {
        // SAFETY: `read_array` is bounds-clamped and never reads past `end`; N = 8.
        u64::from_le_bytes(unsafe { self.read_array() })
    }

    #[inline(always)]
    unsafe fn skip(&mut self, n: usize) {
        let n = n.min(self.remaining());
        // SAFETY: `n` is clamped to the bytes remaining, so the advanced pointer
        // stays within `[base, end]`.
        self.ptr = unsafe { self.ptr.add(n) };
    }

    /// Returns the next `min(n, remaining)` bytes and advances past them. A
    /// short return means the record ended early (corruption).
    #[inline(always)]
    unsafe fn read_bytes(&mut self, n: usize) -> &[u8] {
        let n = n.min(self.remaining());
        // SAFETY: `n` is clamped to the remaining initialized bytes of the slice.
        let data = unsafe { std::slice::from_raw_parts(self.ptr, n) };
        self.ptr = unsafe { self.ptr.add(n) };
        data
    }

    /// Reads a `u16` length prefix, then returns the following
    /// `min(len, remaining)` bytes (clamped, like `read_bytes`).
    #[inline(always)]
    unsafe fn read_len_prefixed(&mut self) -> (&[u8], u16) {
        // SAFETY: `read_u16` and `read_bytes` are both bounds-clamped and never
        // read past `end`; a corrupt `len` only yields a short slice.
        let len = unsafe { self.read_u16() };
        let data = unsafe { self.read_bytes(len as usize) };
        (data, len)
    }
}

/// A formatter reads a fixed-size (or already length-stripped) argument payload,
/// decodes it to its native type, and appends the formatted value to `buf`.
type Formatter = fn(&[u8], &FormatSpec, &mut Vec<u8>);

/// Function-pointer dispatch table indexed by type tag (0x00..=0x0B). Sized to
/// [`TAG_COUNT`] so it stays in lockstep with [`FIXED_SIZES`]; `format_arg`
/// bounds the tag against `FIXED_SIZES.len()` before indexing here.
static FORMATTERS: [Formatter; TAG_COUNT] = [
    fmt_u64,  // 0x00
    fmt_i64,  // 0x01
    fmt_f64,  // 0x02
    fmt_u32,  // 0x03
    fmt_i32,  // 0x04
    fmt_f32,  // 0x05
    fmt_u16,  // 0x06
    fmt_i16,  // 0x07
    fmt_u8,   // 0x08
    fmt_i8,   // 0x09
    fmt_bool, // 0x0A
    fmt_str,  // 0x0B
];

fn fmt_u64(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    let bytes: [u8; 8] = data
        .try_into()
        .expect("invariant: u64 argument must be 8 bytes");
    format::format_u64(u64::from_le_bytes(bytes), spec, buf);
}

fn fmt_i64(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    let bytes: [u8; 8] = data
        .try_into()
        .expect("invariant: i64 argument must be 8 bytes");
    format::format_i64(i64::from_le_bytes(bytes), spec, buf);
}

fn fmt_f64(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    let bytes: [u8; 8] = data
        .try_into()
        .expect("invariant: f64 argument must be 8 bytes");
    format::format_f64(f64::from_le_bytes(bytes), spec, buf);
}

fn fmt_u32(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    let bytes: [u8; 4] = data
        .try_into()
        .expect("invariant: u32 argument must be 4 bytes");
    format::format_u32(u32::from_le_bytes(bytes), spec, buf);
}

fn fmt_i32(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    let bytes: [u8; 4] = data
        .try_into()
        .expect("invariant: i32 argument must be 4 bytes");
    format::format_i32(i32::from_le_bytes(bytes), spec, buf);
}

fn fmt_f32(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    let bytes: [u8; 4] = data
        .try_into()
        .expect("invariant: f32 argument must be 4 bytes");
    format::format_f32(f32::from_le_bytes(bytes), spec, buf);
}

fn fmt_u16(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    let bytes: [u8; 2] = data
        .try_into()
        .expect("invariant: u16 argument must be 2 bytes");
    format::format_u16(u16::from_le_bytes(bytes), spec, buf);
}

fn fmt_i16(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    let bytes: [u8; 2] = data
        .try_into()
        .expect("invariant: i16 argument must be 2 bytes");
    format::format_i16(i16::from_le_bytes(bytes), spec, buf);
}

fn fmt_u8(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    format::format_u8(data[0], spec, buf);
}

fn fmt_i8(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    format::format_i8(data[0] as i8, spec, buf);
}

fn fmt_bool(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    format::format_bool(data[0] != 0, spec, buf);
}

fn fmt_str(data: &[u8], spec: &FormatSpec, buf: &mut Vec<u8>) {
    // `from_utf8_lossy` borrows when the bytes are valid UTF-8 (the normal
    // case) and only allocates to substitute replacement chars if a corrupted
    // ring ever yields invalid bytes. No unsafe, no UB.
    let s = String::from_utf8_lossy(data);
    format::format_str(&s, spec, buf);
}

/// The drain thread state: the sink, the shutdown signal, and the drain's
/// private list of ring buffers.
pub(crate) struct Drain {
    sink: Box<dyn LogSink>,
    timezone_offset: i32,
    shutdown: Arc<AtomicBool>,
    /// `(ring, registration serial)`. The serial lets the drain distinguish a
    /// recycled pool segment's incarnations so it never drains a reincarnated
    /// ring through a stale list entry.
    rings: Vec<(Arc<RingBuffer>, u64)>,
    calibration: Calibration,
    line_pattern: Template,
    /// True when a segmented backpressure policy is active: the drain then
    /// syncs the registry every pass, waits on helper-reserved segments, and
    /// flushes the shared producer-helper line queue once per pass.
    segmented: bool,
}

impl Drain {
    /// Creates a drain. The `shutdown` flag must be the same one the owning
    /// [`Guard`](crate::Guard) sets on drop; `calibration` is the counter-to-
    /// wall-clock mapping sampled once at startup.
    pub(crate) fn new(
        sink: Box<dyn LogSink>,
        timezone_offset: i32,
        shutdown: Arc<AtomicBool>,
        calibration: Calibration,
        line_pattern: Template,
        segmented: bool,
    ) -> Self {
        Self {
            sink,
            timezone_offset,
            shutdown,
            rings: Vec::new(),
            calibration,
            line_pattern,
            segmented,
        }
    }

    /// Runs the poll loop until shutdown is signaled, then does one final pass
    /// so records written just before shutdown are not lost.
    pub(crate) fn run(&mut self) {
        // Reused across every record; cleared per record. No per-record alloc.
        // `staging` persists partial FIFO-backend records between polls; the
        // custom ring backend never touches it.
        let mut staging = Vec::with_capacity(4096);
        // Reused across every record; cleared per record. No per-record alloc.
        let mut buf = Vec::with_capacity(4096);
        // Idle backoff counter. The idle path reads no clock and takes no lock.
        let mut spin = SPIN_MIN;
        // Free-running ring-sync cadence counter; wraps at u32::MAX.
        let mut since_sync: u32 = 0;

        // Set when a poll emits records and cleared by the flush that follows
        // the drain going idle, so a buffered sink is flushed exactly once per
        // busy-then-idle transition rather than on every idle iteration.
        let mut dirty = false;

        loop {
            if self.shutdown.load(Ordering::Acquire) {
                break;
            }

            // Re-scan the ring registry on a coarse cadence in the classic path;
            // under a segmented policy registrations churn on every handoff, so
            // the registry is re-synced every pass instead. Both values are
            // powers of two so the free-running counter stays aligned across
            // its u32 wrap.
            let sync_every: u32 = if self.segmented { 1 } else { SYNC_EVERY };
            let sync_mask: u32 = sync_every - 1;

            // sync_rings() locks the global registry mutex; since_sync free-runs
            // and wraps, and the power-of-two mask keeps the alignment across
            // the wrap. A ring registered between syncs keeps its records in its
            // own buffer until the next sync.
            if (since_sync & sync_mask) == 0 {
                self.sync_rings(&mut staging, &mut buf);
            }
            since_sync = since_sync.wrapping_add(1);

            if self.poll_once(&mut staging, &mut buf) {
                spin = SPIN_MIN; // work found: reset backoff and re-poll now
                dirty = true;
                continue;
            }

            // Just caught up: flush the buffered sink once so records emitted in
            // the burst above become visible without a syscall per record.
            if dirty {
                self.flush_sink();
                dirty = false;
            }

            // Idle: back off with a capped, doubling spin. No clock, no lock.
            for _ in 0..spin {
                std::hint::spin_loop();
            }
            spin = (spin << 1).min(SPIN_CAP); // no overflow: capped
        }

        // Final pass: pick up any last records, including from rings whose
        // producers exited after the previous sync, then flush so nothing is
        // left buffered when the drain exits.
        self.sync_rings(&mut staging, &mut buf);
        self.poll_once(&mut staging, &mut buf);
        self.flush_sink();
    }

    /// Flushes the sink, logging any error to stderr. Called when the drain
    /// goes idle and once more at shutdown.
    fn flush_sink(&mut self) {
        if let Err(e) = self.sink.flush() {
            eprintln!("ticklog: sink flush failed: {}", e);
        }
    }

    /// Syncs the drain's local ring list with the global registry: adds newly
    /// registered rings, updates re-registered (recycled) segments, drops
    /// stale entries, and gives each ring whose producer has exited one final
    /// drain before draining it from the pool. The registry lock is released
    /// before that drain, so sink I/O never runs under the lock.
    fn sync_rings(&mut self, staging: &mut Vec<u8>, buf: &mut Vec<u8>) {
        let mut registry = REGISTRY
            .get()
            .expect("invariant: ring registry must be initialized by configure! before the drain starts")
            .lock()
            .expect("invariant: ring registry mutex poisoned by a panic in another thread");

        // Rings that stay alive for at least this sync cycle: registrations
        // that are live, or reserved (a blocked producer formatting its own
        // segment, or a producer whose Drop clears the reservation any moment).
        #[cfg(not(feature = "fifo-backend"))]
        let keep = |r: &Arc<RingBuffer>| r.live.load(Ordering::Acquire) || r.helper_reserved();
        #[cfg(feature = "fifo-backend")]
        let keep = |r: &Arc<RingBuffer>| r.live.load(Ordering::Acquire);

        // Rings that are closing down now (dead and unreserved): drained and
        // recycled outside the lock below. A reserved ring is never finalized
        // here: its producer clears the reservation (ThreadBuf::drop), and the
        // next sync gives it the final drain.
        let mut finalize: Vec<Arc<RingBuffer>> = registry
            .iter()
            .filter(|r| !keep(r))
            .map(Arc::clone)
            .collect();
        // A dead ring that was never added to the local list (its producer
        // registered and exited between two syncs) is not in `registry`'s
        // retain set below and can only be drained here, sourced from the
        // registry itself; entries in `self.rings` that are dead AND still in
        // the registry were just added above, so nothing is double-drained.
        for (ring, _) in &self.rings {
            if !keep(ring) && !registry.iter().any(|r| Arc::ptr_eq(r, ring)) {
                finalize.push(Arc::clone(ring));
            }
        }

        // Reconcile local entries in place: keep the position for rings that
        // are still registered, refresh the serial of re-registered
        // incarnations, and drop the rest (stale or finalized).
        let mut i = 0;
        while i < self.rings.len() {
            let (ring, _serial) = &self.rings[i];
            let in_registry = registry.iter().any(|r| Arc::ptr_eq(r, ring));
            if in_registry && keep(ring) {
                // Refresh the incarnation, keeping the entry's position.
                #[cfg(not(feature = "fifo-backend"))]
                if let Some(r) = registry.iter().find(|r| Arc::ptr_eq(r, ring)) {
                    let live_serial = r.serial();
                    if live_serial != *_serial {
                        self.rings[i] = (Arc::clone(r), live_serial);
                    }
                }
                i += 1;
            } else {
                self.rings.swap_remove(i);
            }
        }

        // Add newly registered rings at the tail, in registry order.
        for ring in registry.iter() {
            if keep(ring) && !self.rings.iter().any(|(r, _)| Arc::ptr_eq(r, ring)) {
                #[cfg(not(feature = "fifo-backend"))]
                self.rings.push((Arc::clone(ring), ring.serial()));
                #[cfg(feature = "fifo-backend")]
                self.rings.push((Arc::clone(ring), 0));
            }
        }

        // Forget dead rings from the shared registry. Acquire pairs with the
        // producer's Release store of `live`.
        registry.retain(&keep);
        // Release the lock before the final drain: it performs sink I/O, which
        // must never run under the registry mutex.
        drop(registry);

        // Give each dead ring one last drain before dropping it locally or
        // returning it to the pool. A single liveness check per ring decided
        // both drain and removal, so a ring cannot slip from "kept" to
        // "dropped" between the two. Once `live == false` is observed
        // (Acquire, pairing with the producer's Release store), the producer
        // is gone and its head is final, so this drain captures every record it
        // published before exiting. That is the guarantee the live flag exists
        // to provide.
        for ring in finalize {
            staging.clear();
            drain_ring(
                &ring,
                staging,
                self.sink.as_mut(),
                self.timezone_offset,
                &self.calibration,
                &self.line_pattern,
                buf,
            );
            #[cfg(not(feature = "fifo-backend"))]
            if crate::segments::Segments::installed() {
                crate::segments::Segments::get().drain_recycle(ring);
            }
        }
    }

    /// Drains every ring once, emitting all records published since the last
    /// pass, and flushes the producer-helper line queue (segmented policies).
    /// Returns `true` if any record was processed.
    fn poll_once(&mut self, staging: &mut Vec<u8>, buf: &mut Vec<u8>) -> bool {
        let mut had_work = false;
        let mut i = 0;
        while i < self.rings.len() {
            let (ring, _serial) = {
                let (r, s) = &self.rings[i];
                (Arc::clone(r), *s)
            };

            #[cfg(not(feature = "fifo-backend"))]
            {
                if ring.serial() != _serial {
                    // Reincarnated or stale: the next sync reconciles the entry.
                    i += 1;
                    continue;
                }

                if ring.helper_reserved() {
                    // A pool-exhausted producer is formatting this segment itself
                    // and will deliver the lines through the shared queue: skip it
                    // en route to the segments that need the drain's help. The
                    // reservation clears when the producer finishes (progress is
                    // guaranteed: formatting one segment frees exactly the spare it
                    // needs), so the entry is never skipped forever.
                    i += 1;
                    continue;
                }
            }

            staging.clear();
            if drain_ring(
                &ring,
                staging,
                self.sink.as_mut(),
                self.timezone_offset,
                &self.calibration,
                &self.line_pattern,
                buf,
            ) {
                had_work = true;
            }

            // A handed-off, fully drained segment is retired: unregistered
            // from the global registry and (for the bounded pool) returned as
            // a spare. Handed-off Quill segments are unregistered so their
            // control objects don't accumulate; the arena bytes are unbounded
            // by design.
            #[cfg(not(feature = "fifo-backend"))]
            if self.maybe_retire(i, &ring, _serial) {
                // Entry was swapped out; re-examine the entry now at `i`.
            } else {
                i += 1;
            }
            #[cfg(feature = "fifo-backend")]
            {
                i += 1;
            }
        }

        // Segmented helpers produced formatted lines for the drain to write:
        // the sink stays a single writer. This flush happens once per pass,
        // after every ring, so line order from a single helper is preserved.
        #[cfg(not(feature = "fifo-backend"))]
        if self.segmented && crate::segments::Segments::installed() {
            let collected = crate::segments::Segments::get().take_lines();
            for (line, level) in collected {
                if let Err(e) = self.sink.accept(&line, level) {
                    eprintln!("ticklog: sink accept failed: {}", e);
                }
                had_work = true;
            }
        }

        had_work
    }

    /// Retires a handed-off segment whose records the drain just consumed:
    /// unregisters it (so a recycled pool segment is re-registered at the
    /// tail on its next use, keeping per-thread order), and returns it to the
    /// bounded pool if it is pool-backed. Returns `true` if the drain's local
    /// entry was removed (callers must not advance past the vacated slot).
    #[cfg(not(feature = "fifo-backend"))]
    fn maybe_retire(&mut self, idx: usize, ring: &Arc<RingBuffer>, entry_serial: u64) -> bool {
        if !ring.handed_off() || ring.helper_reserved() || ring.serial() != entry_serial {
            return false;
        }
        // Only fully drained segments are retired; otherwise let the next
        // pass consume the remainder.
        if ring.head().load(Ordering::Acquire) != ring.tail().load(Ordering::Acquire) {
            return false;
        }

        let mut registry = REGISTRY
            .get()
            .expect("invariant: ring registry must be initialized")
            .lock()
            .expect("invariant: ring registry mutex poisoned by a panic in another thread");
        // Re-verify the incarnation under the lock: a producer may have
        // re-registered (serial changed) between the check above and now.
        if ring.serial() != entry_serial {
            return false;
        }
        let present = registry.iter().any(|r| Arc::ptr_eq(r, ring));
        if !present {
            // Already retired (e.g. the producer's helper formatted, recycled
            // and unregistered it itself): nothing to do.
            return false;
        }
        registry.retain(|r| !Arc::ptr_eq(r, ring));
        drop(registry);

        self.rings.swap_remove(idx);
        #[cfg(not(feature = "fifo-backend"))]
        if crate::segments::Segments::installed() {
            crate::segments::Segments::get().drain_recycle(Arc::clone(ring));
        }
        true
    }
}

/// Drains one custom-backend ring: emits every record in `[tail, head)` to
/// `sink` and publishes the advanced tail. Returns `true` if any record range
/// was processed.
///
/// Takes the drain's formatting inputs explicitly rather than `&self` so it can
/// run both from the poll loop (over live rings) and from `sync_rings` (a final
/// drain of a dead ring before it is dropped). `staging` is unused: the custom
/// ring stores slot-aligned records in its own memory.
///
/// Sets the ring's `draining` flag for its duration: a pool-exhausted producer
/// waiting to format this same segment as a helper spins on that flag, so the
/// drain's read never races another formatter.
#[cfg(not(feature = "fifo-backend"))]
fn drain_ring(
    ring: &RingBuffer,
    _staging: &mut Vec<u8>,
    sink: &mut dyn LogSink,
    timezone_offset: i32,
    calibration: &Calibration,
    line_pattern: &Template,
    buf: &mut Vec<u8>,
) -> bool {
    ring.mark_draining();
    let had_work = drain_ring_inner(
        ring,
        Some(sink),
        None,
        timezone_offset,
        calibration,
        line_pattern,
        buf,
    );
    ring.clear_draining();
    had_work
}

/// Formats every record in `[tail, head)` of a handed-off segment into owned
/// lines, without advancing the tail (the drain does that, and only for rings
/// it drains itself) and without consulting any sink. Used by the segmented
/// helper path, where a pool-exhausted producer formats its own full segment
/// to keep the blocking time bounded.
///
/// The producer must have stopped appending to `ring` (handed off). It first
/// waits for the drain to clear `draining`: the drain may be mid-drain on this
/// very segment, and the two formatters must not both read the same range as
/// their own delivery.
#[cfg(not(feature = "fifo-backend"))]
pub(crate) fn format_segment(
    ring: &RingBuffer,
    timezone_offset: i32,
    calibration: &Calibration,
    line_pattern: &Template,
) -> Vec<(Vec<u8>, Level)> {
    while ring.draining() {
        std::hint::spin_loop();
        std::thread::yield_now();
    }
    let mut lines: Vec<(Vec<u8>, Level)> = Vec::new();
    let mut buf = Vec::new();
    drain_ring_inner(
        ring,
        None,
        Some(&mut lines),
        timezone_offset,
        calibration,
        line_pattern,
        &mut buf,
    );
    lines
}

/// Shared record-decoding core for the drain thread and the producer-side
/// helper. Formats every record in `[tail, head)` of a custom ring, dispatching
/// each formatted line either to `sink` or, when formatting on the producer
/// side, into `lines` as an owned buffer. Publishes the advanced tail in all
/// cases: the tail defines what the drain has consumed, and the helper
/// precedes its recycle of the segment with this same store.
#[cfg(not(feature = "fifo-backend"))]
fn drain_ring_inner(
    ring: &RingBuffer,
    mut sink: Option<&mut dyn LogSink>,
    mut lines: Option<&mut Vec<(Vec<u8>, Level)>>,
    timezone_offset: i32,
    calibration: &Calibration,
    line_pattern: &Template,
    buf: &mut Vec<u8>,
) -> bool {
    #[cfg(feature = "watermark-head")]
    {
        // The producer may have exited with records unpublished past the last
        // `head` watermark. `live = false` is stored with Release after the
        // producer's final publish, so the Acquire load below orders that
        // store (making the producer-private position safe to read) and this
        // flush publishes the tail so no record is lost.
        if !ring.live.load(Ordering::Acquire) {
            ring.flush_watermark();
        }
    }
    // Thread identity lives on the ring (registered once per incarnation),
    // never in the records. Snapshot it once per pass — a cheap atomic load
    // plus one small-string clone — so formatting needs no per-record lock.
    let ring_thread_id = ring.thread_id();
    let ring_thread_name = ring.thread_name();
    let capacity = ring.capacity();
    let mask = (capacity - 1) as u64;
    // Own index: Relaxed load; the drain is the sole writer of `tail`.
    let mut tail = ring.tail().load(Ordering::Relaxed);
    // SAFETY: head_cache is drain-private; no producer touches it and the
    // drain is single-threaded, so this raw access is unaliased.
    let mut head_cache = unsafe { *ring.head_cache() };

    // Cached-index fast path: pay the cross-core Acquire load of `head` only
    // when the cache says we have caught up.
    if tail == head_cache {
        head_cache = ring.head().load(Ordering::Acquire);
        if tail == head_cache {
            return false; // no new records in this ring
        }
    }

    // The drain reaches the buffer through a base pointer taken from the shared
    // `data` cells, never a slice reference that would race the producer's
    // write. It reads only [tail, head_cache), a region the producer published
    // via its Release store of `head` and will not overwrite while tail lags.
    let base = ring.data_ptr() as *const u8;

    let mut had_work = false;

    while tail < head_cache {
        let offset = (tail & mask) as usize;

        // SAFETY: `offset` is within the ring; the prefix cursor is bounded to
        // the bytes from `offset` to the end of the ring storage.
        let (version, rectype, total_size) = unsafe {
            let mut c = Cursor::new(base.add(offset), capacity - offset);
            let version = c.read_u8();
            let rectype = c.read_u8();
            let total_size = c.read_u16() as u64;
            (version, rectype, total_size)
        };

        if total_size == 0 {
            break; // empty slot: nothing more published in this ring
        }

        // Layer A: reject a corrupt frame before trusting `total_size` to slice
        // or advance the tail. On a cooperative single-process channel this
        // never fires; if it does, resync to the producer's published head and
        // resume rather than walk off into garbage.
        if version != VERSION
            || (rectype != LOG_RECORD && rectype != END_OF_BUFFER)
            || total_size < HEADER_SIZE as u64
            || offset + total_size as usize > capacity
        {
            eprintln!(
                "ticklog: drain discarded a corrupt record \
                 (version={version}, type={rectype}, size={total_size}); resyncing to head"
            );
            tail = head_cache;
            break;
        }

        if rectype == END_OF_BUFFER {
            tail += align_up(total_size, SLOT_SIZE as u64);
            continue; // `tail & mask` wraps to 0 at the ring boundary
        }

        // SAFETY: `total_size` frames one complete record inside the ring
        // (validated above), so this slice is initialized and in-bounds.
        let record = unsafe { std::slice::from_raw_parts(base.add(offset), total_size as usize) };
        let level = record
            .get(4)
            .and_then(|&b| Level::from_u8(b))
            .unwrap_or(Level::Error);

        buf.clear();
        decode_and_format(
            record,
            timezone_offset,
            calibration,
            line_pattern,
            ring_thread_id,
            &ring_thread_name,
            buf,
        );
        match sink.as_mut() {
            Some(s) => {
                if let Err(e) = s.accept(buf, level) {
                    eprintln!("ticklog: sink accept failed: {}", e);
                }
            }
            None => {
                // SAFETY: `lines` is always Some when `sink` is None (the only
                // caller is format_segment, which supplies both). Moving the
                // line out keeps the caller's staging buffer reusable.
                lines
                    .as_mut()
                    .expect("invariant: lines required when formatting without a sink")
                    .push((std::mem::take(buf), level));
            }
        }
        had_work = true;

        tail += align_up(total_size, SLOT_SIZE as u64);
    }

    // Publish the consumed position. Release pairs with the producer's Acquire
    // load of `tail` in its capacity check, so the producer never overwrites a
    // slot the drain has not finished reading.
    ring.tail().store(tail, Ordering::Release);
    // SAFETY: head_cache is drain-private, as above.
    unsafe {
        *ring.head_cache() = head_cache;
    }

    had_work
}

/// Drains one FIFO-backend ring: pops every byte currently buffered into
/// `staging`, formats the complete records it holds, and keeps the incomplete
/// tail for the next poll. Returns `true` if any record was processed.
///
/// The drain holds no lock while decoding, so the producer never blocks on
/// sink I/O: `pop_available` releases the FIFO mutex before formatting.
#[cfg(feature = "fifo-backend")]
fn drain_ring(
    ring: &RingBuffer,
    staging: &mut Vec<u8>,
    sink: &mut dyn LogSink,
    timezone_offset: i32,
    calibration: &Calibration,
    line_pattern: &Template,
    buf: &mut Vec<u8>,
) -> bool {
    ring.pop_available(staging);
    let mut had_work = false;
    let mut pos = 0usize;
    while staging.len() - pos >= HEADER_SIZE {
        let version = staging[pos];
        let rectype = staging[pos + 1];
        let total_size = u16::from_le_bytes([staging[pos + 2], staging[pos + 3]]) as usize;

        if rectype == END_OF_BUFFER {
            // Never produced by the FIFO backend; tolerated. Keep `pos` bounded
            // so a corrupt size cannot drive `pos` past `staging.len()` and
            // panic the drain on the next `staging[pos]` read.
            if pos + total_size > staging.len() {
                break;
            }
            pos += total_size;
            continue;
        }
        if version != VERSION || rectype != LOG_RECORD || total_size < HEADER_SIZE {
            eprintln!("ticklog: drain discarded a corrupt record; resyncing the FIFO cursor");
            pos += 1;
            continue;
        }
        if pos + total_size > staging.len() {
            break; // record still streaming in; wait for the rest
        }

        let record = &staging[pos..pos + total_size];
        let level = record
            .get(4)
            .and_then(|&b| Level::from_u8(b))
            .unwrap_or(Level::Error);

        buf.clear();
        decode_and_format(
            record,
            timezone_offset,
            calibration,
            line_pattern,
            ring.thread_id(),
            &ring.thread_name(),
            buf,
        );
        if let Err(e) = sink.accept(buf, level) {
            eprintln!("ticklog: sink accept failed: {}", e);
        }
        pos += total_size;
        had_work = true;
    }
    if pos > 0 {
        staging.drain(..pos);
    }
    had_work
}

/// Decodes one encoded record and appends a formatted line to `buf`.
///
/// The line carries no trailing newline: per the [`LogSink`] contract the sink
/// terminates lines itself (the built-in sinks append `\n`).
///
/// The caller has validated that `record` spans `total_size` bytes of a
/// `LOG_RECORD`. All formatting into `Vec<u8>` is infallible; unknown argument
/// tags produce a placeholder rather than panicking.
fn decode_and_format(
    record: &[u8],
    timezone_offset: i32,
    calibration: &Calibration,
    line_pattern: &Template,
    ring_thread_id: u64,
    ring_thread_name: &str,
    buf: &mut Vec<u8>,
) {
    // SAFETY: `record.as_ptr()` starts a validated record slice of length
    // total_size (>= HEADER_SIZE for a LOG_RECORD); every read below advances
    // within that slice for a well-formed record.
    let mut c = unsafe { Cursor::new(record.as_ptr(), record.len()) };

    // Step 1: header. Only level, flags, and timestamp are needed here; the
    // version/type/total_size prefix was consumed by the poll loop.
    // SAFETY: the 16-byte header is present for every LOG_RECORD.
    let (level_byte, flags, timestamp) = unsafe {
        c.skip(4); // version, type, total_size
        let level_byte = c.read_u8();
        let flags = c.read_u16();
        c.skip(1); // _pad
        let timestamp = c.read_u64();
        (level_byte, flags, timestamp)
    };
    let level = Level::from_u8(level_byte).unwrap_or(Level::Error);

    // Step 2: decode the timestamp from raw ticks.
    let ns = ticks_to_ns(timestamp, calibration);

    // Step 3: flagged sections, in fixed order.
    let mut fmt: &str = "";
    let mut file_line: Option<(&str, u32)> = None;
    if flags & FLAG_SITE != 0 {
        // SAFETY: the site section is an 8-byte pointer to a promoted
        // `&'static Site` descriptor, present when FLAG_SITE is set.
        // The logging macro constructs the descriptor inline, constant
        // promotion freezes it in read-only data, and the pointer it writes is
        // that frozen address (valid for the whole process), so dereferencing
        // here is sound regardless of which thread produced the record.
        let site_ptr = unsafe { c.read_u64() as *const crate::record::Site };
        // SAFETY: `site_ptr` points at a valid `Site` (see above: the pointer
        // was created from `&Site` by the macro, valid for `'static`).
        let site = unsafe { &*site_ptr };
        fmt = site.fmt;
        file_line = Some((site.file, site.line));
    }
    // Read source section (legacy wire) and thread section (legacy wire),
    // deferring rendering until all identity is known.
    if flags & FLAG_SOURCE != 0 {
        // SAFETY: the source section is 14 bytes (u64 ptr + u16 len + u32 line)
        // present when FLAG_SOURCE is set.
        let (file_ptr, file_len, line) = unsafe {
            let p = std::ptr::with_exposed_provenance::<u8>(c.read_u64() as usize);
            let l = c.read_u16() as usize;
            let ln = c.read_u32();
            (p, l, ln)
        };
        // SAFETY: file_ptr/file_len come from a &'static str (file!()) in
        // read-only data, valid for the whole process.
        let file_bytes = unsafe { std::slice::from_raw_parts(file_ptr, file_len) };
        let file = std::str::from_utf8(file_bytes).unwrap_or("<file>");
        file_line = Some((file, line));
    }
    // Thread identity defaults to the ring's registration (the fast path
    // writes no thread bytes); a legacy wire record may override it with an
    // in-band thread section. The override is owned so its borrow of `c`
    // ends here (the legacy path is cold; the fast path never allocates).
    let mut thread_id: u64 = ring_thread_id;
    let mut thread_name_override: Option<String> = None;
    if flags & FLAG_THREAD != 0 {
        // SAFETY: the thread section is an 8-byte id followed by a
        // length-prefixed name, all within the record slice.
        unsafe {
            thread_id = c.read_u64();
            thread_name_override = {
                let (name_bytes, name_len) = c.read_len_prefixed();
                if name_len > 0 {
                    std::str::from_utf8(name_bytes).ok().map(String::from)
                } else {
                    None
                }
            };
        }
    }
    // Rendering borrows the chosen name (the local override or the ring's);
    // neither borrows `c`, so `&mut c` below is unaliased.
    let thread_name: Option<&str> = thread_name_override.as_deref().or(Some(ring_thread_name));
    if flags & FLAG_PROCESS != 0 {
        // SAFETY: the process section is a 4-byte pid.
        unsafe {
            c.skip(4);
        }
    }
    if flags & FLAG_COMPLEX != 0 {
        // SAFETY: the complex section starts with an 8-byte pointer.
        unsafe {
            c.skip(8);
        }
    }

    // Step 4: read n_args and tags (argument types). The cursor is now
    // positioned at the start of argument payloads for interleave.
    // SAFETY: n_args (u8) is followed by exactly n_args tag bytes.
    let n_args = unsafe { c.read_u8() } as usize;
    let mut tag_buf = [0u8; 256];
    // SAFETY: c is bounded to the validated record slice and read_bytes clamps
    // its length to the bytes remaining, so the read stays in-bounds.
    let n_tags = unsafe {
        let tags = c.read_bytes(n_args);
        tag_buf[..tags.len()].copy_from_slice(tags);
        tags.len()
    };

    // Step 5: render the line from the pattern.
    render_pattern(
        line_pattern,
        ns,
        timezone_offset,
        level,
        fmt,
        file_line,
        thread_id,
        thread_name.as_deref(),
        &tag_buf[..n_tags],
        &mut c,
        buf,
    );
}

/// Renders one log line by walking the pattern [`Template`] and dispatching
/// each [`Segment::Place`] by its field name.
#[allow(clippy::too_many_arguments)]
fn render_pattern(
    template: &Template,
    ns: u64,
    timezone_offset: i32,
    level: Level,
    fmt: &str,
    file_line: Option<(&str, u32)>,
    thread_id: u64,
    thread_name: Option<&str>,
    tags: &[u8],
    c: &mut Cursor,
    buf: &mut Vec<u8>,
) {
    for seg in &template.segments {
        match seg {
            Segment::Lit(s) => buf.extend_from_slice(s.as_bytes()),
            Segment::Place { field, spec } => match field {
                Field::Timestamp => {
                    format_iso8601(ns, timezone_offset, buf);
                }
                Field::Level => {
                    format::format_str(level.as_str(), spec, buf);
                }
                Field::File => {
                    if let Some((file, _)) = file_line {
                        format::format_str(file, spec, buf);
                    }
                }
                Field::Line => {
                    if let Some((_, line)) = file_line {
                        format::format_u32(line, spec, buf);
                    }
                }
                Field::ThreadName => {
                    if let Some(name) = thread_name {
                        format::format_str(name, spec, buf);
                    }
                }
                Field::ThreadId => {
                    buf.extend_from_slice(b"ThreadId(");
                    format::format_u64(thread_id, spec, buf);
                    buf.push(b')');
                }
                Field::Message => {
                    interleave(fmt, tags, c, buf);
                }
            },
        }
    }
}

/// Walks the format string, substituting each `{...}` placeholder with the
/// next argument. Literal text and `{{`/`}}` escapes are copied through.
fn interleave(fmt: &str, tags: &[u8], c: &mut Cursor, buf: &mut Vec<u8>) {
    let bytes = fmt.as_bytes();
    let mut i = 0;
    let mut arg_idx = 0;
    // Cleared on the first unknown tag: once the cursor position is unknown,
    // later arguments cannot be read, only reported.
    let mut parse_ok = true;

    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'{' {
                    buf.push(b'{');
                    i += 2;
                    continue;
                }
                // Find the closing brace. The format string passed compile-time
                // validation, so a match is guaranteed for well-formed input.
                let start = i + 1;
                let mut j = start;
                while j < bytes.len() && bytes[j] != b'}' {
                    j += 1;
                }
                let spec = format::parse_spec(&fmt[start..j]);
                i = if j < bytes.len() { j + 1 } else { j };

                match tags.get(arg_idx).copied() {
                    Some(tag) if parse_ok => {
                        if !format_arg(tag, &spec, c, buf) {
                            parse_ok = false;
                            write_unknown_tag(tag, buf);
                        }
                    }
                    Some(tag) => write_unknown_tag(tag, buf),
                    None => buf.extend_from_slice(b"<missing arg>"),
                }
                arg_idx += 1;
            }
            b'}' => {
                // The only valid `}` in a compile-time-validated format string
                // is the `}}` escape; a placeholder's closing brace was already
                // consumed by the `{` arm above. A lone `}` is rejected by the
                // macro (as in std's `format!`), so reaching one here is a bug.
                debug_assert!(
                    i + 1 < bytes.len() && bytes[i + 1] == b'}',
                    "invariant: lone '}}' in validated format string"
                );
                buf.push(b'}');
                i += 2;
            }
            other => {
                buf.push(other);
                i += 1;
            }
        }
    }
}

/// Reads and formats one argument of type `tag` from the cursor. Returns
/// `false` for an unknown tag, in which case the cursor is left untouched.
fn format_arg(tag: u8, spec: &FormatSpec, c: &mut Cursor, buf: &mut Vec<u8>) -> bool {
    if tag == TAG_STR {
        // SAFETY: a string argument is a u16 length prefix followed by that
        // many UTF-8 bytes, exactly what read_len_prefixed consumes.
        let (data, _) = unsafe { c.read_len_prefixed() };
        fmt_str(data, spec, buf);
        return true;
    }

    let idx = tag as usize;
    if idx < FIXED_SIZES.len() {
        let size = FIXED_SIZES[idx];
        // SAFETY: `size` is the fixed encoded width for this tag; the producer
        // wrote exactly that many argument bytes here.
        let data = unsafe { c.read_bytes(size) };
        FORMATTERS[idx](data, spec, buf);
        return true;
    }

    false
}

/// Appends `<unknown tag 0xNN>` to `buf`.
fn write_unknown_tag(tag: u8, buf: &mut Vec<u8>) {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    buf.extend_from_slice(b"<unknown tag 0x");
    buf.push(HEX[(tag >> 4) as usize]);
    buf.push(HEX[(tag & 0x0F) as usize]);
    buf.push(b'>');
}

#[cfg(all(test, not(feature = "fifo-backend")))]
mod tests {
    use super::*;
    use crate::builder::DEFAULT_LINE_PATTERN;
    use crate::encode::{TAG_BOOL, TAG_F64, TAG_I64, TAG_U16, TAG_U64};
    use crate::record::{HEADER_SIZE, LOG_RECORD, SITE_SECTION_SIZE, Site, VERSION};
    use std::io;
    use std::sync::{Mutex, OnceLock};

    /// Shared handle to the `(line, level)` pairs a [`CaptureSink`] recorded.
    type CaptureCalls = Arc<Mutex<Vec<(String, Level)>>>;

    // ---- test helpers -------------------------------------------------------

    /// Identity calibration: raw tick value equals nanoseconds since epoch.
    fn identity_calibration() -> Calibration {
        Calibration {
            counter_to_ns: 1.0,
            counter_base: 0,
            wall_base_ns: 0,
        }
    }

    /// Encodes a single argument's payload bytes for a fixed-size type.
    fn le_bytes(tag: u8, value: u64) -> (u8, Vec<u8>) {
        let bytes = match tag {
            TAG_U64 | TAG_I64 | TAG_F64 => value.to_le_bytes().to_vec(),
            TAG_U16 => (value as u16).to_le_bytes().to_vec(),
            TAG_BOOL => vec![value as u8],
            _ => panic!("unsupported tag in helper"),
        };
        (tag, bytes)
    }

    /// Encodes a string argument payload: [len u16 LE][utf8 bytes].
    fn str_arg(s: &str) -> (u8, Vec<u8>) {
        let mut v = (s.len() as u16).to_le_bytes().to_vec();
        v.extend_from_slice(s.as_bytes());
        (TAG_STR, v)
    }

    /// A promoted call-site descriptor for the wire: leak a `Site` so the
    /// test record's pointer stays valid for the whole process (the real
    /// producer relies on constant promotion; a leak is the test-only
    /// equivalent, and the boxes are tiny).
    fn site_of(fmt: &'static str, file: &'static str, line: u32) -> &'static Site {
        Box::leak(Box::new(Site { fmt, file, line }))
    }

    /// Builds a full encoded LOG_RECORD in the fast-path layout: a header
    /// plus a `FLAG_SITE` pointer to a promoted call-site descriptor, then
    /// count, tags, and payloads. This is the wire format the producer's
    /// logging macros emit today.
    fn build_record(
        level: Level,
        timestamp: u64,
        site: &'static Site,
        args: &[(u8, Vec<u8>)],
    ) -> Vec<u8> {
        let flags: u16 = FLAG_SITE;

        let mut payload: Vec<u8> = Vec::new();

        // Site section: one pointer to the frozen call-site descriptor.
        payload.extend_from_slice(&(site as *const Site as u64).to_le_bytes());

        // Arguments.
        payload.push(args.len() as u8);
        for (tag, _) in args {
            payload.push(*tag);
        }
        for (_, data) in args {
            payload.extend_from_slice(data);
        }

        let total_size = (HEADER_SIZE + payload.len()) as u16;

        let mut record = Vec::with_capacity(total_size as usize);
        record.push(VERSION); // version
        record.push(LOG_RECORD); // type
        record.extend_from_slice(&total_size.to_le_bytes());
        record.push(level.to_u8());
        record.extend_from_slice(&flags.to_le_bytes());
        record.push(0); // _pad
        record.extend_from_slice(&timestamp.to_le_bytes());
        record.extend_from_slice(&payload);
        record
    }

    /// Builds a LOG_RECORD with the fast-path SITE section plus legacy
    /// FLAG_SOURCE and FLAG_THREAD sections, proving the drain still tolerates
    /// sibling layouts that carry location/thread bytes in the record itself
    /// (their flag bits are unambiguous against the site layout). The source
    /// section must override the site's file/line, matching old semantics.
    fn build_record_with_legacy_extra(
        level: Level,
        timestamp: u64,
        site: &'static Site,
        source: Option<(&'static str, u32)>,
        thread_id: u64,
        thread_name: Option<&str>,
        args: &[(u8, Vec<u8>)],
    ) -> Vec<u8> {
        let mut flags: u16 = FLAG_SITE | FLAG_THREAD;
        if source.is_some() {
            flags |= FLAG_SOURCE;
        }

        let mut payload: Vec<u8> = Vec::new();

        // Site section (fast path).
        payload.extend_from_slice(&(site as *const Site as u64).to_le_bytes());

        // Legacy source section.
        if let Some((file, line)) = source {
            payload.extend_from_slice(&(file.as_ptr() as u64).to_le_bytes());
            payload.extend_from_slice(&(file.len() as u16).to_le_bytes());
            payload.extend_from_slice(&line.to_le_bytes());
        }

        // Legacy thread section.
        payload.extend_from_slice(&thread_id.to_le_bytes());
        let name_bytes = thread_name.map_or(&b""[..], |n| n.as_bytes());
        payload.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
        payload.extend_from_slice(name_bytes);

        // Arguments.
        payload.push(args.len() as u8);
        for (tag, _) in args {
            payload.push(*tag);
        }
        for (_, data) in args {
            payload.extend_from_slice(data);
        }

        let total_size = (HEADER_SIZE + payload.len()) as u16;

        let mut record = Vec::with_capacity(total_size as usize);
        record.push(VERSION); // version
        record.push(LOG_RECORD); // type
        record.extend_from_slice(&total_size.to_le_bytes());
        record.push(level.to_u8());
        record.extend_from_slice(&flags.to_le_bytes());
        record.push(0); // _pad
        record.extend_from_slice(&timestamp.to_le_bytes());
        record.extend_from_slice(&payload);
        record
    }

    fn default_line_pattern() -> Template {
        Template::parse(DEFAULT_LINE_PATTERN)
            .expect("invariant: default pattern is a valid format string")
    }

    fn format_line(record: &[u8]) -> String {
        let mut buf = Vec::new();
        decode_and_format(
            record,
            0,
            &identity_calibration(),
            &default_line_pattern(),
            7, // ring thread id
            "worker",
            &mut buf,
        );
        String::from_utf8(buf).unwrap()
    }

    /// A sink that captures every accepted `(line, level)` pair.
    struct CaptureSink {
        calls: CaptureCalls,
    }

    impl LogSink for CaptureSink {
        fn accept(&mut self, line: &[u8], level: Level) -> io::Result<()> {
            self.calls
                .lock()
                .unwrap()
                .push((String::from_utf8_lossy(line).into_owned(), level));
            Ok(())
        }
    }

    fn capture_drain() -> (Drain, CaptureCalls) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let sink = CaptureSink {
            calls: Arc::clone(&calls),
        };
        let drain = Drain::new(
            Box::new(sink),
            0,
            Arc::new(AtomicBool::new(false)),
            identity_calibration(),
            default_line_pattern(),
            false,
        );
        (drain, calls)
    }

    /// Writes `bytes` into a ring at `offset` and advances `head` by
    /// `align_up(len, SLOT_SIZE)` to mimic the producer.
    #[cfg(not(feature = "fifo-backend"))]
    fn place_record(ring: &RingBuffer, offset: u64, bytes: &[u8]) {
        // SAFETY: single-threaded test with exclusive access to the ring data.
        let data = unsafe { std::slice::from_raw_parts_mut(ring.data_ptr(), ring.capacity()) };
        let start = offset as usize;
        data[start..start + bytes.len()].copy_from_slice(bytes);
        let new_head = offset + align_up(bytes.len() as u64, SLOT_SIZE as u64);
        ring.head().store(new_head, Ordering::Release);
    }

    // ---- Cursor tests -------------------------------------------------------

    #[test]
    fn cursor_reads_types_and_advances() {
        let mut data = Vec::new();
        data.push(0xABu8);
        data.extend_from_slice(&0xBEEFu16.to_le_bytes());
        data.extend_from_slice(&0xDEAD_BEEFu32.to_le_bytes());
        data.extend_from_slice(&0x0102_0304_0506_0708u64.to_le_bytes());

        // SAFETY: the cursor reads exactly the 15 bytes just written.
        unsafe {
            let mut c = Cursor::new(data.as_ptr(), data.len());
            assert_eq!(c.read_u8(), 0xAB);
            assert_eq!(c.read_u16(), 0xBEEF);
            assert_eq!(c.read_u32(), 0xDEAD_BEEF);
            assert_eq!(c.read_u64(), 0x0102_0304_0506_0708);
        }
    }

    #[test]
    fn cursor_skip_and_read_bytes() {
        let data = [1u8, 2, 3, 4, 5, 6];
        // SAFETY: reads/skip stay within the 6-byte array.
        unsafe {
            let mut c = Cursor::new(data.as_ptr(), data.len());
            c.skip(2);
            assert_eq!(c.read_bytes(3), &[3, 4, 5]);
            assert_eq!(c.read_u8(), 6);
        }
    }

    #[test]
    fn cursor_read_len_prefixed() {
        let mut data = (5u16).to_le_bytes().to_vec();
        data.extend_from_slice(b"hello");
        data.push(0x99); // trailing byte after the string
        // SAFETY: the length prefix (5) matches the 5 string bytes present.
        unsafe {
            let mut c = Cursor::new(data.as_ptr(), data.len());
            let (s, len) = c.read_len_prefixed();
            assert_eq!(len, 5);
            assert_eq!(s, b"hello");
            assert_eq!(c.read_u8(), 0x99);
        }
    }

    #[test]
    fn cursor_read_len_prefixed_empty() {
        let data = (0u16).to_le_bytes().to_vec();
        // SAFETY: zero-length string; only the 2-byte prefix is read.
        unsafe {
            let mut c = Cursor::new(data.as_ptr(), data.len());
            let (s, len) = c.read_len_prefixed();
            assert_eq!(len, 0);
            assert!(s.is_empty());
        }
    }

    #[test]
    fn cursor_clamps_reads_at_end() {
        // A 3-byte slice whose reads are all attempted past its end.
        let data = [1u8, 2, 3];
        // SAFETY: the cursor is bounded to these 3 bytes; every read below is
        // clamped, so none touches memory past the slice.
        unsafe {
            let mut c = Cursor::new(data.as_ptr(), data.len());
            // A u64 needs 8 bytes but only 3 remain: yields 0, consumes nothing.
            assert_eq!(c.read_u64(), 0);
            assert_eq!(c.remaining(), 3);
            // read_bytes clamps to what is left.
            assert_eq!(c.read_bytes(100), &[1, 2, 3]);
            assert_eq!(c.remaining(), 0);
            // Exhausted: further fixed-width reads are 0.
            assert_eq!(c.read_u16(), 0);
        }
    }

    #[test]
    fn cursor_len_prefix_clamps_when_string_overruns() {
        // Prefix claims 9 bytes but only 3 follow: read_len_prefixed returns the
        // 3 present rather than reading past the slice.
        let mut data = (9u16).to_le_bytes().to_vec();
        data.extend_from_slice(b"abc");
        // SAFETY: bounded to the 5 bytes written; the over-long prefix is clamped.
        unsafe {
            let mut c = Cursor::new(data.as_ptr(), data.len());
            let (s, len) = c.read_len_prefixed();
            assert_eq!(len, 9);
            assert_eq!(s, b"abc");
        }
    }

    // ---- FORMATTERS tests ---------------------------------------------------

    #[test]
    fn formatter_u64_roundtrip() {
        let mut buf = Vec::new();
        fmt_u64(&42u64.to_le_bytes(), &FormatSpec::default(), &mut buf);
        assert_eq!(buf, b"42");
    }

    #[test]
    fn formatter_i64_negative() {
        let mut buf = Vec::new();
        fmt_i64(&(-7i64).to_le_bytes(), &FormatSpec::default(), &mut buf);
        assert_eq!(buf, b"-7");
    }

    #[test]
    fn formatter_f64_roundtrip() {
        let mut buf = Vec::new();
        fmt_f64(&3.5f64.to_le_bytes(), &FormatSpec::default(), &mut buf);
        assert_eq!(buf, b"3.5");
    }

    #[test]
    fn formatter_bool_and_str() {
        let mut buf = Vec::new();
        fmt_bool(&[1], &FormatSpec::default(), &mut buf);
        assert_eq!(buf, b"true");

        buf.clear();
        fmt_str(b"hi", &FormatSpec::default(), &mut buf);
        assert_eq!(buf, b"hi");
    }

    #[test]
    fn formatter_str_invalid_utf8_is_lossy() {
        let mut buf = Vec::new();
        // 0xFF is not valid UTF-8; lossy substitution must not panic.
        fmt_str(&[0xFF], &FormatSpec::default(), &mut buf);
        assert_eq!(buf, "\u{FFFD}".as_bytes());
    }

    // ---- decode_and_format tests --------------------------------------------

    #[test]
    fn decode_full_line() {
        let record = build_record(
            Level::Info,
            0,
            site_of("x={}", "a.rs", 7),
            &[le_bytes(TAG_U64, 42)],
        );
        let line = format_line(&record);
        assert_eq!(line, "1970-01-01T00:00:00.000000000Z INFO a.rs:7 x=42",);
    }

    #[test]
    fn decode_multiple_args_and_types() {
        let record = build_record(
            Level::Error,
            0,
            site_of("{} {} {}", "", 0),
            &[le_bytes(TAG_U16, 5), str_arg("ok"), le_bytes(TAG_BOOL, 1)],
        );
        let line = format_line(&record);
        assert_eq!(line, "1970-01-01T00:00:00.000000000Z ERROR :0 5 ok true",);
    }

    #[test]
    fn decode_escaped_braces() {
        let record = build_record(
            Level::Info,
            0,
            site_of("{{{}}}", "", 0),
            &[le_bytes(TAG_U64, 9)],
        );
        let line = format_line(&record);
        assert_eq!(line, "1970-01-01T00:00:00.000000000Z INFO :0 {9}",);
    }

    #[test]
    fn decode_unknown_tag_emits_placeholder() {
        // Tag 0x7F is not a known type; the drain must not panic.
        let record = build_record(Level::Info, 0, site_of("v={}", "", 0), &[(0x7F, vec![0u8])]);
        let line = format_line(&record);
        assert_eq!(
            line,
            "1970-01-01T00:00:00.000000000Z INFO :0 v=<unknown tag 0x7F>",
        );
    }

    #[test]
    fn decode_missing_arg_placeholder() {
        // Two placeholders but only one argument supplied.
        let record = build_record(
            Level::Info,
            0,
            site_of("{} {}", "", 0),
            &[le_bytes(TAG_U64, 1)],
        );
        let line = format_line(&record);
        assert_eq!(
            line,
            "1970-01-01T00:00:00.000000000Z INFO :0 1 <missing arg>",
        );
    }

    #[test]
    fn decode_tolerates_count_byte_exceeding_tags_present() {
        // Corruption: the count byte claims more args than the record actually
        // holds. The decoder must clamp to the tags present rather than panic on
        // a length-mismatched copy, and report the shortfall as a missing arg.
        let mut record = build_record(Level::Info, 0, site_of("v={}", "", 0), &[]);
        // The count byte follows the fixed header and the site section.
        record[HEADER_SIZE + SITE_SECTION_SIZE] = 200;
        let line = format_line(&record);
        assert!(
            line.ends_with("v=<missing arg>"),
            "expected a missing-arg placeholder, got {line:?}"
        );
    }

    #[test]
    fn decode_timestamp_conversion() {
        // With identity calibration, the raw tick is nanoseconds since epoch.
        let record = build_record(Level::Info, 1_234_567_890, site_of("t", "", 0), &[]);
        let line = format_line(&record);
        assert_eq!(line, "1970-01-01T00:00:01.234567890Z INFO :0 t",);
    }

    #[test]
    fn decodes_records_with_legacy_extra_sections() {
        // A site-layout record that also carries legacy source/thread bytes.
        // The source section overrides the site's file/line; the thread bytes
        // override the ring identity. Both must render even though the modern
        // producer never emits them.
        let record = build_record_with_legacy_extra(
            Level::Info,
            0,
            site_of("n={}", "site.rs", 9),
            Some(("old.rs", 3)),
            99,
            Some("legacy"),
            &[le_bytes(TAG_U64, 7)],
        );
        let mut buf = Vec::new();
        decode_and_format(
            &record,
            0,
            &identity_calibration(),
            &default_line_pattern(),
            1, // ring identity: different from the record's, must be overridden
            "ring",
            &mut buf,
        );
        let line = String::from_utf8(buf).unwrap();
        assert!(
            line.ends_with("INFO old.rs:3 n=7"),
            "source override / message mismatch: {line:?}"
        );
        // A pattern that renders the thread proves the in-band override is used.
        let mut buf = Vec::new();
        let template = Template::parse("{timestamp} {level} {thread_id} {thread_name}: {message}")
            .expect("valid template");
        decode_and_format(
            &record,
            0,
            &identity_calibration(),
            &template,
            1,
            "ring",
            &mut buf,
        );
        let line = String::from_utf8(buf).unwrap();
        assert!(
            line.contains("ThreadId(99)") && line.contains("legacy"),
            "in-band thread override not applied: {line:?}"
        );
    }

    // ---- poll loop tests ----------------------------------------------------

    #[test]
    fn poll_empty_ring_no_accepts() {
        let (mut drain, calls) = capture_drain();
        drain.rings.push((Arc::new(RingBuffer::new()), 0));
        let mut staging = Vec::new();
        let mut buf = Vec::new();
        assert!(!drain.poll_once(&mut staging, &mut buf));
        assert!(calls.lock().unwrap().is_empty());
    }

    #[test]
    #[cfg(not(feature = "fifo-backend"))]
    fn poll_one_record_accepts_and_advances_tail() {
        let (mut drain, calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());
        let record = build_record(
            Level::Info,
            0,
            site_of("x={}", "a.rs", 7),
            &[le_bytes(TAG_U64, 42)],
        );
        place_record(&ring, 0, &record);
        let head = ring.head().load(Ordering::Acquire);
        drain.rings.push((Arc::clone(&ring), ring.serial()));

        let mut staging = Vec::new();
        let mut buf = Vec::new();
        assert!(drain.poll_once(&mut staging, &mut buf));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(
            recorded[0].0,
            "1970-01-01T00:00:00.000000000Z INFO a.rs:7 x=42",
        );
        assert_eq!(recorded[0].1, Level::Info);
        // tail advanced to head; a second poll finds no work.
        assert_eq!(ring.tail().load(Ordering::Relaxed), head);
        drop(recorded);
        assert!(!drain.poll_once(&mut staging, &mut buf));
    }

    #[test]
    #[cfg(not(feature = "fifo-backend"))]
    fn poll_discards_corrupt_record_and_resyncs_to_head() {
        let (mut drain, calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());

        // A valid record, then one whose type byte is neither LOG_RECORD nor
        // END_OF_BUFFER (framing corruption), then a second valid record.
        let r1 = build_record(Level::Info, 0, site_of("first", "", 0), &[]);
        place_record(&ring, 0, &r1);
        let off2 = align_up(r1.len() as u64, SLOT_SIZE as u64);

        let mut bad = build_record(Level::Info, 0, site_of("corrupt", "", 0), &[]);
        bad[1] = 0x7F; // record type: not LOG_RECORD (1) or END_OF_BUFFER (2)
        place_record(&ring, off2, &bad);
        let off3 = off2 + align_up(bad.len() as u64, SLOT_SIZE as u64);

        let r2 = build_record(Level::Info, 0, site_of("second", "", 0), &[]);
        place_record(&ring, off3, &r2);

        drain.rings.push((Arc::clone(&ring), ring.serial()));
        let mut staging = Vec::new();
        let mut buf = Vec::new();
        drain.poll_once(&mut staging, &mut buf);

        // Only the record before the corruption is emitted: the drain must not
        // decode the corrupt bytes as a record, and resyncs `tail` to the
        // producer's published head rather than desyncing onward.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "got {recorded:?}");
        assert!(recorded[0].0.ends_with("first"), "got {:?}", recorded[0].0);
    }

    #[test]
    #[cfg(not(feature = "fifo-backend"))]
    fn poll_skips_end_of_buffer_record() {
        let (mut drain, calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());

        // An EOB record spanning one slot, followed by a real record.
        let mut eob = Vec::new();
        eob.push(VERSION); // version
        eob.push(END_OF_BUFFER); // type
        eob.extend_from_slice(&(SLOT_SIZE as u16).to_le_bytes()); // total_size
        eob.resize(SLOT_SIZE, 0); // pad to a full slot
        place_record(&ring, 0, &eob);

        let record = build_record(Level::Info, 0, site_of("hi", "", 0), &[]);
        place_record(&ring, SLOT_SIZE as u64, &record);

        drain.rings.push((Arc::clone(&ring), ring.serial()));
        let mut staging = Vec::new();
        let mut buf = Vec::new();
        assert!(drain.poll_once(&mut staging, &mut buf));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, "1970-01-01T00:00:00.000000000Z INFO :0 hi",);
    }

    #[test]
    #[cfg(not(feature = "fifo-backend"))]
    fn poll_multiple_records_in_one_ring() {
        let (mut drain, calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());

        let r1 = build_record(Level::Info, 0, site_of("a", "", 0), &[]);
        let r2 = build_record(Level::Warn, 0, site_of("b", "", 0), &[]);
        let slot = SLOT_SIZE as u64;
        // Place two records in consecutive slots and set head past both.
        {
            // SAFETY: single-threaded test with exclusive access.
            let data = unsafe { std::slice::from_raw_parts_mut(ring.data_ptr(), ring.capacity()) };
            data[..r1.len()].copy_from_slice(&r1);
            let off2 = slot as usize;
            data[off2..off2 + r2.len()].copy_from_slice(&r2);
        }
        ring.head().store(2 * slot, Ordering::Release);
        drain.rings.push((Arc::clone(&ring), ring.serial()));

        let mut staging = Vec::new();
        let mut buf = Vec::new();
        assert!(drain.poll_once(&mut staging, &mut buf));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2);
        assert_eq!(recorded[0].1, Level::Info);
        assert_eq!(recorded[1].1, Level::Warn);
    }

    // ---- sync_rings tests ---------------------------------------------------

    fn init_registry() {
        static INIT: OnceLock<()> = OnceLock::new();
        INIT.get_or_init(|| {
            let _ = REGISTRY.set(Mutex::new(Vec::new()));
        });
    }

    #[test]
    fn sync_picks_up_and_drops_rings() {
        init_registry();
        let (mut drain, _calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());
        REGISTRY
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .push(Arc::clone(&ring));

        let mut staging = Vec::new();
        let mut buf = Vec::new();
        drain.sync_rings(&mut staging, &mut buf);
        assert!(drain.rings.iter().any(|(r, _)| Arc::ptr_eq(r, &ring)));

        // Mark the ring dead; the next sync must drop it.
        ring.live.store(false, Ordering::Release);
        drain.sync_rings(&mut staging, &mut buf);
        assert!(!drain.rings.iter().any(|(r, _)| Arc::ptr_eq(r, &ring)));
    }

    #[test]
    fn sync_does_not_add_same_ring_twice() {
        init_registry();
        let (mut drain, _calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());
        REGISTRY
            .get()
            .unwrap()
            .lock()
            .unwrap()
            .push(Arc::clone(&ring));

        let mut staging = Vec::new();
        let mut buf = Vec::new();
        drain.sync_rings(&mut staging, &mut buf);
        drain.sync_rings(&mut staging, &mut buf);
        let count = drain
            .rings
            .iter()
            .filter(|(r, _)| Arc::ptr_eq(r, &ring))
            .count();
        assert_eq!(count, 1);
        // Clean up so a dead ring is not left in the shared registry.
        ring.live.store(false, Ordering::Release);
        drain.sync_rings(&mut staging, &mut buf);
    }

    #[test]
    #[cfg(not(feature = "fifo-backend"))]
    fn sync_drains_dead_ring_before_dropping_it() {
        init_registry();
        let (mut drain, calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());

        // Thread-exit ordering: publish a record (head advances), then mark the
        // producer dead (live = false) with the record still unconsumed. The
        // ring is kept out of the shared REGISTRY so the assertion on `calls`
        // cannot be perturbed by a ring another parallel test registered.
        let record = build_record(Level::Info, 0, site_of("bye", "", 0), &[]);
        place_record(&ring, 0, &record);
        ring.live.store(false, Ordering::Release);
        drain.rings.push((Arc::clone(&ring), ring.serial()));

        let mut staging = Vec::new();
        let mut buf = Vec::new();
        drain.sync_rings(&mut staging, &mut buf);

        // The dead ring's record must reach the sink before the ring is dropped.
        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(recorded[0].0, "1970-01-01T00:00:00.000000000Z INFO :0 bye",);
        drop(recorded);
        // And the ring is gone from the drain's local list afterwards.
        assert!(!drain.rings.iter().any(|(r, _)| Arc::ptr_eq(r, &ring)));
    }

    // ---- Concurrent producer/drain (aliasing model) -------------------------

    /// The producer (`write_record`) and the drain (`drain_ring`) run against
    /// one ring on two threads at once. Both must reach the ring's byte buffer
    /// without ever forming a `&`/`&mut` that spans it: a whole-slice `&mut` on
    /// the producer overlapping a whole-slice `&` on the drain is a
    /// Stacked/Tree-Borrows violation even though the byte ranges are disjoint.
    ///
    /// This is invisible to a normal run; it is a regression guard for Miri:
    ///   MIRIFLAGS="-Zmiri-tree-borrows" \
    ///     cargo +nightly miri test --lib producer_and_drain_concurrent
    #[test]
    #[cfg(not(feature = "fifo-backend"))]
    fn producer_and_drain_concurrent_no_aliasing_ub() {
        use crate::builder::Backpressure;
        use std::sync::Barrier;

        const N: usize = 8;
        let ring = Arc::new(RingBuffer::new());
        let mut staging = Vec::new();
        let record = build_record(
            Level::Info,
            0x1234,
            site_of("hello {}", "f.rs", 1),
            &[le_bytes(TAG_U64, 42)],
        );

        // Both threads start their loops together to maximize the window in
        // which the producer and drain hold references to the buffer at once.
        let barrier = Arc::new(Barrier::new(2));

        let producer = {
            let ring = Arc::clone(&ring);
            let barrier = Arc::clone(&barrier);
            let record = record.clone();
            std::thread::spawn(move || {
                barrier.wait();
                for _ in 0..N {
                    let len = record.len();
                    let slot = ring.reserve(len, Backpressure::Block).unwrap();
                    unsafe {
                        std::ptr::copy_nonoverlapping(record.as_ptr(), slot.ptr, len);
                    }
                    ring.publish(slot);
                }
            })
        };

        let calls = Arc::new(Mutex::new(Vec::new()));
        let mut sink = CaptureSink {
            calls: Arc::clone(&calls),
        };
        let calibration = identity_calibration();
        let line_pattern = default_line_pattern();
        let mut buf = Vec::new();

        barrier.wait();
        let mut guard = 0;
        while calls.lock().unwrap().len() < N {
            drain_ring(
                &ring,
                &mut staging,
                &mut sink,
                0,
                &calibration,
                &line_pattern,
                &mut buf,
            );
            guard += 1;
            assert!(
                guard < 1_000_000,
                "drain stalled before consuming all records"
            );
        }

        producer.join().unwrap();
        assert_eq!(calls.lock().unwrap().len(), N);
    }

    // ---- FIFO-backend drain tests ------------------------------------------

    /// Pushes a fully assembled record into a FIFO ring like the producer
    /// macro path does (reserve, then commit the byte stream).
    #[cfg(feature = "fifo-backend")]
    fn push_record(ring: &RingBuffer, bytes: &[u8]) {
        use crate::builder::Backpressure;
        let slot = ring.reserve(bytes.len(), Backpressure::Drop).unwrap();
        ring.commit(bytes);
        let _ = slot;
    }

    #[test]
    #[cfg(feature = "fifo-backend")]
    fn fifo_drain_formats_complete_records() {
        let (mut drain, calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());
        let record = build_record(
            Level::Info,
            0,
            site_of("x={}", "a.rs", 7),
            &[le_bytes(TAG_U64, 42)],
        );
        drain.rings.push((Arc::clone(&ring), ring.serial()));
        push_record(&ring, &record);

        let mut staging = Vec::new();
        let mut buf = Vec::new();
        assert!(drain.poll_once(&mut staging, &mut buf));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1);
        assert_eq!(
            recorded[0].0,
            "1970-01-01T00:00:00.000000000Z INFO a.rs:7 x=42",
        );
        // Second poll finds nothing further and the staging is empty again.
        drop(recorded);
        assert!(!drain.poll_once(&mut staging, &mut buf));
        assert!(staging.is_empty());
    }

    #[test]
    #[cfg(all(feature = "fifo-backend", not(feature = "backend-triple-buffer")))]
    fn fifo_drain_passes_whole_records_atomically() {
        // The producer commits each record under the FIFO mutex, and the drain
        // pops under the same mutex, so the drain always observes whole
        // records: a partially committed record is never visible. Two records
        // committed back-to-back must both be emitted in a single pump.
        let (mut drain, calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());
        drain.rings.push((Arc::clone(&ring), ring.serial()));

        let r1 = build_record(Level::Info, 0, site_of("one", "", 0), &[]);
        let r2 = build_record(Level::Warn, 0, site_of("two", "", 0), &[]);
        push_record(&ring, &r1);
        push_record(&ring, &r2);

        let mut staging = Vec::new();
        let mut buf = Vec::new();
        assert!(drain.poll_once(&mut staging, &mut buf));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 2, "got {recorded:?}");
        assert_eq!(recorded[0].1, Level::Info);
        assert_eq!(recorded[1].1, Level::Warn);
        drop(recorded);

        // Ring is empty and a re-poll finds nothing.
        assert!(!drain.poll_once(&mut staging, &mut buf));
        assert!(ring.is_empty());
    }

    /// The `backend-triple-buffer` exchange has exactly one slot. `commit`
    /// bypassing the reserve rendezvous overwrites an outstanding value, and
    /// `commit` is the draft this test drives directly (through this
    /// interface). So the drain sees at most one, latest record.
    #[test]
    #[cfg(feature = "backend-triple-buffer")]
    fn triple_buffer_drain_delivers_latest_record_only() {
        let (mut drain, calls) = capture_drain();
        let ring = Arc::new(RingBuffer::new());
        drain.rings.push((Arc::clone(&ring), ring.serial()));

        let r1 = build_record(Level::Info, 0, site_of("one", "", 0), &[]);
        let r2 = build_record(Level::Warn, 0, site_of("two", "", 0), &[]);

        // Two commits without a consumed slot in between: r2 overwrites r1.
        ring.commit(&r1);
        ring.commit(&r2);

        let mut staging = Vec::new();
        let mut buf = Vec::new();
        assert!(drain.poll_once(&mut staging, &mut buf));

        let recorded = calls.lock().unwrap();
        assert_eq!(recorded.len(), 1, "got {recorded:?}");
        assert_eq!(recorded[0].1, Level::Warn); // only the latest survives
        drop(recorded);

        // A re-poll finds nothing further.
        assert!(!drain.poll_once(&mut staging, &mut buf));
        assert!(ring.is_empty());
    }
}
