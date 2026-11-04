#!/usr/bin/env bash
set -euo pipefail
run_id=${1:?g4-NNN}
candidate=${2:?frozen full commit}
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
run="/home/drive-native-test/.local/state/drive-native-ultra-validation/$run_id"
python3 "$harness/run-g4-supplement.py" --g4-authorized --run "$run" --candidate-commit "$candidate"
bash "$harness/functional-g4.sh" "$run_id" "$candidate"
bash "$harness/schema-g4.sh" "$run_id" "$candidate"
bash "$harness/history-scale-g4.sh" "$run_id" "$candidate"
python3 "$harness/finalize-g4-integrity.py" "$run"
