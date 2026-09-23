//! Quill-style segmented policy: strict per-thread order and zero loss even
//! while the drain is stalled.
//!
//! `Backpressure::Quill` grows the arena without bound, so a producer that
//! outruns a stalled drain hands off full segments to the registry and carves
//! fresh ones instead of blocking (or dropping). Every record must survive —
//! in the exact order it was logged, because a single producer never reorders
//! its own segments and the drain emits them in registration order.
//!
//! The drain is stalled by blocking inside the sink's first `accept`, exactly
//! as `backpressure_drop.rs` does. Under Quill this must not lose a single
//! record: the crash-guard is that the producer visibly never blocks.
//!
//! This runs only on the crate-local ring: under a `backend-*` feature the
//! segmented policies degrade to `Block` (see `builder.rs`), so the
//! arena-handed-off-segment premise does not hold there.
//!
//! Each integration test is its own binary, so this file's single `configure!`
//! owns the process-global registry for the whole run.

use std::io;
use std::sync::{Arc, Condvar, Mutex};

use ticklog::{Backpressure, Level, LogSink, info};

/// More records than several 1 MiB segments can hold, forcing many handoffs
/// while the drain is stalled.
const PRODUCED: usize = 200_000;

/// A one-shot gate: the drain blocks on `wait` until the test calls `open`.
struct Gate {
    open: Mutex<bool>,
    ready: Condvar,
}

impl Gate {
    fn new() -> Self {
        Self {
            open: Mutex::new(false),
            ready: Condvar::new(),
        }
    }

    fn wait(&self) {
        let mut open = self.open.lock().unwrap();
        while !*open {
            open = self.ready.wait(open).unwrap();
        }
    }

    fn open(&self) {
        *self.open.lock().unwrap() = true;
        self.ready.notify_all();
    }
}

/// A sink that blocks on its first `accept` until the gate opens, stalling the
/// drain, then records every line it receives.
struct GateSink {
    lines: Arc<Mutex<Vec<String>>>,
    gate: Arc<Gate>,
    passed: bool,
}

impl LogSink for GateSink {
    fn accept(&mut self, line: &[u8], _level: Level) -> io::Result<()> {
        if !self.passed {
            self.gate.wait();
            self.passed = true;
        }
        self.lines
            .lock()
            .unwrap()
            .push(String::from_utf8_lossy(line).into_owned());
        Ok(())
    }
}

/// Extracts the `i` from a line ending in `seq=<i>`.
fn seq_of(line: &str) -> Option<usize> {
    line.rsplit("seq=").next()?.trim().parse().ok()
}

#[test]
#[cfg(not(feature = "fifo-backend"))]
fn quill_preserves_order_and_loses_nothing_under_stalled_drain() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let gate = Arc::new(Gate::new());
    let sink = GateSink {
        lines: Arc::clone(&lines),
        gate: Arc::clone(&gate),
        passed: false,
    };

    let guard = ticklog::configure! {
        sink: sink,
        max_level: Level::Info,
        backpressure: Backpressure::Quill,
    }
    .expect("first configure in a fresh process must succeed");

    // The drain is stalled on the gate. Under Quill each full segment is
    // handed off and a fresh one carved from the arena, so this loop never
    // blocks; if it did, the test would hang here instead of reaching the
    // assertions (the whole point: Quill trades memory, not time).
    for i in 0..PRODUCED {
        info!("seq={}", i as u64);
    }

    // Let the drain deliver every handed-off segment, then flush on drop.
    gate.open();
    drop(guard);

    let captured = lines.lock().unwrap();

    // Zero loss: the unbounded arena absorbed the whole burst.
    assert_eq!(
        captured.len(),
        PRODUCED,
        "Quill must lose nothing under a stalled drain, got {} of {}",
        captured.len(),
        PRODUCED
    );

    // Strict per-thread order across all handoffs: the drain emits a single
    // producer's segments in registration order.
    let seqs: Vec<usize> = captured
        .iter()
        .map(|line| seq_of(line).expect("each line ends in seq=<i>"))
        .collect();
    assert_eq!(
        seqs,
        (0..PRODUCED).collect::<Vec<usize>>(),
        "Quill must preserve strict record order"
    );
}
