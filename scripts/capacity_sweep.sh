#!/usr/bin/env bash
set -euo pipefail

# Sustained logging throughput vs per-thread ring capacity, comparing the
# custom SPSC ring with the experimental ringbuffer-crate backend, under
# Backpressure::Block (no drops; producers throttle to the drain).
#
# Runs examples/capacity_probe.rs for every capacity in the matrix, for both
# backends, REP times each, and keeps the MEDIAN ns/log per cell (damped
# against scheduler noise).
#
# The default RECORDS (300000/thread) is chosen so a single thread's slot-
# aligned burst (~19 MiB) exceeds even the largest ring; every cell then runs
# in the same backpressure regime, and only buffering distinguishes its
# sustained rate.
#
# Usage:
#   ./scripts/capacity_sweep.sh [THREADS [RECORDS_PER_THREAD [REPS]]]
#
# Environment:
#   THREADS=..., RECORDS=..., REPS=... can be used instead of positionals.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
LOGS_DIR="$PROJECT_DIR/logs/benches"
TIMESTAMP="$(date +%Y%m%d%H%M%S)"

THREADS="${THREADS:-${1:-16}}"
RECORDS="${RECORDS:-${2:-300000}}"
REPS="${REPS:-${3:-5}}"
CAPACITIES=(65536 262144 1048576 4194304 16777216)

# Ensure cargo is on PATH. Source .cargo/env from common locations.
if ! command -v cargo &>/dev/null; then
    for cargo_env in "${CARGO_HOME:+$CARGO_HOME/env}" "$HOME/.cargo/env"; do
        if [[ -f "$cargo_env" ]]; then
            source "$cargo_env"
            break
        fi
    done
fi

cd "$PROJECT_DIR"
mkdir -p "$LOGS_DIR"

OUT="$LOGS_DIR/capacity-${TIMESTAMP}.csv"
echo "=== ticklog capacity sweep ==="
echo "Timestamp: $TIMESTAMP"
echo "Threads:   $THREADS"
echo "Records:   $RECORDS per thread"
echo "Reps:      $REPS"
echo "Capacities: ${CAPACITIES[*]}"
echo "Log:       $OUT"
echo ""

echo "build,backend,capacity,threads,records,reps,median_ns_per_log,median_recs_per_sec"

> "$OUT"
printf "build,backend,capacity,threads,records,reps,median_ns_per_log,median_recs_per_sec\n" >> "$OUT"

median() {
    sort -n | awk '{ a[NR]=$1 } END { print (NR % 2 ? a[(NR + 1) / 2] : (a[NR / 2] + a[NR / 2 + 1]) / 2) }'
}

run_cell() {
    local build="$1" backend="$2" cap="$3"
    local i line ns=() rps=() median_ns median_rps
    for i in $(seq 1 "$REPS"); do
        line="$("$PROJECT_DIR/target/release/examples/capacity_probe" "$cap" "$THREADS" "$RECORDS")"
        ns+=("$(cut -d, -f6 <<<"$line")")
        rps+=("$(cut -d, -f7 <<<"$line")")
    done
    median_ns="$(printf '%s\n' "${ns[@]}" | median)"
    median_rps="$(printf '%s\n' "${rps[@]}" | median)"
    printf "%s,%s,%s,%s,%s,%s,%.1f,%.0f\n" \
        "$build" "$backend" "$cap" "$THREADS" "$RECORDS" "$REPS" "$median_ns" "$median_rps"
}

for spec in "custom:" "ringbuffer:--features backend-ringbuffer" \
            "ringbuf:--features backend-ringbuf" \
            "triple-buffer:--features backend-triple-buffer"; do
    build="${spec%%:*}"
    features="${spec#*:}"
    echo -n "building $build backend ... "
    cargo build --release --example capacity_probe $features
    echo "OK"

    for cap in "${CAPACITIES[@]}"; do
        row="$(run_cell "$build" "$build" "$cap")"
        echo "$row" | tee -a "$OUT"
    done
done

echo ""
echo "Saved: $OUT"