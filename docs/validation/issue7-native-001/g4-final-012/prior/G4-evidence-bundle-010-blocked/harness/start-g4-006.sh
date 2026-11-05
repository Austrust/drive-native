#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/issue7-validation"
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-006" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-006 \
  --candidate-commit d39a59769e0c0abcde033ae0f74bb2806f218d8a \
  --copy-target /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-005/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized --preflight-only
