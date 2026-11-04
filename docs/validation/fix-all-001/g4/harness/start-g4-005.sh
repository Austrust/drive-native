#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/ultra-benchmark"
python3 - <<'PY'
import json,pathlib
root=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation')
assert json.loads((root/'g4-004/summary.json').read_text())['complete']
assert json.loads((root/'g4-004/final-integrity.json').read_text())['passed']
PY
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-005" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-005 \
  --candidate-commit bae1ef1ddc4061df586221557011f4e30f04d134 \
  --copy-target /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-004/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized
