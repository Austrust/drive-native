#!/usr/bin/env bash
# Execute only after the coordinator authorizes post-G4 final-candidate builds.
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
exec python3 "$P032_WORK/ultra-benchmark/build-candidate-reuse.py" --build-authorized \
  --archives "$P032_WORK/ultra-benchmark/prepared-g4-005" \
  --reuse-build /home/drive-native-test/.local/state/drive-native-ultra-benchmark/build-001 \
  --build-root /home/drive-native-test/.local/state/drive-native-ultra-benchmark/build-002
