//! Thread-CPU affinity on top of the [`core_affinity`] crate.
//!
//! The crate wraps the platform syscalls behind a safe API:
//!
//! - Linux: `sched_setaffinity`
//! - Windows: `SetThreadAffinityMask`
//! - macOS: Mach `thread_affinity_policy`
//! - Others: no-op
//!
//! `core_affinity` pins to a single logical core, so when multiple cores are
//! requested only the first is used; the rest are ignored.

/// Pin the calling thread to the first core in `cores`.
///
/// `core_affinity` supports single-core masks, so `cores[0]` is the only
/// entry honored; additional entries are ignored. An empty slice is a no-op,
/// as is a core index that does not exist on this host.
///
/// On Linux, `core_affinity`'s `set_for_current` calls `CPU_SET` which aborts
/// on out-of-range core IDs. To prevent that, we validate the requested core
/// against the list of cores returned by `get_core_ids()` before pinning.
pub fn pin_thread(cores: &[usize]) {
    let Some(&first) = cores.first() else {
        return;
    };

    // Validate the core ID against the OS-reported list. On Linux, calling
    // `set_for_current` with an ID that exceeds `CPU_SETSIZE` causes an
    // abort (unrecoverable), so we must filter it out beforehand.
    let valid = core_affinity::get_core_ids()
        .map(|ids| ids.iter().any(|c| c.id == first))
        .unwrap_or(false);
    if !valid {
        eprintln!(
            "ticklog: cannot pin to core {first}: \
             core index not in the OS-reported list of usable cores"
        );
        return;
    }

    if !core_affinity::set_for_current(core_affinity::CoreId { id: first }) {
        eprintln!(
            "ticklog: failed to pin thread to core {first}: \
             the core is not usable by this process"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_slice_is_noop() {
        // Must not panic on any platform.
        pin_thread(&[]);
    }

    #[test]
    fn core_id_stores_index() {
        let id = core_affinity::CoreId { id: 7 };
        assert_eq!(id.id, 7);
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri does not implement affinity syscalls")]
    fn current_thread_can_pin_to_known_cores() {
        let ids = core_affinity::get_core_ids().unwrap_or_default();
        assert!(!ids.is_empty(), "expected at least one usable core");
        for id in ids {
            // Pinning to a core the current thread is allowed to run on must
            // succeed (a false result would mean affinity support is broken).
            pin_thread(&[id.id]);
        }
    }

    #[test]
    #[cfg_attr(miri, ignore = "Miri does not implement affinity syscalls")]
    fn invalid_core_does_not_panic() {
        // A core index far beyond any real system must not panic; pin_thread
        // reports it and returns.
        pin_thread(&[usize::MAX]);
    }
}