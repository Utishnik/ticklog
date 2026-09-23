use std::env;

fn main() {
    // Mirror the backend-precedence from src/ring.rs: the split (rtrb)
    // layout is active only when `backend-rtrb` wins the re-export slot.
    let rtrb = env::var_os("CARGO_FEATURE_BACKEND_RTRB").is_some();
    let ringbuffer = env::var_os("CARGO_FEATURE_BACKEND_RINGBUFFER").is_some();
    let ringbuf = env::var_os("CARGO_FEATURE_BACKEND_RINGBUF").is_some();
    let triple = env::var_os("CARGO_FEATURE_BACKEND_TRIPLE_BUFFER").is_some();
    if rtrb && !ringbuffer && !ringbuf && !triple {
        println!("cargo:rustc-cfg=ticklog_split_ring");
    }
}
