#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/issue7-validation"
test ! -e "$harness/prepared-g4-012/launcher.log"
export CARGO_INCREMENTAL=0
python3 -B "$harness/run-g4.py" --archives "$harness/prepared-g4-012" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-012 \
  --candidate-commit 9c9aabae447d92275f93dae65320788240a03113 \
  --registry-seed /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-009/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized 2>&1 | tee "$harness/prepared-g4-012/launcher.log"
