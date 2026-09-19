#!/usr/bin/env bash
# setup.sh -- Fetch and build every cross-language benchmark candidate.
#
# Prerequisites:
#   - C compiler (cc)
#   - Rust toolchain (cargo)
#   - Go 1.23+
#   - CMake 3.20+
#   - Linux only: make (for NanoLog runtime)
#
# Usage:
#   ./setup.sh            # build everything for this platform
#   ./setup.sh --clean    # remove build artifacts first

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR"

OS="$(uname -s)"
NPROC="$(getconf _NPROCESSORS_ONLN 2>/dev/null || echo 4)"

# -- CLI ----------------------------------------------------------------

CLEAN=0
if [[ "${1:-}" == "--clean" ]]; then
    CLEAN=1
fi

if [[ "$CLEAN" -eq 1 ]]; then
    echo "=== Cleaning build artifacts ==="
    rm -rf bin/
    rm -f  calibrate
    rm -rf rust/ticklog/target/
    rm -rf cpp/quill/build/
    rm -rf cpp/nanolog/build/
    echo "  done"
fi

mkdir -p bin results

# -- calibrate.c --------------------------------------------------------

echo "=== Building calibrate ==="
cc -O2 -o calibrate calibrate.c
echo "  done -> calibrate"

# -- Rust: ticklog ------------------------------------------------------

echo "=== Building rust/ticklog (default custom ring) ==="
cd rust/ticklog
cargo build --release 2>&1
cp target/release/ticklog-cross-lang-harness "$SCRIPT_DIR/bin/ticklog_harness"
cd "$SCRIPT_DIR"
echo "  done -> bin/ticklog_harness"

echo "=== Building rust/ticklog (backend-ringbuf) ==="
cd rust/ticklog
cargo build --release --features backend-ringbuf 2>&1
cp target/release/ticklog-cross-lang-harness "$SCRIPT_DIR/bin/ticklog_ringbuf_harness"
cd "$SCRIPT_DIR"
echo "  done -> bin/ticklog_ringbuf_harness"

echo "=== Building rust/ticklog (backend-triple-buffer) ==="
cd rust/ticklog
cargo build --release --features backend-triple-buffer 2>&1
cp target/release/ticklog-cross-lang-harness "$SCRIPT_DIR/bin/ticklog_triple_buffer_harness"
cd "$SCRIPT_DIR"
echo "  done -> bin/ticklog_triple_buffer_harness"

# All non-Block backpressure policies (each is a separate ticklog feature).
# The default policy (no feature) is Block; every other policy builds its own
# harness binary so run.sh can exercise them side-by-side next to the C++
# (quill/NanoLog) and Go (zerolog/zap) candidates.
echo "=== Building rust/ticklog (all non-Block policies) ==="
cd rust/ticklog
for pol in fastwm watermark fast quill nanolog; do
    echo "  + policy-$pol -> bin/ticklog_policy_${pol}_harness"
    cargo build --release --features "policy-$pol" 2>&1
    cp target/release/ticklog-cross-lang-harness \
        "$SCRIPT_DIR/bin/ticklog_policy_${pol}_harness"
done
cd "$SCRIPT_DIR"
echo "  done -> bin/ticklog_policy_*_harness (fastwm watermark fast quill nanolog)"

# -- Go: zerolog + zap --------------------------------------------------

echo "=== Building Go harnesses ==="
cd go
go build -buildvcs=false -o ../bin/zerolog_harness ./zerolog/
echo "  done -> bin/zerolog_harness"
go build -buildvcs=false -o ../bin/zap_harness ./zap/
echo "  done -> bin/zap_harness"
cd "$SCRIPT_DIR"

# -- C++: Quill ---------------------------------------------------------

echo "=== Building cpp/quill ==="
cd cpp/quill
cmake -B build -DCMAKE_BUILD_TYPE=Release 2>&1
cmake --build build 2>&1
cd "$SCRIPT_DIR"
echo "  done -> cpp/quill/build/quill_harness"

# -- C++: NanoLog (Linux only) ------------------------------------------

if [[ "$OS" == "Linux" ]]; then
    echo "=== Building cpp/nanolog ==="

    # Fetch NanoLog runtime if not already present.
    if [ ! -d "$HOME/NanoLog" ]; then
        echo "  fetching NanoLog..."
        git clone --depth 1 https://github.com/PlatformLab/NanoLog.git "$HOME/NanoLog" 2>&1
    fi

    # NanoLog's runtime Makefile invokes `python` (its code-generating
    # preprocessor). Distros that ship only `python3` (e.g. Ubuntu 24.04)
    # have no `python` on PATH, so provide a build-local shim.
    if ! command -v python &>/dev/null; then
        if command -v python3 &>/dev/null; then
            echo "  'python' not found; using build-local python->python3 shim"
            mkdir -p "$SCRIPT_DIR/.pyshim"
            ln -sf "$(command -v python3)" "$SCRIPT_DIR/.pyshim/python"
            export PATH="$SCRIPT_DIR/.pyshim:$PATH"
        else
            echo "error: NanoLog build needs Python; neither 'python' nor 'python3' found" >&2
            exit 1
        fi
    fi

    echo "  building runtime (make -j$NPROC)..."
    make -C "$HOME/NanoLog/runtime" -j"$NPROC" 2>&1

    cd cpp/nanolog
    cmake -B build -DCMAKE_BUILD_TYPE=Release \
        -DNANOLOG_RUNTIME_DIR="$HOME/NanoLog/runtime" 2>&1
    cmake --build build 2>&1
    cd "$SCRIPT_DIR"
    echo "  done -> cpp/nanolog/build/nanolog_harness"
else
    echo "=== Skipping cpp/nanolog (Linux only) ==="
fi

# -- Done ---------------------------------------------------------------

echo ""
echo "=== All builds complete ==="
echo ""
echo "Binaries:"
echo "  ./calibrate"
echo "  bin/ticklog_harness"
echo "  bin/ticklog_ringbuf_harness"
echo "  bin/ticklog_triple_buffer_harness"
echo "  bin/zerolog_harness"
echo "  bin/zap_harness"
echo "  cpp/quill/build/quill_harness"
if [[ "$OS" == "Linux" ]]; then
    echo "  cpp/nanolog/build/nanolog_harness"
fi
echo ""
echo "Next: ./run.sh"
