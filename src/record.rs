//! The on-the-wire log record format.
//!
//! Layout constants are shared by the producer (which assembles records here)
//! and the drain (which decodes them), so the two sides cannot drift. The
//! record is a fixed 16-byte header followed by flagged sections:
//!
//! ```text
//! header:  version u8 | type u8 | total_size u16 | level u8 | flags u16 | pad u8 | timestamp u64
//! site:    site_ptr u64                                          (FLAG_SITE)
//! source:  file_ptr u64 | file_len u16 | line u32                (FLAG_SOURCE, legacy wire)
//! thread:  thread_id u64 | name_len u16 | name bytes             (FLAG_THREAD, legacy wire)
//! args:    count u8 | tag u8 * count | payload bytes * count
//! ```
//!
//! The per-record cost is deliberately minimal: the record carries only a
//! pointer to a [`Site`] (a `&'static` call-site descriptor holding the format
//! string, source file, and line). The format/source strings live in the
//! binary's read-only data and are referenced by pointer, never copied, so the
//! producer writes one 8-byte pointer instead of the fmt/source sections early
//! designs embedded per record. Thread identity is not in the record at all:
//! a ring belongs to exactly one producer, so the drain keys thread id/name
//! off the ring's registration. The `FLAG_SOURCE`/`FLAG_THREAD` sections
//! remain decodable for older wire records but are not produced.

use crate::level::Level;
use core::mem::size_of;

/// Record format version written in byte 0 of every header.
pub(crate) const VERSION: u8 = 0x02;
/// Record type: a normal encoded log record.
pub(crate) const LOG_RECORD: u8 = 1;
/// Record type: filler written before a ring wrap so no record straddles the
/// physical end of the buffer.
pub(crate) const END_OF_BUFFER: u8 = 2;

/// Fixed record header size in bytes: version, type, total_size, level, flags,
/// pad, timestamp. Summed from field widths so it tracks the layout above.
pub(crate) const HEADER_SIZE: usize = size_of::<u8>()  // version
    + size_of::<u8>()   // type
    + size_of::<u16>()  // total_size
    + size_of::<u8>()   // level
    + size_of::<u16>()  // flags
    + size_of::<u8>()   // _pad
    + size_of::<u64>(); // timestamp
/// Encoded size of the site section: an 8-byte pointer to a [`Site`].
pub(crate) const SITE_SECTION_SIZE: usize = size_of::<u64>();
/// Size of the argument count byte that precedes the tags and payloads.
pub(crate) const COUNT_SIZE: usize = size_of::<u8>();

/// Total size of a record's fixed sections, before any arguments: header,
/// site, and the count byte. The logging macros hardcode this base (they
/// expand in the caller's crate and cannot read this const); a compile-time
/// assertion in `macros` guards the two against drift.
pub const BASE_RECORD_SIZE: usize = HEADER_SIZE + SITE_SECTION_SIZE + COUNT_SIZE;

/// Flag bit: the site section is present.
pub(crate) const FLAG_SITE: u16 = 0x01;
/// Flag bit: a legacy in-band source section is present.
pub(crate) const FLAG_SOURCE: u16 = 0x02;
/// Flag bit: a legacy in-band thread section is present.
pub(crate) const FLAG_THREAD: u16 = 0x04;
/// Flag bit: the process section is present.
pub(crate) const FLAG_PROCESS: u16 = 0x08;
/// Flag bit: the complex-type section is present.
pub(crate) const FLAG_COMPLEX: u16 = 0x10;

/// Largest record the u16 `total_size` header field can frame. A record that
/// would encode larger than this is dropped rather than truncated.
pub(crate) const MAX_RECORD_SIZE: usize = u16::MAX as usize;

/// Descriptor for one logging call site: the format string plus the source
/// file and line the macros captured.
///
/// The logging macros construct `&Site` via Rust's constant promotion at every
/// macro instantiation, so each call site's descriptor address is effectively
/// unique and `'static` (read-only data). The producer writes that address
/// into every record; the drain dereferences it to recover the format string,
/// file, and line without paying per-record section costs. Equality of two
/// promoted descriptors is by value, so the compiler may fold identical
/// descriptors — harmless, because the encoded content is identical.
///
/// Public only so the logging macros can construct it through `$crate`;
/// referenced via `crate::__private`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct Site {
    /// The format string literal, e.g. `"listening on {}"`.
    pub fmt: &'static str,
    /// The source file (`file!()`), e.g. `"src/main.rs"`.
    pub file: &'static str,
    /// The source line (`line!()`).
    pub line: u32,
}

/// Assembles a record by writing the fixed sections (header, site) into
/// `dst`, then delegating argument encoding to the caller's monomorphized
/// closure.
///
/// The closure receives a mutable slice starting right after the count byte,
/// sized to fit exactly `args_bytes + n_args` bytes (tags + payloads). It is
/// monomorphized per unique argument-type signature, so `Loggable::type_tag()`
/// and `Loggable::encode()` calls inside it resolve to concrete impls with no
/// vtable dispatch.
///
/// # Safety
///
/// `dst` must point to a writable region of at least `total_size` bytes. The
/// caller must guarantee that `write_args` writes exactly `n_args` tag bytes
/// followed by `args_bytes` payload bytes.
// The record fields (header, site, arg count/size, and the arg-writing
// closure) are genuinely distinct inputs to this monomorphized hot-path
// assembler; bundling them into a struct would add indirection at the single
// call site without making anything clearer.
#[allow(clippy::too_many_arguments)]
#[inline]
pub(crate) fn assemble(
    dst: *mut u8,
    level: Level,
    timestamp: u64,
    flags: u16,
    site: &'static Site,
    n_args: u8,
    total_size: usize,
    write_args: impl FnOnce(&mut [u8]),
) {
    let total = total_size as u16;
    let level_u8 = level.to_u8();

    debug_assert!(total_size <= MAX_RECORD_SIZE);

    // SAFETY: The caller guarantees `dst` points to `total_size` writable
    // bytes. Every one of those bytes is written by the stores below and the
    // caller's `write_args` closure, so no uninitialized byte is ever read.
    // The caller's documented contract guarantees `write_args` fills exactly
    // the tags + payloads region.
    unsafe {
        let buf = std::slice::from_raw_parts_mut(dst, total_size);
        // Every byte offset below is < total_size (>= HEADER_SIZE + 8 in the
        // site-layout fast path the macros always use), so each unaligned
        // 8-byte window lands inside `buf`. All pointers are derived from
        // `buf`, so no second provenance aliases it for the drain to trip on.
        let p = buf.as_mut_ptr();

        // Header, field-packed into two u64 stores instead of seven per-field
        // copies (layout: version u8 | type u8 | total_size u16 | level u8 |
        // flags u16 | _pad u8 | timestamp u64, all little-endian).
        let header0 = (VERSION as u64)
            | ((LOG_RECORD as u64) << 8)
            | ((total as u64) << 16)
            | ((level_u8 as u64) << 32)
            | ((flags as u64) << 40);
        std::ptr::write_unaligned(p.cast::<u64>(), header0.to_le());
        std::ptr::write_unaligned(p.add(8).cast::<u64>(), timestamp.to_le());
        let mut pos = 16usize;

        // Site section: the accepted fast path. The drain dereferences the
        // pointer to the promoted call-site descriptor for fmt/file/line.
        if flags & FLAG_SITE != 0 {
            std::ptr::write_unaligned(
                p.add(pos).cast::<u64>(),
                (site as *const Site as u64).to_le(),
            );
            pos += 8;
        }

        // Count byte
        *p.add(pos) = n_args;
        pos += 1;

        // Delegate to the monomorphized closure for tags + payloads.
        write_args(&mut buf[pos..]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode::Loggable;

    /// Test helper: wraps [`assemble`] with a `&[&dyn Loggable]` slice for
    /// convenience. Computes sizes from the slice and delegates to the
    /// real (monomorphized) `assemble` via a closure.
    fn check_assemble(
        scratch: &mut Vec<u8>,
        level: Level,
        timestamp: u64,
        site: &'static Site,
        args: &[&dyn Loggable],
    ) -> bool {
        if args.len() > u8::MAX as usize {
            return false;
        }

        let mut args_payload = 0usize;
        for arg in args {
            args_payload += arg.encoded_size();
        }
        let flags = FLAG_SITE;
        let total_size = HEADER_SIZE + SITE_SECTION_SIZE + COUNT_SIZE + args.len() + args_payload;
        if total_size > MAX_RECORD_SIZE {
            return false;
        }

        let n_args = args.len() as u8;
        scratch.clear();
        scratch.reserve(total_size);
        assemble(
            scratch.as_mut_ptr(),
            level,
            timestamp,
            flags,
            site,
            n_args,
            total_size,
            |buf| {
                let mut pos = 0usize;
                for arg in args {
                    buf[pos] = arg.type_tag();
                    pos += 1;
                }
                for arg in args {
                    let s = arg.encoded_size();
                    arg.encode(&mut buf[pos..pos + s]);
                    pos += s;
                }
            },
        );
        // SAFETY: `assemble` wrote exactly `total_size` bytes into the
        // Vec's allocation (reserved above); the region is initialized
        // and the Vec's capacity is sufficient.
        unsafe { scratch.set_len(total_size) };
        true
    }

    // Reads a little-endian u16 at `offset`.
    fn read_u16(bytes: &[u8], offset: usize) -> u16 {
        u16::from_le_bytes(bytes[offset..offset + 2].try_into().unwrap())
    }

    // Reads a little-endian u64 at `offset`.
    fn read_u64(bytes: &[u8], offset: usize) -> u64 {
        u64::from_le_bytes(bytes[offset..offset + 8].try_into().unwrap())
    }

    const SITE: &Site = &Site {
        fmt: "value {}",
        file: "src/x.rs",
        line: 42,
    };

    #[test]
    fn header_fields_are_written() {
        let mut buf = Vec::new();
        let ok = check_assemble(&mut buf, Level::Warn, 0xABCD, SITE, &[]);
        assert!(ok);

        assert_eq!(buf[0], VERSION);
        assert_eq!(buf[1], LOG_RECORD);
        assert_eq!(read_u16(&buf, 2) as usize, buf.len());
        assert_eq!(buf[4], Level::Warn.to_u8());
        assert_eq!(read_u16(&buf, 5), FLAG_SITE);
        assert_eq!(buf[7], 0);
        assert_eq!(read_u64(&buf, 8), 0xABCD);
    }

    #[test]
    fn header_packed_first_word_is_byte_exact() {
        // Exercises the two-u64 packing: every byte of the first header word
        // derives from a distinct field, so a wrong shift or OR would show up
        // exactly here. A flag word with both LE bytes non-zero is only
        // reachable by driving `assemble` directly (the flag constants all fit
        // the low byte).
        let flags = 0x12ABu16; // includes FLAG_SITE so the site section runs
        let total_size = HEADER_SIZE + SITE_SECTION_SIZE + COUNT_SIZE;
        let mut buf = vec![0u8; total_size];
        assemble(
            buf.as_mut_ptr(),
            Level::Debug,
            0x0203_0405_0607_0809,
            flags,
            SITE,
            0,
            total_size,
            |_| {},
        );

        let total = buf.len() as u16;
        assert_eq!(buf[0], VERSION);
        assert_eq!(buf[1], LOG_RECORD);
        assert_eq!(buf[2], total.to_le_bytes()[0]);
        assert_eq!(buf[3], total.to_le_bytes()[1]);
        assert_eq!(buf[4], Level::Debug.to_u8());
        assert_eq!(buf[5], 0xAB); // flags low byte
        assert_eq!(buf[6], 0x12); // flags high byte
        assert_eq!(buf[7], 0x00); // _pad
        assert_eq!(read_u64(&buf, 8), 0x0203_0405_0607_0809);
        assert_eq!(read_u64(&buf, 16), SITE as *const Site as u64);
        assert_eq!(buf[24], 0); // count byte right after the site section
    }

    #[test]
    fn site_section_references_the_call_site_descriptor() {
        let mut buf = Vec::new();
        check_assemble(&mut buf, Level::Info, 0, SITE, &[&1u64]);

        // The 8-byte site pointer sits right after the header.
        let site_ptr = read_u64(&buf, HEADER_SIZE);
        assert_eq!(site_ptr, SITE as *const Site as u64);

        // The count byte follows the site section.
        let count_at = HEADER_SIZE + SITE_SECTION_SIZE;
        assert_eq!(buf[count_at], 1);
    }

    #[test]
    fn arguments_are_tag_grouped_then_payload_grouped() {
        let mut buf = Vec::new();
        // Two args: u16 (tag 0x06, 2 bytes) then bool (tag 0x0A, 1 byte).
        check_assemble(
            &mut buf,
            Level::Info,
            0,
            &Site {
                fmt: "{} {}",
                file: "f",
                line: 1,
            },
            &[&0x1234u16, &true],
        );

        let args_at = HEADER_SIZE + SITE_SECTION_SIZE;
        assert_eq!(buf[args_at], 2); // count
        assert_eq!(buf[args_at + 1], 0x06); // u16 tag
        assert_eq!(buf[args_at + 2], 0x0A); // bool tag
        // Payloads follow the two tags.
        assert_eq!(read_u16(&buf, args_at + 3), 0x1234);
        assert_eq!(buf[args_at + 5], 1); // bool true
    }

    #[test]
    fn total_size_matches_buffer_length() {
        let mut buf = Vec::new();
        check_assemble(&mut buf, Level::Error, 0, SITE, &[&"hello"]);
        assert_eq!(read_u16(&buf, 2) as usize, buf.len());
    }

    #[test]
    fn rejects_more_than_255_arguments() {
        let args: Vec<&dyn Loggable> = (0..256).map(|_| &1u8 as &dyn Loggable).collect();
        let mut buf = Vec::new();
        assert!(!check_assemble(&mut buf, Level::Info, 0, SITE, &args));
    }

    #[test]
    fn rejects_record_larger_than_u16_total_size() {
        // A string argument just large enough to push total_size past u16::MAX.
        let big = "x".repeat(u16::MAX as usize);
        let mut buf = Vec::new();
        assert!(!check_assemble(
            &mut buf,
            Level::Info,
            0,
            SITE,
            &[&big.as_str()]
        ));
    }

    #[test]
    fn zero_args_writes_count_zero() {
        let mut buf = Vec::new();
        check_assemble(&mut buf, Level::Info, 0, SITE, &[]);
        let args_at = HEADER_SIZE + SITE_SECTION_SIZE;
        assert_eq!(buf[args_at], 0);
        assert_eq!(buf.len(), args_at + 1);
    }
}
