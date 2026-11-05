#!/usr/bin/env bash
set -euo pipefail
source /mnt/c/Users/a_tas/Desktop/MFMLAB/Personal_Sync/20-Coding/P032-群晖Drive按需同步/work/fsearch-perf-runs/env.sh
harness="$P032_WORK/issue7-validation"
python3 - <<'PY'
import json,pathlib
root=pathlib.Path('/home/drive-native-test/.local/state/drive-native-ultra-validation/g4-006')
assert json.loads((root/'summary.json').read_text())['complete']
PY
exec python3 "$harness/run-g4.py" --archives "$harness/prepared-g4-007" \
  --run /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-007 \
  --candidate-commit ca86a85e2be9c83c7a59407fc0737114158ecf7f \
  --copy-target /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-006/target \
  --nautilus-private-prefix /home/drive-native-test/.local/state/drive-native-ultra-validation/g4-002/nautilus-private/prefix \
  --g4-authorized
