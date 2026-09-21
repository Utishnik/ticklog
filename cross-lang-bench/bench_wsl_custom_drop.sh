#!/usr/bin/env bash
# WSL custom-ring Drop (policy-drop) sweep — byte-mirror of the WINDOWS
#   results/windows/sweep/ticklog_custom_drop.json run.
# CONTRACT (these are the ONLY json fields that differ between the two os):
#   os            : this file records "linux" (windows file records "windows")
#   ns_per_tick   : 0.313097  (calibrated in WSL2 on this same physical CPU,
#                    README:13) — IDENTICAL to the windows value
#   candidate     : "ticklog" (custom SPSC ring, NOT rtrb) — IDENTICAL
#   threads       : 1,2,4,8,16 — IDENTICAL
#   ring_capacity : NOT passed => harness DEFAULT_RING_SIZE (1 MiB) — IDENTICAL
#   backpressure  : Drop (via --features policy-drop => policy()==Drop,
#                    main.rs:40) — IDENTICAL
#   sink          : NullSink (default) — IDENTICAL
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench || exit 1
mkdir -p bin results
NS=0.313097
LOG=results/wsl_ticklog_custom_drop.log
OUT=results/wsl_ticklog_custom_drop.json
: > "$LOG"
{
    echo "=== cargo build --release --features policy-drop (os=linux ELF) ==="
    ( cd rust/ticklog && cargo build --release --features policy-drop 2>&1 | tail -6 )
    RC=${PIPESTATUS[0]}
    H=$(find rust/ticklog/target/release -maxdepth 1 -type f -executable -name 'ticklog*harness*' | head -1)
    if [[ $RC -ne 0 || -z "$H" ]]; then
        echo "  BUILD FAILED rc=$RC (see $LOG)"
        exit 2
    fi
    cp -f "$H" bin/ticklog_policy_drop_harness
    echo "  ok -> bin/ticklog_policy_drop_harness ($(basename "$H"))"
    echo "=== custom-ring DROP sweep (os=linux ns_per_tick=$NS threads 1,2,4,8,16) ==="
    ./bin/ticklog_policy_drop_harness \
        --ns-per-tick "$NS" --output "$OUT" \
        --candidate ticklog --threads 1,2,4,8,16 2>&1 | tee -a "$LOG"
    echo "  exit=${PIPESTATUS[0]} -> $OUT"
} >> "$LOG" 2>&1
tail -4 "$LOG"
ls -la "$OUT" 2>/dev/null && echo "WSL_CUSTOM_DROP_DONE"
