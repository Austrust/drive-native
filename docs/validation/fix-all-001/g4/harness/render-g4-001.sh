#!/usr/bin/env bash
set -euo pipefail
run=/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-001
harness=/mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/ultra-benchmark
export PYTHONPATH="$run/source/desktop" PYTHONDONTWRITEBYTECODE=1 QT_QPA_PLATFORM=offscreen
export XDG_CONFIG_HOME="$run/render-config" XDG_DATA_HOME="$run/render-data" XDG_CACHE_HOME="$run/render-cache" XDG_RUNTIME_DIR="$run/render-runtime" TMPDIR="$run/tmp"
mkdir -m700 "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME" "$XDG_RUNTIME_DIR"
unset DBUS_SESSION_BUS_ADDRESS DISPLAY WAYLAND_DISPLAY
exec dbus-run-session -- /opt/drive-native-build/gui-venv/bin/python "$harness/render-mainwindow-permissions.py" "$run/mainwindow-permission-screenshots" > "$run/logs/mainwindow-permission-render.log" 2>&1
