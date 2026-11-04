#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
python3 "$harness/run-g4-supplement.py" --g4-authorized \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002 \
  --candidate-commit 652eabe741396c8332a7182599aaf4e53d5af48b
bash "$harness/functional-g4-002.sh"
