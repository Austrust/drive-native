#!/usr/bin/env bash
set -euo pipefail
[[ ${1:-} == coordinator-exclusive-START ]]
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
base=/home/drive-native-test/.local/state/drive-native-ultra-benchmark
candidate=bae1ef1ddc4061df586221557011f4e30f04d134
window="$base/window-001"
test ! -e "$window"
mkdir -m700 "$window"
df -B1 "$base" > "$window/private-filesystem-space.txt"
date --iso-8601=seconds > "$window/start.txt"
python3 "$harness/run-paired.py" --mode primary --exclusive-window-confirmed \
  --candidate-commit "$candidate" --build-manifest "$base/build-002/build.json" \
  --run "$base/primary-001" > "$window/primary.log" 2>&1
python3 "$harness/run-paired.py" --mode diagnostic --exclusive-window-confirmed \
  --candidate-commit "$candidate" --build-manifest "$base/build-002/build.json" \
  --run "$base/diagnostic-001" > "$window/diagnostic.log" 2>&1
date --iso-8601=seconds > "$window/end.txt"
echo 'G5 primary and diagnostic commands completed; inspect complete/truth/mount flags before declaring pass'
