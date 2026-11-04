#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
python3 "$harness/run-g4-supplement.py" --g4-authorized \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-003 \
  --candidate-commit c744729141ca8b23bde63c61ca8a644bdfdaee81
bash "$harness/functional-g4-003.sh"
bash "$harness/schema-g4-003.sh"
python3 "$harness/finalize-g4-integrity.py" /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-003
