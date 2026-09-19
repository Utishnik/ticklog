#!/usr/bin/env bash
set -uo pipefail
cd /root
if [ ! -d NanoLog/runtime ]; then
    attempt=0
    while [ $attempt -lt 4 ]; do
        attempt=$((attempt+1))
        echo "--- attempt $attempt: git clone NanoLog ---"
        rm -rf NanoLog
        if git clone --depth 1 https://github.com/PlatformLab/NanoLog.git /root/NanoLog 2>&1 | tail -1; then
            break
        fi
        sleep 4
    done
fi
echo "--- contents ---"
ls NanoLog/runtime 2>&1 | head -8
