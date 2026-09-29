//! Fast-path sinks for pipelines that do not need rendered log lines.
//!
//! The drain normally formats every record (pattern rendering, timestamp
//! conversion, thread snapshot) before handing a line to the sink. Sinks here
//! implement [`RawLogSink`], which switches the whole pipeline to the raw fast
//! path: records are validated and tail-advanced exactly as before, but
//! rendering is skipped entirely.
//!
//! - [`NullSink`] does literally nothing per record. It is the baseline for
//!   throughput benchmarks (the pipeline, not the sink, is what is measured)
//!   and keeps state at zero so nothing on the drain thread can contend with
//!   producers.
//! - [`InMemorySink`] captures raw records behind an [`InMemoryHandle`] for
//!   tests. It can also run as a plain formatted sink (e.g. inside a
//!   [`FanOut`](crate::FanOut), which always takes the formatted path), in
//!   which case it captures rendered lines instead.

use std::collections::VecDeque;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

use super::{LogSink, RawLogSink};
use crate::level::Level;

/// A sink that discards every record with no work, no state, and no
/// synchronization.
///
/// Both paths (`accept` and `accept_raw`) are `Ok(())` no-ops, and
/// [`raw_sink`](LogSink::raw_sink) opts into the drain's raw fast path, so the
/// drain thread performs only ring validation and tail advancement — nothing
/// else. Use it to measure the logging pipeline itself, or when log output is
/// intentionally discarded (e.g. a latency probe with all sinks disabled).
///
/// ```
/// use ticklog::{Level, LogSink, NullSink};
///
/// let mut sink = NullSink::new();
/// sink.accept(b"discarded", Level::Info).unwrap();
/// ```
///
/// Counting what reached a `NullSink` is deliberately not its job: assert
/// delivery with [`InMemorySink`], or (for a pure-drain test) by reading the
/// ring's advanced tail.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NullSink;

impl NullSink {
    /// Creates a null sink.
    pub const fn new() -> Self {
        Self
    }
}

impl LogSink for NullSink {
    #[inline(always)]
    fn accept(&mut self, _line: &[u8], _level: Level) -> io::Result<()> {
        Ok(())
    }

    #[inline(always)]
    fn raw_sink(&mut self) -> Option<&mut dyn RawLogSink> {
        Some(self)
    }
}

impl RawLogSink for NullSink {
    #[inline(always)]
    fn accept_raw(&mut self, _record: &[u8], _level: Level) -> io::Result<()> {
        Ok(())
    }
}

/// What a bounded [`InMemorySink`] does when it is full.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Overflow {
    /// Keep capturing regardless of capacity (the default for
    /// [`InMemorySink::new`]; `capacity` is ignored).
    #[default]
    Unbounded,
    /// At capacity, discard the incoming record or line and count it as
    /// dropped. Retains the earliest captures.
    DropNewest,
    /// At capacity, discard the oldest capture and count it as dropped so the
    /// incoming one fits. Retains the most recent captures.
    DropOldest,
}

/// One raw record captured by an [`InMemorySink`]: the level the drain parsed
/// and the record's full wire bytes (header plus payload).
///
/// Accessors read fixed offsets from the wire header and return `0` when the
/// captured buffer is shorter than the field (only reachable by pushing short
/// buffers directly through [`RawLogSink::accept_raw`]; records delivered by
/// the drain are always complete).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CapturedRecord {
    level: Level,
    bytes: Vec<u8>,
}

impl CapturedRecord {
    /// The level the drain parsed from the record header.
    pub fn level(&self) -> Level {
        self.level
    }

    /// The complete wire bytes of the record.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The header's `total_size` field (wire bytes 2..4, little-endian).
    pub fn total_size(&self) -> u16 {
        le_u16(&self.bytes, 2)
    }

    /// The header's `flags` field (wire bytes 5..7, little-endian).
    pub fn flags(&self) -> u16 {
        le_u16(&self.bytes, 5)
    }

    /// The producer's raw timestamp in clock ticks (wire bytes 8..16,
    /// little-endian); not converted to wall-clock time on this path.
    pub fn timestamp_ticks(&self) -> u64 {
        le_u64(&self.bytes, 8)
    }
}

// Reads a little-endian u16 at `offset`, or 0 when out of range.
fn le_u16(bytes: &[u8], offset: usize) -> u16 {
    match bytes.get(offset..offset + 2) {
        Some(b) => u16::from_le_bytes([b[0], b[1]]),
        None => 0,
    }
}

// Reads a little-endian u64 at `offset`, or 0 when out of range.
fn le_u64(bytes: &[u8], offset: usize) -> u64 {
    match bytes.get(offset..offset + 8) {
        Some(b) => u64::from_le_bytes(b.try_into().expect("slice is 8 bytes")),
        None => 0,
    }
}

/// State shared between an [`InMemorySink`] and its [`InMemoryHandle`].
struct Shared {
    state: Mutex<State>,
    capacity: usize,
    overflow: Overflow,
}

struct State {
    records: VecDeque<CapturedRecord>,
    lines: VecDeque<(Vec<u8>, Level)>,
    dropped_records: u64,
    dropped_lines: u64,
}

impl Shared {
    /// Locks the shared state, tolerating a poisoned mutex: a test that
    /// panics while inspecting captured data must not permanently break later
    /// log calls in the same process.
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn push_record(&self, record: &[u8], level: Level) {
        let mut st = self.lock();
        match self.overflow {
            Overflow::Unbounded => {}
            _ => {
                if st.records.len() >= self.capacity {
                    match self.overflow {
                        Overflow::DropNewest => {
                            st.dropped_records += 1;
                            return;
                        }
                        Overflow::DropOldest => {
                            st.records.pop_front();
                            st.dropped_records += 1;
                        }
                        Overflow::Unbounded => unreachable!("handled above"),
                    }
                }
            }
        }
        st.records.push_back(CapturedRecord {
            level,
            bytes: record.to_vec(),
        });
    }

    fn push_line(&self, line: &[u8], level: Level) {
        let mut st = self.lock();
        match self.overflow {
            Overflow::Unbounded => {}
            _ => {
                if st.lines.len() >= self.capacity {
                    match self.overflow {
                        Overflow::DropNewest => {
                            st.dropped_lines += 1;
                            return;
                        }
                        Overflow::DropOldest => {
                            st.lines.pop_front();
                            st.dropped_lines += 1;
                        }
                        Overflow::Unbounded => unreachable!("handled above"),
                    }
                }
            }
        }
        st.lines.push_back((line.to_vec(), level));
    }
}

/// A test sink that captures log records behind a cheaply cloneable
/// [`InMemoryHandle`].
///
/// In the normal (root) configuration the drain hands it **raw records** via
/// [`RawLogSink`], so no line formatting happens at all —
/// [`handle`](InMemorySink::handle) then exposes [`records`](InMemoryHandle::records).
/// When the sink ends up on the formatted path instead (inside a
/// [`FanOut`](crate::FanOut)), it captures rendered **lines** —
/// [`lines`](InMemoryHandle::lines). A single configuration never mixes the
/// two, so `records_len()` and `lines_len()` together tell you which path the
/// drain took.
///
/// The handle shares state with the sink: clone it *before* handing the sink
/// to [`configure!`](crate::configure).
///
/// ```
/// use ticklog::{configure, InMemorySink, Level};
///
/// let sink = InMemorySink::new();
/// let handle = sink.handle();
/// let guard = configure! { sink: sink, max_level: Level::Trace }.unwrap();
/// ticklog::info!("captured {}", 1);
/// drop(guard);
/// assert_eq!(handle.records_len(), 1);
/// ```
///
/// Records and lines are captured in arrival order per sink. Capacity is
/// per-store: a sink that somehow fills both stores applies `capacity` to
/// each independently.
pub struct InMemorySink {
    shared: Arc<Shared>,
}

impl InMemorySink {
    /// Creates an unbounded sink: every record is retained until
    /// [`clear`](InMemoryHandle::clear).
    pub fn new() -> Self {
        Self::with_capacity(usize::MAX, Overflow::Unbounded)
    }

    /// Creates a bounded sink with a maximum of `capacity` captures per store
    /// (records and lines), applying `overflow` when full.
    ///
    /// # Panics
    ///
    /// Panics when `capacity` is 0 (a zero-capacity capture sink cannot make
    /// progress under any overflow policy).
    pub fn bounded(capacity: usize, overflow: Overflow) -> Self {
        assert!(capacity > 0, "InMemorySink capacity must be greater than 0");
        Self::with_capacity(capacity, overflow)
    }

    fn with_capacity(capacity: usize, overflow: Overflow) -> Self {
        Self {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    records: VecDeque::new(),
                    lines: VecDeque::new(),
                    dropped_records: 0,
                    dropped_lines: 0,
                }),
                capacity,
                overflow,
            }),
        }
    }

    /// Returns a handle sharing this sink's captured data. Clone it before
    /// the sink is moved into [`configure!`](crate::configure).
    pub fn handle(&self) -> InMemoryHandle {
        InMemoryHandle {
            shared: Arc::clone(&self.shared),
        }
    }
}

impl Default for InMemorySink {
    fn default() -> Self {
        Self::new()
    }
}

impl LogSink for InMemorySink {
    fn accept(&mut self, line: &[u8], level: Level) -> io::Result<()> {
        self.shared.push_line(line, level);
        Ok(())
    }

    fn raw_sink(&mut self) -> Option<&mut dyn RawLogSink> {
        Some(self)
    }
}

impl RawLogSink for InMemorySink {
    fn accept_raw(&mut self, record: &[u8], level: Level) -> io::Result<()> {
        self.shared.push_record(record, level);
        Ok(())
    }
}

/// A cloneable view of an [`InMemorySink`]'s captured data.
///
/// All accessors lock the shared state only for the duration of the copy, so
/// inspecting the sink never deadlocks the drain thread for long and never
/// requires stopping the logging pipeline.
#[derive(Clone)]
pub struct InMemoryHandle {
    shared: Arc<Shared>,
}

impl InMemoryHandle {
    /// Number of raw records captured so far.
    pub fn records_len(&self) -> usize {
        self.shared.lock().records.len()
    }

    /// Number of formatted lines captured so far.
    pub fn lines_len(&self) -> usize {
        self.shared.lock().lines.len()
    }

    /// True when neither records nor lines have been captured.
    pub fn is_empty(&self) -> bool {
        let st = self.shared.lock();
        st.records.is_empty() && st.lines.is_empty()
    }

    /// Copies out every captured record, in arrival order.
    pub fn records(&self) -> Vec<CapturedRecord> {
        self.shared.lock().records.iter().cloned().collect()
    }

    /// Copies out every captured formatted line, in arrival order.
    pub fn lines(&self) -> Vec<(Vec<u8>, Level)> {
        self.shared.lock().lines.iter().cloned().collect()
    }

    /// The levels of the captured raw records, in arrival order.
    pub fn levels(&self) -> Vec<Level> {
        self.shared.lock().records.iter().map(|r| r.level).collect()
    }

    /// Captured formatted lines decoded as UTF-8 (lossy), in arrival order.
    pub fn line_strings(&self) -> Vec<String> {
        self.shared
            .lock()
            .lines
            .iter()
            .map(|(l, _)| String::from_utf8_lossy(l).into_owned())
            .collect()
    }

    /// Records discarded by the [`Overflow`] policy.
    pub fn dropped_records(&self) -> u64 {
        self.shared.lock().dropped_records
    }

    /// Lines discarded by the [`Overflow`] policy (formatted path only).
    pub fn dropped_lines(&self) -> u64 {
        self.shared.lock().dropped_lines
    }

    /// Empties both stores and resets the drop counters.
    pub fn clear(&self) {
        let mut st = self.shared.lock();
        st.records.clear();
        st.lines.clear();
        st.dropped_records = 0;
        st.dropped_lines = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::record::{HEADER_SIZE, LOG_RECORD, VERSION};

    /// Builds a wire record of `total` bytes with the given level, flags, and
    /// timestamp. Payload bytes are filled with `0xAB` so captures can be
    /// compared for exactness.
    fn wire_record(total: usize, level: Level, flags: u16, timestamp: u64) -> Vec<u8> {
        assert!(total >= HEADER_SIZE);
        let mut b = vec![0u8; total];
        b[0] = VERSION;
        b[1] = LOG_RECORD;
        b[2..4].copy_from_slice(&(total as u16).to_le_bytes());
        b[4] = level as u8;
        b[5..7].copy_from_slice(&flags.to_le_bytes());
        b[7] = 0;
        b[8..16].copy_from_slice(&timestamp.to_le_bytes());
        for byte in &mut b[HEADER_SIZE..] {
            *byte = 0xAB;
        }
        b
    }

    #[test]
    fn null_sink_accepts_formatted_and_raw() {
        let mut sink = NullSink::new();
        assert!(sink.accept(b"line", Level::Info).is_ok());
        assert!(sink.accept_raw(&[1, 2, 3], Level::Error).is_ok());
        assert!(sink.flush().is_ok());
    }

    #[test]
    fn null_sink_opts_into_the_raw_path() {
        let mut sink = NullSink::new();
        assert!(sink.raw_sink().is_some());
    }

    #[test]
    fn null_sink_is_send_and_static() {
        fn assert_send<T: Send>() {}
        fn assert_static<T: 'static>() {}
        assert_send::<NullSink>();
        assert_static::<NullSink>();
    }

    #[test]
    fn in_memory_opts_into_the_raw_path() {
        let mut sink = InMemorySink::new();
        assert!(sink.raw_sink().is_some());
    }

    #[test]
    fn in_memory_raw_accept_captures_record_not_line() {
        let mut sink = InMemorySink::new();
        let handle = sink.handle();
        let record = wire_record(32, Level::Warn, 0x01, 77);
        sink.accept_raw(&record, Level::Warn).unwrap();
        assert_eq!(handle.records_len(), 1);
        assert_eq!(handle.lines_len(), 0, "raw path must not capture lines");
        assert_eq!(handle.records()[0].as_bytes(), &record);
        assert_eq!(handle.levels(), vec![Level::Warn]);
    }

    #[test]
    fn in_memory_formatted_accept_captures_line_not_record() {
        let mut sink = InMemorySink::new();
        let handle = sink.handle();
        sink.accept(b"rendered line", Level::Info).unwrap();
        assert_eq!(handle.lines_len(), 1);
        assert_eq!(
            handle.records_len(),
            0,
            "formatted path must not capture records"
        );
        assert_eq!(handle.line_strings(), vec!["rendered line".to_string()]);
        assert_eq!(handle.lines()[0].1, Level::Info);
    }

    #[test]
    fn in_memory_new_is_unbounded() {
        let mut sink = InMemorySink::new();
        let handle = sink.handle();
        for i in 0..1000 {
            let record = wire_record(24, Level::Debug, 0, i as u64);
            sink.accept_raw(&record, Level::Debug).unwrap();
        }
        assert_eq!(handle.records_len(), 1000);
        assert_eq!(handle.dropped_records(), 0);
    }

    #[test]
    fn in_memory_drop_newest_keeps_earliest() {
        let mut sink = InMemorySink::bounded(3, Overflow::DropNewest);
        let handle = sink.handle();
        for i in 0..5u64 {
            sink.accept_raw(&wire_record(24, Level::Info, 0, i), Level::Info)
                .unwrap();
        }
        assert_eq!(handle.records_len(), 3);
        assert_eq!(handle.dropped_records(), 2);
        let stamps: Vec<u64> = handle
            .records()
            .iter()
            .map(|r| r.timestamp_ticks())
            .collect();
        assert_eq!(stamps, vec![0, 1, 2], "newest records were dropped");
    }

    #[test]
    fn in_memory_drop_oldest_keeps_latest() {
        let mut sink = InMemorySink::bounded(3, Overflow::DropOldest);
        let handle = sink.handle();
        for i in 0..5u64 {
            sink.accept_raw(&wire_record(24, Level::Info, 0, i), Level::Info)
                .unwrap();
        }
        assert_eq!(handle.records_len(), 3);
        assert_eq!(handle.dropped_records(), 2);
        let stamps: Vec<u64> = handle
            .records()
            .iter()
            .map(|r| r.timestamp_ticks())
            .collect();
        assert_eq!(stamps, vec![2, 3, 4], "oldest records were dropped");
    }

    #[test]
    fn in_memory_bounded_lines_overflow_independently() {
        let mut sink = InMemorySink::bounded(2, Overflow::DropNewest);
        let handle = sink.handle();
        sink.accept(b"one", Level::Info).unwrap();
        sink.accept(b"two", Level::Info).unwrap();
        sink.accept(b"three", Level::Info).unwrap();
        assert_eq!(handle.lines_len(), 2);
        assert_eq!(handle.dropped_lines(), 1);
        assert_eq!(handle.records_len(), 0);
        assert_eq!(handle.dropped_records(), 0);
    }

    #[test]
    fn in_memory_unbounded_bounded_constructor_ignores_capacity() {
        let mut sink = InMemorySink::bounded(2, Overflow::Unbounded);
        let handle = sink.handle();
        for i in 0..10u64 {
            sink.accept_raw(&wire_record(24, Level::Info, 0, i), Level::Info)
                .unwrap();
        }
        assert_eq!(handle.records_len(), 10, "Unbounded ignores capacity");
        assert_eq!(handle.dropped_records(), 0);
    }

    #[test]
    #[should_panic(expected = "capacity must be greater than 0")]
    fn in_memory_zero_capacity_panics() {
        let _ = InMemorySink::bounded(0, Overflow::DropNewest);
    }

    #[test]
    fn captured_record_reads_wire_header_fields() {
        let record = wire_record(40, Level::Error, 0x0102, 0x0102_0304_0506_0708);
        let mut sink = InMemorySink::new();
        let handle = sink.handle();
        sink.accept_raw(&record, Level::Error).unwrap();
        let captured = &handle.records()[0];
        assert_eq!(captured.level(), Level::Error);
        assert_eq!(captured.as_bytes().len(), 40);
        assert_eq!(captured.total_size(), 40);
        assert_eq!(captured.flags(), 0x0102);
        assert_eq!(captured.timestamp_ticks(), 0x0102_0304_0506_0708);
    }

    #[test]
    fn captured_record_short_buffer_reads_zero() {
        let mut sink = InMemorySink::new();
        let handle = sink.handle();
        sink.accept_raw(&[1], Level::Info).unwrap();
        let captured = &handle.records()[0];
        assert_eq!(captured.total_size(), 0);
        assert_eq!(captured.flags(), 0);
        assert_eq!(captured.timestamp_ticks(), 0);
    }

    #[test]
    fn handle_observes_writes_after_the_sink_moves() {
        let sink = InMemorySink::new();
        let handle = sink.handle();
        fn moved(sink: InMemorySink) {
            let mut sink = sink;
            sink.accept_raw(&wire_record(24, Level::Trace, 0, 9), Level::Trace)
                .unwrap();
        }
        moved(sink);
        assert_eq!(handle.records_len(), 1, "handle shares state with the sink");
    }

    #[test]
    fn clear_resets_records_lines_and_counters() {
        let mut sink = InMemorySink::bounded(1, Overflow::DropNewest);
        let handle = sink.handle();
        sink.accept_raw(&wire_record(24, Level::Info, 0, 0), Level::Info)
            .unwrap();
        sink.accept_raw(&wire_record(24, Level::Info, 0, 1), Level::Info)
            .unwrap();
        sink.accept(b"line", Level::Info).unwrap();
        sink.accept(b"line", Level::Info).unwrap();
        assert_eq!(handle.records_len(), 1);
        assert_eq!(handle.dropped_records(), 1);
        assert_eq!(handle.lines_len(), 1);
        assert_eq!(handle.dropped_lines(), 1);
        handle.clear();
        assert!(handle.is_empty());
        assert_eq!(handle.dropped_records(), 0);
        assert_eq!(handle.dropped_lines(), 0);
    }

    #[test]
    fn default_in_memory_sink_is_unbounded() {
        assert_eq!(InMemorySink::default().handle().records_len(), 0);
    }
}
