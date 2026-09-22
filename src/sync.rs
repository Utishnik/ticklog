//! Atomic-type shim: under `cfg(ticklog_loom)` the model checker must see
//! every atomic operation, so production atomics re-export loom's types when
//! the loom cfg is on and `std`'s otherwise. Mutex/Condvar stay on `std` for
//! now — loom tests target the lock-free ring/shutdown protocols, not the
//! registry critical section.
//!
//! Enable with `RUSTFLAGS="--cfg ticklog_loom"` (see `loom_tests` docs).

#[cfg(ticklog_loom)]
pub(crate) use loom::sync::atomic::{AtomicBool, AtomicU64, Ordering};
#[cfg(not(ticklog_loom))]
pub(crate) use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
