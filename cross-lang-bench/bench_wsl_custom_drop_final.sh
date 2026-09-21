#!/usr/bin/env bash
# WSL drop-sweep launcher (mirror of windows ticklog_custom_drop).
#  - harness = the SAME ticklog crate as the windows run, but built with
#    --features policy-drop so main.rs policy() -> Backpressure::Drop
#    (cfg!(feature="policy-drop") arm, main.rs:40-41) and the harness name
#    recorded in json = "ticklog" (candidate default; NOT rtrb, NOT rtrb2).
#  - ns-per-tick 0.313097 (identical to the windows windows/ticklog_custom_drop.json
#    — this is the WSL2-calibrated 0.313097 from README:13, SAME physical CPU).
#  - threads 1,2,4,8,16 (identical sweep matrix).
#  - ring-capacity NOT passed -> harness DEFAULT_RING_SIZE (1 MiB) — byte-equal
#    to the windows custom-drop run (also used the 1 MiB default).
#  - output -> results/wsl_ticklog_custom_drop.json  (wsl_ prefix: os=linux ELF).
set -uo pipefail
cd /mnt/c/Users/Admin/Desktop/ticklog/cross-lang-bench || exit 1
mkdir -p bin results
NS=0.313097
LOG=results/wsl_ticklog_custom_drop.log
OUT=results/wsl_ticklog_custom_drop.json
: > "$LOG"
{
    echo "=== building harness --features policy-drop (Drop; os=linux ELF) ==="
    ( cd rust/ticklog && cargo build --release --features policy-drop 2>&1 | tail -8 )
    RC=${PIPESTATUS[0]}
    HARNESS=$(find rust/ticklog/target/release -maxdepth 1 -type f -executable -name 'ticklog*harness*' | head -1)
    if [[ $RC -ne 0 || -z "$HARNESS" ]]; then
        echo "  BUILD FAILED rc=$RC (see $LOG)"
        exit 2
    fi
    cp -f "$HARNESS" bin/ticklog_policy_drop_harness
    echo "  ok -> bin/ticklog_policy_drop_harness ($(basename "$HARNESS"))"
    echo "=== running custom-ring DROP sweep (os=linux, ns-per-tick=$NS, threads 1,2,4,8,16) ==="
    ./bin/ticklog_policy_drop_harness --ns-per-tick "$NS" \
        --output "$OUT" \
        --candidate ticklog --threads 1,2,4,8,16 2>&1 | tee -a "$LOG"
    RC=${PIPESTATUS[0]}
    echo "  exit=$RC -> $OUT"
} >> "$LOG" 2>&1
tail -6 "$LOG"
ls -la "$OUT" 2>/dev/null && echo "WSL_DROP_DONE -> $OUT"