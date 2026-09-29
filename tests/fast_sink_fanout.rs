//! `FanOut` forces the formatted path even when its children are raw-capable:
//! every child must see the same rendered input, with each child's own
//! `max_level` filter applied — the reason `FanOut` never reports a raw sink.

use ticklog::{FanOut, InMemorySink, Level, LogSinkExt, configure, error, info, trace, warn};

#[test]
fn fan_out_falls_back_to_formatted_lines_per_child() {
    let warn_child = InMemorySink::new();
    let warn_handle = warn_child.handle();
    let all_child = InMemorySink::new();
    let all_handle = all_child.handle();

    let fan = FanOut::new()
        .add(warn_child.with_max_level(Level::Warn))
        .add(all_child);

    let guard = configure! {
        sink: fan,
        max_level: Level::Trace,
    }
    .expect("first configure in a fresh process must succeed");

    trace!("t");
    info!("i");
    warn!("w");
    error!("e");

    drop(guard);

    // The fan-out never takes the raw path: both children captured rendered
    // lines, not wire records.
    assert_eq!(all_handle.records_len(), 0, "FanOut must stay formatted");
    assert_eq!(
        all_handle.lines_len(),
        4,
        "every record reaches the unfiltered child"
    );

    // The child's WithLevel filter applies per record on the formatted path.
    assert_eq!(warn_handle.records_len(), 0);
    assert_eq!(
        warn_handle.lines_len(),
        2,
        "only WARN and ERROR pass the child's max_level"
    );
    let levels: Vec<Level> = warn_handle.lines().iter().map(|(_, l)| *l).collect();
    assert_eq!(levels, vec![Level::Warn, Level::Error]);

    // The captured lines are actual rendered text.
    let lines = all_handle.line_strings();
    assert_eq!(lines.len(), 4);
    assert!(lines[0].ends_with('t'), "trace line: {:?}", lines[0]);
    assert!(lines[1].ends_with('i'), "info line: {:?}", lines[1]);
    assert!(
        lines[1].contains(" INFO "),
        "formatted line carries its level: {:?}",
        lines[1]
    );
}
