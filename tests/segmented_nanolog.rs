//! NanoLog-style segmented policy: bounded pool, guaranteed delivery through
//! the cooperative helper path, and zero loss under pool exhaustion.
//!
//! `Backpressure::NanoLog` keeps an 8 MiB budget of pooled segments. When a
//! producer exhausts the pool while the drain is stalled, it does the drain's
//! work on its own handed-off segments (formatting them into the shared line
//! queue, then recycling them), so records are never dropped — only delayed.
//!
//! With the default 1 MiB rings the pool holds 8 segments of 16384 single-slot
//! records each (8 * 16384 = 131072), so `PRODUCED` = 200_000 forces the
//! helper path for the last ~69k records while the gate keeps the drain from
//! recycling anything itself.
//!
//! Because helper-formatted lines and the drain's inline output share one
//! queue, strict across-handoff order is NOT guaranteed here (see the
//! `segments` module docs); the test asserts the total count and the multiset
//! of sequences instead.
//!
//! This runs only on the crate-local ring: under a `backend-*` feature the
//! segmented policies degrade to `Block`, so the pool is never installed.
//!
//! Each integration test is its own binary, so this file's single `configure!`
//! owns the process-global registry for the whole run.

use std::io;
use std::sync::{Arc, Condvar, Mutex};

use ticklog::{Backpressure, Level, LogSink, info};

/// Enough records to exhaust the whole pool (131072 slots) and keep going, so
/// the helper path is exercised for a large tail of the burst.
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
fn nanolog_never_drops_and_delivers_every_sequence_under_pool_exhaustion() {
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
        backpressure: Backpressure::NanoLog,
    }
    .expect("first configure in a fresh process must succeed");

    // The drain is stalled on the gate, so once the 8 MiB pool is exhausted
    // the producer must format its own handed-off segments to free spares. If
    // the bound leaked a record or deadlocked, this loop or the assertions
    // below would catch it: NanoLog's contract is "never drop, only delay".
    for i in 0..PRODUCED {
        info!("seq={}", i as u64);
    }

    // Let the drain deliver everything (inline segments + the helper queue),
    // then flush on drop.
    gate.open();
    drop(guard);

    let captured = lines.lock().unwrap();

    // Zero loss, even with the pool exhausted and the drain stalled the whole
    // burst: every record is formatted exactly once, by the helper or drain.
    assert_eq!(
        captured.len(),
        PRODUCED,
        "NanoLog must lose nothing under pool exhaustion, got {} of {}",
        captured.len(),
        PRODUCED
    );

    // The helper path may interleave with the drain's inline output (shared
    // line queue), so only the multiset is order-independent.
    let mut seqs: Vec<usize> = captured
        .iter()
        .map(|line| seq_of(line).expect("each line ends in seq=<i>"))
        .collect();
    seqs.sort_unstable();
    assert_eq!(
        seqs,
        (0..PRODUCED).collect::<Vec<usize>>(),
        "NanoLog must deliver every sequence exactly once"
    );
}
