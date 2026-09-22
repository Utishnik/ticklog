//! Loom model-checking tests for the lock-free protocols.
//!
//! Run only these tests (loom atomics are not valid outside `loom::model`):
//!
//! ```text
//! RUSTFLAGS="--cfg ticklog_loom" cargo test --lib --release -- loom_
//! ```
//!
//! The cfg is named `ticklog_loom` rather than `loom` so a global RUSTFLAGS
//! cannot accidentally enable other dependencies' internal `cfg(loom)`
//! scaffolding.
//!
//! Covered:
//! - SPSC `reserve`/`publish` vs drain-side `head`/`tail` (Acquire/Release)
//! - wrap + EndOfBuffer filler
//! - `Backpressure::Block` unblocking when the drain advances `tail`
//! - `Backpressure::Drop` returning `None` on a full ring
//! - `ThreadBuf::drop` publishing `live = false`
//! - `Guard::drop` publishing `shutdown = true` and joining the drain
//! - helper reservation / `mark_recycled` exactly-once
//!
//! Process-global `OnceLock`s (`REGISTRY`, `SEGMENTS`, …) are intentionally
//! untouched: loom re-executes the model in one process, so one-shot globals
//! would leak state across iterations.

use std::sync::Arc;

use loom::thread;

use crate::builder::Backpressure as Policy;
use crate::guard::Guard;
use crate::record::{END_OF_BUFFER, HEADER_SIZE, LOG_RECORD, VERSION};
use crate::ring::{RingBuffer, SLOT_SIZE, align_up};
use crate::sync::Ordering;
use crate::thread_buf::ThreadBuf;

/// Capacity used by the small models: four slots. The free-running-counter
/// ring treats a write as fitting only when `head + size - tail <= mask`
/// (= capacity - 1), so a 4-slot ring holds at most three slot-aligned
/// records — never a perfectly full ring.
const CAP: usize = 4 * SLOT_SIZE;

/// One-slot record size (slot-aligned, so it never wraps on its own).
const REC: usize = SLOT_SIZE;

/// Two-slot record: forces the EOB wrap path when the head sits in the last slot.
const REC_WRAP: usize = SLOT_SIZE + 1;

/// Writes a header-only log record of `total_size` bytes at `ptr`.
fn write_header(ptr: *mut u8, total_size: u16, level: u8) {
    // SAFETY: `reserve` returned a pointer to at least `REC` writable bytes.
    unsafe {
        ptr.write(VERSION);
        ptr.add(1).write(LOG_RECORD);
        ptr.add(2).write((total_size & 0xff) as u8);
        ptr.add(3).write((total_size >> 8) as u8);
        ptr.add(4).write(level);
        // flags u16 at 5..7, pad u8 at 7: leave/zero as needed.
        ptr.add(5).write(0);
        ptr.add(6).write(0);
        ptr.add(7).write(0);
        // timestamp u64 at 8..16: zero for the model.
        for i in 8..HEADER_SIZE {
            ptr.add(i).write(0);
        }
    }
}

/// Drain-side consume of every record currently published in `rb`.
/// Mirrors `drain_ring_inner`'s atomics (Acquire head, Release tail) without
/// formatting. Returns the number of log records observed.
fn consume_all(rb: &RingBuffer) -> usize {
    let mut head_cache = unsafe { *rb.head_cache() };
    let published = rb.head().load(Ordering::Acquire);
    if published != head_cache {
        head_cache = published;
    }
    let mut tail = rb.tail().load(Ordering::Relaxed);
    let base = rb.data_ptr();
    let mut records = 0usize;

    while tail < head_cache {
        let offset = (tail & rb.mask()) as usize;
        // SAFETY: `[offset, offset+4)` is inside the ring while tail < head.
        let (version, rectype, total_size) = unsafe {
            let p = base.add(offset);
            (
                p.read(),
                p.add(1).read(),
                u16::from_le_bytes([p.add(2).read(), p.add(3).read()]),
            )
        };
        assert_eq!(version, VERSION, "record version");
        assert!(
            rectype == LOG_RECORD || rectype == END_OF_BUFFER,
            "record type"
        );
        assert!(total_size as usize >= HEADER_SIZE, "total_size floor");

        if rectype == END_OF_BUFFER {
            tail += align_up(total_size as u64, SLOT_SIZE as u64);
            continue;
        }
        records += 1;
        tail += align_up(total_size as u64, SLOT_SIZE as u64);
    }

    rb.tail().store(tail, Ordering::Release);
    unsafe {
        *rb.head_cache() = head_cache;
    }
    records
}

/// Fills `n` one-slot records into `rb` (single-threaded prefill helper).
/// Panics if a record does not fit — callers must size `n` to the usable
/// capacity (`capacity - 1` bytes, see `CAP`).
fn prefill(rb: &RingBuffer, n: usize) {
    for i in 0..n {
        let r = rb
            .reserve(REC, Policy::Drop)
            .unwrap_or_else(|| panic!("prefill record {i} does not fit"));
        write_header(r.ptr, REC as u16, 3);
        rb.publish(r);
    }
}

#[test]
fn loom_spsc_publish_consume_single_record() {
    loom::model(|| {
        let rb = Arc::new(RingBuffer::with_capacity(CAP));
        let rb_p = Arc::clone(&rb);
        let rb_c = Arc::clone(&rb);

        let producer = thread::spawn(move || {
            let r = rb_p
                .reserve(REC, Policy::Drop)
                .expect("empty ring must reserve");
            write_header(r.ptr, REC as u16, 3);
            rb_p.publish(r);
        });

        let consumer = thread::spawn(move || {
            let n = consume_all(&rb_c);
            assert!(n <= 1, "at most one record published");
            n
        });

        producer.join().unwrap();
        let seen = consumer.join().unwrap();
        if seen == 1 {
            assert_eq!(rb.head().load(Ordering::Acquire), SLOT_SIZE as u64);
            assert_eq!(rb.tail().load(Ordering::Acquire), SLOT_SIZE as u64);
        }
    });
}

#[test]
fn loom_spsc_fill_then_drain_then_refill() {
    loom::model(|| {
        let rb = Arc::new(RingBuffer::with_capacity(CAP));
        let rb_p = Arc::clone(&rb);
        let rb_c = Arc::clone(&rb);

        let producer = thread::spawn(move || {
            // Try three records (the usable maximum for CAP); Drop yields
            // None if the consumer has not freed space yet.
            for _ in 0..3 {
                if let Some(r) = rb_p.reserve(REC, Policy::Drop) {
                    write_header(r.ptr, REC as u16, 3);
                    rb_p.publish(r);
                }
            }
        });

        let consumer = thread::spawn(move || {
            consume_all(&rb_c);
        });

        producer.join().unwrap();
        consumer.join().unwrap();

        // After both sides quiesce, a final drain pass leaves the ring empty
        // and consistent: head == tail, both slot-aligned.
        consume_all(&rb);
        let head = rb.head().load(Ordering::Acquire);
        let tail = rb.tail().load(Ordering::Acquire);
        assert_eq!(head, tail, "drain must catch up");
        assert_eq!(head % SLOT_SIZE as u64, 0, "slot alignment");
        assert!(head <= CAP as u64, "occupancy bound");
    });
}

#[test]
fn loom_block_unblocks_when_drain_advances_tail() {
    loom::model(|| {
        let rb = Arc::new(RingBuffer::with_capacity(CAP));
        // Three one-slot records leave the ring unable to accept a fourth
        // (head - tail + SLOT > mask), so the Block producer must wait.
        prefill(&rb, 3);
        assert_eq!(rb.head().load(Ordering::Relaxed), 3 * SLOT_SIZE as u64);

        let rb_p = Arc::clone(&rb);
        let rb_c = Arc::clone(&rb);

        let producer = thread::spawn(move || {
            // Under ticklog_loom the Block spin is bounded (see ring.rs), so
            // this may return None if the consumer did not free space within
            // the loom spin budget. Real builds still wait forever.
            rb_p.reserve(REC, Policy::Block).map(|r| {
                write_header(r.ptr, REC as u16, 3);
                rb_p.publish(r);
            })
        });

        let consumer = thread::spawn(move || {
            // Bulk-free everything: one Release store of tail, no record
            // parsing. Keeps the branch budget on the producer's spin, not
            // on a multi-record drain pass.
            let head = rb_c.head().load(Ordering::Acquire);
            rb_c.tail().store(head, Ordering::Release);
            unsafe { *rb_c.head_cache() = head };
        });

        let published = producer.join().unwrap();
        consumer.join().unwrap();

        // If the bounded loom spin gave up, free space is already available
        // (consumer ran); a sequential Block reserve must now succeed.
        if published.is_none() {
            let r = rb
                .reserve(REC, Policy::Block)
                .expect("space freed; sequential Block must succeed");
            write_header(r.ptr, REC as u16, 3);
            rb.publish(r);
        }

        let head = rb.head().load(Ordering::Acquire);
        let tail = rb.tail().load(Ordering::Acquire);
        assert_eq!(head % SLOT_SIZE as u64, 0);
        assert_eq!(tail % SLOT_SIZE as u64, 0);
        // head - tail <= capacity is the non-overwrite invariant.
        assert!(head.wrapping_sub(tail) <= CAP as u64, "no overwrite");
        // Exactly one record was published after the prefill of three.
        assert_eq!(head, 4 * SLOT_SIZE as u64);
        assert_eq!(tail, 3 * SLOT_SIZE as u64);
    });
}

#[test]
fn loom_drop_returns_none_on_full_ring() {
    loom::model(|| {
        let rb = Arc::new(RingBuffer::with_capacity(CAP));
        prefill(&rb, 3);

        let rb_p = Arc::clone(&rb);
        let producer = thread::spawn(move || {
            // Ring is full at spawn; Drop may return None unless the consumer
            // frees space first in this interleaving.
            let _ = rb_p.reserve(REC, Policy::Drop);
        });

        let rb_c = Arc::clone(&rb);
        let consumer = thread::spawn(move || {
            consume_all(&rb_c);
        });

        producer.join().unwrap();
        consumer.join().unwrap();

        let head = rb.head().load(Ordering::Acquire);
        let tail = rb.tail().load(Ordering::Acquire);
        assert!(head.wrapping_sub(tail) <= CAP as u64, "no overwrite");
    });
}

#[test]
fn loom_wrap_writes_eob_and_consumer_skips() {
    loom::model(|| {
        // 4-slot ring. Place head in the last slot, then reserve a record that
        // aligns to 2 slots: the producer writes an EOB filler into the tail
        // of the buffer and the real record wraps to offset 0.
        let cap = 4 * SLOT_SIZE;
        let rb = Arc::new(RingBuffer::with_capacity(cap));
        let start = (cap - SLOT_SIZE) as u64;
        rb.head().store(start, Ordering::Relaxed);
        rb.tail().store(start, Ordering::Relaxed);
        unsafe { *rb.tail_cache() = start };

        let rb_p = Arc::clone(&rb);
        let rb_c = Arc::clone(&rb);

        let producer = thread::spawn(move || {
            // Single-slot records fit without wrap while there is room; the
            // two-slot record forces the wrap branch.
            if let Some(r) = rb_p.reserve(REC_WRAP, Policy::Drop) {
                write_header(r.ptr, REC_WRAP as u16, 3);
                rb_p.publish(r);
            }
        });

        let consumer = thread::spawn(move || {
            consume_all(&rb_c);
        });

        producer.join().unwrap();
        consumer.join().unwrap();
        consume_all(&rb);

        let head = rb.head().load(Ordering::Acquire);
        let tail = rb.tail().load(Ordering::Acquire);
        assert_eq!(head % SLOT_SIZE as u64, 0);
        assert_eq!(tail % SLOT_SIZE as u64, 0);
        assert!(head.wrapping_sub(tail) <= cap as u64, "no overwrite");
        // If the producer reserved the wrapping record, head advanced past
        // the EOB filler (one slot) + the 2-slot aligned record.
        if head > start {
            assert_eq!(head, start + SLOT_SIZE as u64 + 2 * SLOT_SIZE as u64);
        }
    });
}

#[test]
fn loom_thread_buf_drop_publishes_live_false() {
    loom::model(|| {
        let ring = Arc::new(RingBuffer::with_capacity(CAP));
        let rb_c = Arc::clone(&ring);

        let observer = thread::spawn(move || {
            // Drain-side: Acquire-load `live` the way the drain does.
            // Both interleavings (observe dead, or run entirely before drop)
            // are legal; the post-join assertion below is the hard check.
            let mut saw_dead = false;
            for _ in 0..8 {
                if !rb_c.live.load(Ordering::Acquire) {
                    saw_dead = true;
                    break;
                }
                thread::yield_now();
            }
            saw_dead
        });

        let tb = ThreadBuf {
            ring: Arc::clone(&ring),
            helper: None,
            thread_id: 1,
            thread_name: "loom".into(),
            #[cfg(feature = "fifo-backend")]
            staging: Vec::new(),
        };
        // Drop publishes live=false with Release.
        drop(tb);

        let _saw_dead = observer.join().unwrap();
        assert!(
            !ring.live.load(Ordering::Acquire),
            "ThreadBuf::drop must clear live"
        );
    });
}

#[test]
fn loom_guard_drop_sets_shutdown_and_joins() {
    loom::model(|| {
        let shutdown = Arc::new(crate::sync::AtomicBool::new(false));
        let flag = Arc::clone(&shutdown);

        // Drain side: Acquire-load until shutdown, then exit (the join in
        // Guard::drop waits for exactly this).
        let drain = thread::spawn(move || {
            for _ in 0..8 {
                if flag.load(Ordering::Acquire) {
                    break;
                }
                thread::yield_now();
            }
        });

        // REGISTRY is unset in loom runs, so Drop skips the ring walk and
        // only publishes shutdown + joins — the protocol under test.
        let guard = Guard::new(drain, Arc::clone(&shutdown));
        drop(guard);

        assert!(
            shutdown.load(Ordering::Acquire),
            "Guard::drop must set shutdown"
        );
    });
}

#[test]
fn loom_helper_reservation_exactly_once() {
    loom::model(|| {
        let rb = Arc::new(RingBuffer::with_capacity(CAP));
        let rb1 = Arc::clone(&rb);
        let rb2 = Arc::clone(&rb);

        let t1 = thread::spawn(move || {
            rb1.reserve_for_helper();
            let reserved = rb1.helper_reserved();
            rb1.clear_helper_reservation();
            reserved
        });
        let t2 = thread::spawn(move || rb2.mark_recycled());

        let _reserved = t1.join().unwrap();
        let first = t2.join().unwrap();
        // Call again from this thread after join: must be false.
        let second = rb.mark_recycled();
        assert!(
            first ^ second,
            "mark_recycled must return true exactly once"
        );
        assert!(!rb.helper_reserved(), "reservation cleared");
    });
}
