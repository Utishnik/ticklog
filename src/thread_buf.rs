//! Per-thread ring buffer ownership and the global ring registry.
//!
//! Each thread holds an [`Arc`]`<`[`RingBuffer`]`>` in a thread-local
//! [`UnsafeCell`] slot. The global [`REGISTRY`] tracks all active rings.

use std::cell::{Cell, UnsafeCell};
#[cfg(not(feature = "fifo-backend"))]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use crate::builder::Backpressure;
use crate::error::TicklogError;
#[cfg(not(feature = "fifo-backend"))]
use crate::ring::Reservation;
use crate::ring::{DEFAULT_RING_SIZE, RingBuffer};

/// A thread's local ring buffer and cached metadata.
///
/// Cached values (`thread_id`, `thread_name`) are set once at creation and
/// never change. The ring is shared with the drain thread via [`Arc`].
pub(crate) struct ThreadBuf {
    /// The ring buffer this thread writes records into.
    pub(crate) ring: Arc<RingBuffer>,
    /// A handed-off segment that this producer reserved for cooperative
    /// formatting while it waits for a free NanoLog pool segment. At most one
    /// outstanding segment per producer: a blocked producer holds exactly one
    /// full segment and cannot produce more until it unblocks. The
    /// reservation is cleared in [`Drop`] so a dying producer never leaves
    /// the drain waiting on it.
    pub(crate) helper: Option<Arc<RingBuffer>>,
    /// Cached stable thread identifier.
    pub(crate) thread_id: u64,
    /// Cached thread name. Falls back to `<unnamed>` when the OS thread has no name.
    pub(crate) thread_name: String,
    /// Reusable staging buffer for backend-record byte pushes. Only the
    /// experimental FIFO backends write records this way.
    #[cfg(feature = "fifo-backend")]
    pub(crate) staging: Vec<u8>,
}

impl Drop for ThreadBuf {
    fn drop(&mut self) {
        // A reserved-but-unformatted segment (producer exited mid-wait) must
        // not leave the drain blocked on it: clear the reservation and mark
        // the segment dead so the drain's final drain recycles it.
        #[cfg(not(feature = "fifo-backend"))]
        if let Some(ring) = self.helper.take() {
            ring.clear_helper_reservation();
            ring.live.store(false, Ordering::Release);
        }
        // Signal to the drain: no more records will be written.
        // Release pairs with the drain's Acquire load of `live`,
        // guaranteeing all prior head stores are visible.
        self.ring.live.store(false, Ordering::Release);
        // Arc<RingBuffer> dropped implicitly. Buffer stays alive if
        // the drain still holds its clone.
    }
}

/// Maximum length of a cached thread name, in bytes. Names longer than this
/// are truncated so the ring's registered name stays small.
const MAX_THREAD_NAME_LEN: usize = 256;

/// Per-thread ring buffer capacity, set once at [`crate::configure!`] time.
/// Falls back to [`DEFAULT_RING_SIZE`] when the ring registry is initialized
/// directly (as in tests) without a matching configure call.
pub(crate) static RING_CAPACITY: OnceLock<usize> = OnceLock::new();

/// The active [`Backpressure`] policy, set once at [`crate::configure!`] time.
/// The drain and the segmented handoff path consult it on every pass.
pub(crate) static BACKPRESSURE: OnceLock<Backpressure> = OnceLock::new();

/// Monotonic registration counter for ring `serial`s. Each registration of a
/// ring object (including re-registration of a recycled pool segment) stamps a
/// fresh serial, which lets the drain distinguish a segment's incarnations.
#[cfg(not(feature = "fifo-backend"))]
static NEXT_SERIAL: AtomicU64 = AtomicU64::new(0);

/// Extracts a stable `u64` identifier from [`std::thread::ThreadId`] by
/// parsing its `Debug` representation.
///
/// Returns `0` if the internal format changes unexpectedly.
fn get_stable_thread_id() -> u64 {
    let id = thread::current().id();
    let id_str = format!("{id:?}"); // e.g. "ThreadId(2)"
    id_str
        .strip_prefix("ThreadId(")
        .and_then(|s| s.strip_suffix(')'))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0)
}

/// Per-thread state: the ring buffer plus a re-entrancy flag.
///
/// The flag lives alongside `buf` in the same thread-local so the hot path
/// still performs a single TLS lookup. It is read and written through [`Cell`]
/// (interior mutability, never a `&mut`), so touching it never aliases the
/// `&mut Option<ThreadBuf>` formed from the sibling `buf` field.
struct ThreadSlot {
    /// Set while [`with_thread_buf`] is running its closure. A re-entrant call
    /// on the same thread observes it set and bails out before forming a second
    /// `&mut *buf.get()`; that aliasing would be Undefined Behavior.
    active: Cell<bool>,
    /// The thread's ring buffer and scratch. [`UnsafeCell`] (not
    /// [`std::cell::RefCell`]) keeps the hot path branch-free; the `active`
    /// flag, not a runtime borrow count, is what enforces exclusive access.
    buf: UnsafeCell<Option<ThreadBuf>>,
}

thread_local! {
    /// Per-thread ring buffer and re-entrancy flag. Each thread has exclusive
    /// access to its own slot.
    static THREAD_SLOT: ThreadSlot = const {
        ThreadSlot {
            active: Cell::new(false),
            buf: UnsafeCell::new(None),
        }
    };
}

/// Resets the re-entrancy flag when it drops, so the flag is cleared even if the
/// closure passed to [`with_thread_buf`] panics; a single panicking encode
/// must not permanently silence this thread's logging.
struct ActiveGuard<'a>(&'a Cell<bool>);

impl Drop for ActiveGuard<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

/// Accesses the current thread's [`ThreadBuf`], initializing it on first
/// call, and runs `f` with exclusive access to it.
///
/// Lazy init allocates a new [`RingBuffer`], registers it with the global
/// [`REGISTRY`], and caches thread metadata. Subsequent calls return
/// immediately.
///
/// Returns `Some(f(..))` normally. Returns `None` without running `f` if this
/// is a **re-entrant** call on the same thread, i.e. `f` itself (a
/// `Loggable::encode` that logs, or a panic hook that logs mid-encode) called
/// back into `with_thread_buf`. Such a nested log is dropped rather than
/// allowed to form a second aliasing `&mut` into the thread-local slot.
///
/// # Panics
///
/// Panics if the [`REGISTRY`] has not been initialized by
/// [`crate::configure!`].
pub(crate) fn with_thread_buf<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut ThreadBuf) -> R,
{
    THREAD_SLOT.with(|slot| {
        // Refuse a re-entrant call. Checked through `Cell` (no `&mut`), so it
        // happens before (and without aliasing) the `&mut *buf.get()` below.
        if slot.active.get() {
            return None;
        }
        slot.active.set(true);
        // Clear `active` on the way out, including on a panic unwinding through
        // `f`, so one panicking encode doesn't leave the thread permanently
        // marked active (which would drop all its future records).
        let _active = ActiveGuard(&slot.active);

        // SAFETY: `active` was false and is now true, so no other frame on this
        // thread holds a reference into `buf`; this `&mut` is exclusive. The raw
        // pointer from `buf.get()` is thread-local and never shared across
        // threads.
        let opt = unsafe { &mut *slot.buf.get() };

        if opt.is_none() {
            let capacity = RING_CAPACITY.get().copied().unwrap_or(DEFAULT_RING_SIZE);
            #[cfg(not(feature = "fifo-backend"))]
            let ring = if crate::segments::Segments::installed() {
                // Segmented policies allocate a producer's first buffer from
                // the arena-backed pool instead of the crate allocator.
                crate::segments::Segments::get().initial_ring()
            } else {
                Arc::new(RingBuffer::with_capacity(capacity))
            };
            #[cfg(feature = "fifo-backend")]
            let ring = Arc::new(RingBuffer::with_capacity(capacity));
            let thread_id = get_stable_thread_id();
            let thread_name: String = thread::current()
                .name()
                .map(String::from)
                .unwrap_or_else(|| "<unnamed>".to_string());
            // Truncate names longer than the display limit on a valid
            // UTF-8 boundary so the slice never splits a multi-byte
            // character (which would panic).
            let thread_name = if thread_name.len() > MAX_THREAD_NAME_LEN {
                let mut end = MAX_THREAD_NAME_LEN;
                while !thread_name.is_char_boundary(end) {
                    end -= 1;
                }
                thread_name[..end].to_string()
            } else {
                thread_name
            };
            // The drain keys thread identity off the ring (a ring belongs to
            // exactly one producer), so the name is registered here once and
            // never repeated in the records.
            ring.set_thread_info(thread_id, &thread_name);
            register_ring(Arc::clone(&ring));
            *opt = Some(ThreadBuf {
                ring,
                helper: None,
                thread_id,
                thread_name,
                #[cfg(feature = "fifo-backend")]
                staging: Vec::with_capacity(1024),
            });
        }

        // SAFETY: `opt` was just initialized above if it was `None`, so
        // `unwrap_unchecked` is sound.
        Some(f(unsafe { opt.as_mut().unwrap_unchecked() }))
    })
}

/// Global registry of all active ring buffers.
///
/// Set once at initialization. New producer threads register their rings
/// here during lazy init or [`warm_up`].
pub(crate) static REGISTRY: OnceLock<Mutex<Vec<Arc<RingBuffer>>>> = OnceLock::new();

/// Registers a ring buffer with the global [`REGISTRY`].
///
/// Re-registering an already-registered ring object (a recycled pool segment)
/// bumps its `serial` in place instead of adding a duplicate entry; the drain
/// uses the serial to tell the incarnations apart.
///
/// # Panics
///
/// Panics if the [`REGISTRY`] has not been initialized.
pub(crate) fn register_ring(ring: Arc<RingBuffer>) {
    let mut rings = REGISTRY
        .get()
        .expect("invariant: ring registry not initialized; call ticklog::configure! before logging")
        .lock()
        .expect("invariant: ring registry mutex poisoned by a panic in another thread");
    // Stamp a fresh serial. Happens under the registry lock so the drain's
    // serial checks and any in-place registry replacement below stay atomic
    // with respect to re-registration.
    #[cfg(not(feature = "fifo-backend"))]
    let serial = NEXT_SERIAL.fetch_add(1, Ordering::Relaxed);
    #[cfg(not(feature = "fifo-backend"))]
    ring.set_serial(serial);
    if let Some(existing) = rings.iter_mut().find(|r| Arc::ptr_eq(r, &ring)) {
        *existing = ring;
    } else {
        rings.push(ring);
    }
}

/// Reserves space for a record, applying the full backpressure policy.
///
/// Beyond the ring's own policy handling (`Drop` discards, `Block` spins),
/// the segmented policies (`NanoLog`, `Quill`) *hand the full segment to the
/// drain* and switch to a fresh arena-backed one instead of waiting:
///
/// - **Quill** carves a fresh segment, so the producer never blocks.
/// - **NanoLog** claims a pooled segment; when the pool is exhausted the
///   segment is reserved and the producer formats it itself (the drain's
///   work) into the shared line queue, recycling it to free the needed spare.
///
/// Returns `None` when the record must be dropped ([`Backpressure::Drop`], or
/// teardown under a segmented policy).
#[cfg(not(feature = "fifo-backend"))]
#[inline(always)]
pub(crate) fn reserve_with_policy(
    tb: &mut ThreadBuf,
    total_size: usize,
    policy: Backpressure,
) -> Option<Reservation> {
    match policy {
        Backpressure::Drop | Backpressure::Block => tb.ring.reserve(total_size, policy),
        Backpressure::NanoLog | Backpressure::Quill => {
            // Reachable under fifo-backend only after `configure!` degraded
            // the policy to Block, so the pool is never installed here: the
            // ring handles it (Block spin / no-op).
            if !crate::segments::Segments::installed() {
                return tb.ring.reserve(total_size, policy);
            }
            let segments = crate::segments::Segments::get();
            loop {
                if let Some(slot) = tb.ring.reserve(total_size, policy) {
                    return Some(slot);
                }
                // The current segment is full: hand it to the drain and take
                // a fresh one.
                #[cfg(feature = "watermark-head")]
                tb.ring.flush_watermark();
                tb.ring.mark_handed_off();
                let new_ring = if policy == Backpressure::NanoLog {
                    let spare = segments.try_take_spare();
                    match spare {
                        // A free pooled segment: no need to block, the drain
                        // will drain the old one inline.
                        Some(spare) => spare,
                        None => {
                            // Pool exhausted: reserve our segment and format
                            // it ourselves while we wait for a spare.
                            tb.ring.reserve_for_helper();
                            tb.helper = Some(Arc::clone(&tb.ring));
                            segments.first_ring(&mut tb.helper, Some(&tb.ring.live))?
                        }
                    }
                } else {
                    segments.alloc_fresh()
                };
                // The drain keys thread identity off the ring; record it for
                // this (possibly recycled) segment's new incarnation before it
                // becomes visible through the registry.
                new_ring.set_thread_info(tb.thread_id, &tb.thread_name);
                register_ring(Arc::clone(&new_ring));
                tb.ring = new_ring;
                // Try the fresh (empty) segment; the loop also re-enters after
                // any future handoff.
            }
        }
    }
}

/// Prepares the calling thread for logging by allocating its buffer up front,
/// moving the one-time, first-log allocation off a latency-sensitive path.
///
/// Calling it more than once on a thread is a no-op. Threads that skip it are
/// prepared lazily on their first log call instead.
///
/// # Errors
///
/// Returns [`TicklogError::NotInitialized`] if ticklog has not been
/// initialized yet (no [`configure!`][crate::configure!] has run).
pub fn warm_up() -> Result<(), TicklogError> {
    if REGISTRY.get().is_none() {
        return Err(TicklogError::NotInitialized);
    }
    with_thread_buf(|_| {});
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_registry() {
        let _ = REGISTRY.set(Mutex::new(Vec::new()));
    }

    #[test]
    fn get_stable_thread_id_returns_nonzero() {
        let id = get_stable_thread_id();
        assert!(id > 0, "expected non-zero thread id, got {id}");
    }

    #[test]
    fn thread_buf_holds_ring_and_metadata() {
        let ring = Arc::new(RingBuffer::new());
        let tb = ThreadBuf {
            ring: Arc::clone(&ring),
            helper: None,
            thread_id: 42,
            thread_name: "test-thread".into(),
            #[cfg(feature = "fifo-backend")]
            staging: Vec::new(),
        };
        assert_eq!(tb.thread_id, 42);
        assert_eq!(&tb.thread_name, "test-thread");
        assert!(tb.ring.live.load(Ordering::Relaxed));
    }

    #[test]
    fn drop_sets_live_to_false() {
        let ring = Arc::new(RingBuffer::new());
        let tb = ThreadBuf {
            ring: Arc::clone(&ring),
            helper: None,
            thread_id: 1,
            thread_name: "t".into(),
            #[cfg(feature = "fifo-backend")]
            staging: Vec::new(),
        };
        assert!(ring.live.load(Ordering::Relaxed));
        drop(tb);
        assert!(!ring.live.load(Ordering::Relaxed));
    }

    #[test]
    fn buffer_survives_thread_buf_drop_when_other_arcs_exist() {
        let ring = Arc::new(RingBuffer::new());
        let other = Arc::clone(&ring);
        let tb = ThreadBuf {
            ring: Arc::clone(&ring),
            helper: None,
            thread_id: 1,
            thread_name: "t".into(),
            #[cfg(feature = "fifo-backend")]
            staging: Vec::new(),
        };
        drop(tb);
        // Drop set live = false on the shared RingBuffer.
        assert!(!ring.live.load(Ordering::Relaxed));
        // `other` is still a valid Arc; clone and drop without panic.
        let still_here = Arc::clone(&other);
        drop(still_here);
        drop(ring);
        drop(other);
    }

    #[test]
    fn warm_up_and_with_thread_buf_lifecycle() {
        init_registry();
        // First call initializes.
        warm_up().unwrap();
        // Second call is idempotent.
        warm_up().unwrap();
        // with_thread_buf finds existing ThreadBuf from warm_up.
        with_thread_buf(|tb| {
            assert!(tb.thread_id > 0);
        });
    }

    #[test]
    fn register_ring_adds_to_registry() {
        init_registry();
        let mut rings = REGISTRY.get().unwrap().lock().unwrap();
        let count_before = rings.len();
        rings.push(Arc::new(RingBuffer::new()));
        assert_eq!(rings.len(), count_before + 1);
    }

    #[test]
    fn with_thread_buf_thread_id_matches_current_thread() {
        init_registry();
        let expected = get_stable_thread_id();
        with_thread_buf(|tb| {
            assert_eq!(tb.thread_id, expected);
        });
    }

    #[test]
    fn reentrant_with_thread_buf_is_refused() {
        init_registry();
        // Models a `Loggable::encode` (or a panic hook) that logs while the
        // outer record is still being assembled: the inner call runs while the
        // outer `&mut ThreadBuf` is live. On the unguarded code the inner
        // `&mut *buf.get()` aliases the outer one -> UB (Miri flags it).
        // With the re-entrancy guard the inner call is refused.
        let outer = with_thread_buf(|_outer| {
            // Re-entrant call on the same thread must return None.
            with_thread_buf(|_inner| ())
        });
        assert_eq!(
            outer,
            Some(None),
            "outer must succeed, inner must be refused"
        );
    }

    #[test]
    fn set_thread_info_is_visible_to_the_drain() {
        // The record layout carries no thread bytes; the ring supplies them.
        // Guard that registration publishes the id/name for the drain to read.
        let ring = Arc::new(RingBuffer::new());
        ring.set_thread_info(37, "producer-37");
        assert_eq!(ring.thread_id(), 37);
        assert_eq!(ring.thread_name(), "producer-37");
        // Re-registration (handoff) overwrites the previous identity.
        ring.set_thread_info(38, "producer-38");
        assert_eq!(ring.thread_id(), 38);
        assert_eq!(ring.thread_name(), "producer-38");
    }

    /// A thread name whose byte-length exceeds [`MAX_THREAD_NAME_LEN`] and
    /// whose 256th byte falls inside a multi-byte UTF-8 character must not
    /// panic. 255 ASCII `a`s + `é` (2 bytes) = 257 bytes; the byte-index
    /// slice `[..256]` lands mid-char without `floor_char_boundary`.
    #[test]
    fn thread_name_truncation_respects_utf8_boundary() {
        init_registry();
        let mut name = "a".repeat(255);
        name.push('é'); // U+00E9, 2 bytes in UTF-8
        assert_eq!(name.len(), 257);

        let joined = std::thread::Builder::new()
            .name(name)
            .spawn(move || {
                with_thread_buf(|tb| {
                    assert!(
                        tb.thread_name.len() <= MAX_THREAD_NAME_LEN,
                        "truncated name too long: {}",
                        tb.thread_name.len()
                    );
                    // Must be valid UTF-8.
                    let _ = tb.thread_name.chars().count();
                    tb.thread_id
                })
            })
            .unwrap()
            .join()
            .unwrap();

        assert!(joined.is_some(), "with_thread_buf must succeed");
    }
}
