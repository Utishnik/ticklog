//! Initialization of the logging system.
//!
//! [`crate::configure!`] initializes logging and returns a [`Guard`]. Logging stops
//! when the guard is dropped.

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::affinity;
use crate::drain::Drain;
use crate::error::TicklogError;
use crate::format::Template;
use crate::guard::Guard;
use crate::sink::LogSink;
use crate::sync::AtomicBool;
use crate::thread_buf::REGISTRY;
use crate::timestamp;

/// Minimum valid timezone offset in seconds east of UTC (UTC-12:00).
const MIN_TZ_OFFSET: i32 = -43_200;
/// Maximum valid timezone offset in seconds east of UTC (UTC+14:00).
const MAX_TZ_OFFSET: i32 = 50_400;

/// Default log-line pattern used when `configure!` does not specify `format:`.
pub(crate) const DEFAULT_LINE_PATTERN: &str = "{timestamp} {level} {file}:{line} {message}";

/// What a logging thread does when its buffer is full.
///
/// The two segmented strategies ([`NanoLog`] and [`Quill`]) rename the ring:
/// instead of growing in place they **hand the full buffer to the drain** and
/// switch to a fresh one, so a producer never has to wait for the drain to
/// catch up with *its own* backlog. Buffer memory comes from the crate-local
/// `r3` arena allocator (one region per thread, no cross-thread contention).
///
/// [`NanoLog`]: Backpressure::NanoLog
/// [`Quill`]: Backpressure::Quill
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Backpressure {
    /// Discard the record and return immediately, never blocking the caller.
    /// This is the default.
    Drop,
    /// Spin until space frees up. Never drops records, but burns CPU while the
    /// buffer stays full. Useful for debugging and tests.
    Block,
    /// NanoLog-style **bounded** buffer handoff. When the current segment is
    /// full it is handed to the drain and the thread takes a fresh one from a
    /// preallocated pool. The thread only blocks when the *whole pool* is
    /// exhausted; while blocked it does not spin idly — it formats its own
    /// handed-off segments (the drain's decoding/formatting work) into a
    /// shared line queue that the drain then writes to the sink.
    ///
    /// Memory is bounded by the pool; records are never dropped, only delayed.
    /// Implemented on the crate-local ring backend (with no `backend-*` Cargo
    /// feature enabled).
    NanoLog,
    /// Quill-style **unbounded** growing queue. Each full segment is handed to
    /// the drain and a fresh, larger arena-backed segment is allocated, so the
    /// producer never blocks and never drops. Memory grows without limit (up
    /// to OOM under sustained overproduction), trading memory for zero loss.
    ///
    /// Implemented on the crate-local ring backend (with no `backend-*` Cargo
    /// feature enabled).
    Quill,
}

/// Initializes the logging system and returns a [`Guard`].
///
/// Every field is optional. The `format` key accepts a pattern string with
/// `{field}` placeholders (`timestamp`, `level`, `file`, `line`,
/// `thread_name`, `thread_id`, `message`) with `std::fmt`-style format
/// specs (`:<8`, `:>10`, `:#x`, `.precision`, etc.).
///
/// ```no_run
/// # use ticklog::{ConsoleSink, Level, Backpressure};
/// let _guard = ticklog::configure! {
///     sink: ConsoleSink::stderr(),
///     max_level: Level::Trace,
///     backpressure: Backpressure::Drop,
///     timezone_offset: 3600,
///     drain_affinity: Some(vec![0]),
///     format: "{timestamp} [{level:>5}] {file}:{line} {message}",
/// }
/// .unwrap();
/// ```
#[macro_export]
macro_rules! configure {
    ($($key:ident : $val:expr),* $(,)?) => {{
        // `#[macro_export]` hoists these bridge macros to the calling crate's
        // root regardless of the surrounding block, so the logging macros can
        // resolve them without a path. The trade-off is that two sibling crates
        // each calling `configure!` would collide on these names at link time.
        // That cannot happen in practice because `configure!` is one-shot per
        // process and panics on a second call.
        #[allow(non_local_definitions)]
        #[macro_export]
        macro_rules! __ticklog_max_level {
            () => { $crate::configure!(__pick max_level { $($key : $val ,)* }) };
        }
        #[allow(non_local_definitions)]
        #[macro_export]
        macro_rules! __ticklog_backpressure {
            () => { $crate::configure!(__pick backpressure { $($key : $val ,)* }) };
        }
        #[allow(non_local_definitions)]
        #[macro_export]
        macro_rules! __ticklog_ring_capacity {
            () => { $crate::configure!(__pick ring_capacity { $($key : $val ,)* }) };
        }
        $crate::__private::__configure_rt(
            Box::new($crate::configure!(__pick sink { $($key : $val ,)* })),
            $crate::configure!(__pick timezone_offset { $($key : $val ,)* }),
            $crate::configure!(__pick drain_affinity { $($key : $val ,)* }),
            $crate::configure!(__pick ring_capacity { $($key : $val ,)* }),
            $crate::configure!(__pick format { $($key : $val ,)* }),
            $crate::configure!(__pick backpressure { $($key : $val ,)* }),
        )
    }};

    // __pick max_level
    (__pick max_level { max_level: $val:expr, $($rest:tt)* }) => { $val };
    (__pick max_level { $_other:ident : $_val:expr, $($rest:tt)* }) => {
        $crate::configure!(__pick max_level { $($rest)* })
    };
    (__pick max_level { }) => { $crate::Level::Info };

    // __pick backpressure
    (__pick backpressure { backpressure: $val:expr, $($rest:tt)* }) => { $val };
    (__pick backpressure { $_other:ident : $_val:expr, $($rest:tt)* }) => {
        $crate::configure!(__pick backpressure { $($rest)* })
    };
    (__pick backpressure { }) => { $crate::Backpressure::Drop };

    // __pick sink
    (__pick sink { sink: $val:expr, $($rest:tt)* }) => { $val };
    (__pick sink { $_other:ident : $_val:expr, $($rest:tt)* }) => {
        $crate::configure!(__pick sink { $($rest)* })
    };
    (__pick sink { }) => { $crate::ConsoleSink::stderr() };

    // __pick timezone_offset
    (__pick timezone_offset { timezone_offset: $val:expr, $($rest:tt)* }) => { $val };
    (__pick timezone_offset { $_other:ident : $_val:expr, $($rest:tt)* }) => {
        $crate::configure!(__pick timezone_offset { $($rest)* })
    };
    (__pick timezone_offset { }) => { 0i32 };

    // __pick drain_affinity
    (__pick drain_affinity { drain_affinity: $val:expr, $($rest:tt)* }) => { $val };
    (__pick drain_affinity { $_other:ident : $_val:expr, $($rest:tt)* }) => {
        $crate::configure!(__pick drain_affinity { $($rest)* })
    };
    (__pick drain_affinity { }) => { None::<Vec<usize>> };

    // __pick ring_capacity
    (__pick ring_capacity { ring_capacity: $val:expr, $($rest:tt)* }) => { ($val) as usize };
    (__pick ring_capacity { $_other:ident : $_val:expr, $($rest:tt)* }) => {
        $crate::configure!(__pick ring_capacity { $($rest)* })
    };
    (__pick ring_capacity { }) => { $crate::__private::DEFAULT_RING_SIZE };

    // __pick format
    (__pick format { format: $val:expr, $($rest:tt)* }) => { $val };
    (__pick format { $_other:ident : $_val:expr, $($rest:tt)* }) => {
        $crate::configure!(__pick format { $($rest)* })
    };
    (__pick format { }) => { "" };
}

/// Runtime portion of [`configure!`]: spawns the drain, calibrates the clock,
/// claims the ring registry, and (for the segmented policies) builds the
/// arena-backed buffer pool.
#[doc(hidden)]
pub fn __configure_rt(
    sink: Box<dyn LogSink>,
    timezone_offset: i32,
    drain_affinity: Option<Vec<usize>>,
    ring_capacity: usize,
    format_str: impl Into<String>,
    backpressure: Backpressure,
) -> Result<Guard, TicklogError> {
    if !(MIN_TZ_OFFSET..=MAX_TZ_OFFSET).contains(&timezone_offset) {
        return Err(TicklogError::InvalidTimezoneOffset(timezone_offset));
    }

    // Reject an invalid ring capacity before claiming any global resources.
    if !ring_capacity.is_power_of_two() || ring_capacity < crate::ring::SLOT_SIZE {
        return Err(TicklogError::InvalidRingCapacity(ring_capacity));
    }

    // Parse the log-line pattern before claiming any global resources, so an
    // invalid pattern rejects cleanly without leaving side-effects.
    let format_str: String = format_str.into();
    let pattern_str = if format_str.is_empty() {
        DEFAULT_LINE_PATTERN
    } else {
        &format_str
    };
    let line_pattern = Template::parse(pattern_str).map_err(TicklogError::InvalidFormatPattern)?;

    let segmented = matches!(backpressure, Backpressure::NanoLog | Backpressure::Quill);
    // The segmented policies depend on the crate-local (non-FIFO) ring backend.
    // Under a `backend-*` feature they degrade to `Block`: the semantics are
    // still "never drop", just without segment handoff.
    let backpressure = if segmented && cfg!(feature = "fifo-backend") {
        Backpressure::Block
    } else {
        backpressure
    };

    // Must be set before any producer thread can log, and before the drain
    // thread starts polling (it consults it on every pass in segmented mode).
    let _ = crate::thread_buf::BACKPRESSURE.set(backpressure);

    REGISTRY
        .set(Mutex::new(Vec::new()))
        .map_err(|_| TicklogError::AlreadyInitialized)?;

    let calibration = timestamp::calibrate();

    // Build the segmented buffer machinery before the drain starts so a fast
    // producer cannot race an uninitialized pool. A side effect of this call:
    // the pool owns the `r3` arena that all segment buffers are carved from.
    // Under a `backend-*` feature the module (and this block) is not compiled:
    // NanoLog/Quill degrade to Block above, so no pool is ever installed.
    #[cfg(not(feature = "fifo-backend"))]
    if segmented {
        let pool = crate::segments::Segments::init(
            backpressure,
            ring_capacity,
            timezone_offset,
            calibration,
            line_pattern.clone(),
        );
        crate::segments::SEGMENTS
            .set(pool)
            .map_err(|_| TicklogError::AlreadyInitialized)?;
    }

    let shutdown = Arc::new(AtomicBool::new(false));
    let drain = Drain::new(
        sink,
        timezone_offset,
        Arc::clone(&shutdown),
        calibration,
        line_pattern,
        segmented && !cfg!(feature = "fifo-backend"),
    );

    // The capacity is validated above and REGISTRY succeeded, so this set
    // always wins on the first (and only) configure call.
    let _ = crate::thread_buf::RING_CAPACITY.set(ring_capacity);

    let drain_affinity_opt = drain_affinity.clone();
    #[cfg(not(ticklog_loom))]
    let handle = thread::Builder::new()
        .name("ticklog-drain".to_string())
        .spawn(move || {
            if let Some(ref cores) = drain_affinity_opt {
                affinity::pin_thread(cores);
            }
            let mut drain = drain;
            drain.run();
        })
        .map_err(TicklogError::DrainSpawnFailed)?;
    // Under loom, `configure!` is not exercised by model tests; keep a
    // placeholder path so the module still compiles with loom atomics.
    #[cfg(ticklog_loom)]
    let handle = {
        let _ = drain_affinity_opt;
        let _ = drain;
        loom::thread::spawn(|| {})
    };

    Ok(Guard::new(handle, shutdown))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sink::ConsoleSink;

    #[test]
    fn backpressure_discriminants() {
        assert_eq!(Backpressure::Drop as u8, 0);
        assert_eq!(Backpressure::Block as u8, 1);
        assert_eq!(Backpressure::NanoLog as u8, 2);
        assert_eq!(Backpressure::Quill as u8, 3);
    }

    #[test]
    fn configure_rt_rejects_out_of_range_timezone_offset() {
        assert!(matches!(
            __configure_rt(
                Box::new(ConsoleSink::stderr()),
                50_401,
                None,
                crate::ring::DEFAULT_RING_SIZE,
                "",
                Backpressure::Drop
            ),
            Err(TicklogError::InvalidTimezoneOffset(50_401))
        ));
        assert!(matches!(
            __configure_rt(
                Box::new(ConsoleSink::stderr()),
                -43_201,
                None,
                crate::ring::DEFAULT_RING_SIZE,
                "",
                Backpressure::Drop
            ),
            Err(TicklogError::InvalidTimezoneOffset(-43_201))
        ));
    }

    #[test]
    fn configure_rt_already_initialized() {
        let _ = REGISTRY.set(Mutex::new(Vec::new()));
        let result = __configure_rt(
            Box::new(ConsoleSink::stderr()),
            0,
            None,
            crate::ring::DEFAULT_RING_SIZE,
            "",
            Backpressure::Drop,
        );
        assert!(matches!(result, Err(TicklogError::AlreadyInitialized)));
    }

    #[test]
    fn configure_rt_rejects_invalid_format_pattern() {
        // We need REGISTRY unset for this to reach the parse step; use an
        // invalid pattern to trigger the error.
        let result = __configure_rt(
            Box::new(ConsoleSink::stderr()),
            0,
            None,
            crate::ring::DEFAULT_RING_SIZE,
            "{unknown_field}",
            Backpressure::Drop,
        );
        assert!(matches!(result, Err(TicklogError::InvalidFormatPattern(_))));
    }

    #[test]
    fn configure_rt_rejects_invalid_ring_capacity() {
        // Not a power of two.
        assert!(matches!(
            __configure_rt(
                Box::new(ConsoleSink::stderr()),
                0,
                None,
                1_000_000,
                "{}",
                Backpressure::Drop
            ),
            Err(TicklogError::InvalidRingCapacity(1_000_000))
        ));
        // Smaller than the minimum slot size.
        assert!(matches!(
            __configure_rt(
                Box::new(ConsoleSink::stderr()),
                0,
                None,
                8,
                "{}",
                Backpressure::Drop
            ),
            Err(TicklogError::InvalidRingCapacity(8))
        ));
    }
}
